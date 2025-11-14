use sysinfo::{System, SystemExt, ProcessExt, CpuExt, PidExt};
use tokio::time::{sleep, Duration};
use tokio::sync::mpsc;
use common::Event;
use log::{info, warn};

pub async fn start_collectors(tx: mpsc::Sender<Event>, cpu_interval: u64) -> anyhow::Result<()> {
    info!("Starting collectors with {}s CPU interval", cpu_interval);
    
    // CPU collector
    let tx_cpu = tx.clone();
    tokio::spawn(async move {
        let mut sys = System::new_all();
        let mut high_cpu_count = 0;
        
        loop {
            sys.refresh_cpu();
            sys.refresh_memory();
            
            let cpu_usage = sys.global_cpu_info().cpu_usage();
            
            // Generate events based on thresholds
            if cpu_usage > 95.0 {
                warn!("Critical CPU usage: {:.1}%", cpu_usage);
                let event = create_cpu_event(cpu_usage, "CRITICAL", &sys);
                let _ = tx_cpu.send(event).await;
                high_cpu_count = 0;
            } else if cpu_usage > 80.0 {
                high_cpu_count += 1;
                if high_cpu_count >= 2 {
                    warn!("High CPU usage: {:.1}%", cpu_usage);
                    let event = create_cpu_event(cpu_usage, "WARNING", &sys);
                    let _ = tx_cpu.send(event).await;
                }
            } else {
                high_cpu_count = 0;
            }
            
            sleep(Duration::from_secs(cpu_interval)).await;
        }
    });
    
    // Memory collector
    let tx_mem = tx.clone();
    tokio::spawn(async move {
        let mut sys = System::new_all();
        
        loop {
            sys.refresh_memory();
            
            let total_mem = sys.total_memory();
            let used_mem = sys.used_memory();
            let mem_percent = (used_mem as f32 / total_mem as f32) * 100.0;
            
            if mem_percent > 95.0 {
                warn!("Critical memory usage: {:.1}%", mem_percent);
                let event = create_memory_event(mem_percent, used_mem, total_mem, "CRITICAL", &sys);
                let _ = tx_mem.send(event).await;
            } else if mem_percent > 85.0 {
                warn!("High memory usage: {:.1}%", mem_percent);
                let event = create_memory_event(mem_percent, used_mem, total_mem, "WARNING", &sys);
                let _ = tx_mem.send(event).await;
            }
            
            sleep(Duration::from_secs(cpu_interval)).await;
        }
    });
    
    Ok(())
}

fn create_cpu_event(cpu_usage: f32, severity: &str, sys: &System) -> Event {
    use chrono::Utc;
    use serde_json::json;
    
    let event_id = format!("cpu_{}", Utc::now().timestamp_millis());
    let ts = Utc::now().to_rfc3339();
    
    // Find top CPU process
    let top_proc = sys.processes()
        .values()
        .max_by(|a, b| a.cpu_usage().partial_cmp(&b.cpu_usage()).unwrap());
    
    let entity = json!({
        "cpu_usage": cpu_usage,
        "type": "system_cpu",
        "top_process": top_proc.map(|p| json!({
            "name": p.name(),
            "pid": p.pid().as_u32(),
            "cpu": p.cpu_usage()
        }))
    });
    
    let evidence = json!({
        "threshold": if severity == "CRITICAL" { 95.0 } else { 80.0 },
        "sustained": severity == "WARNING",
        "timestamp": ts.clone()
    });
    
    Event {
        event_id,
        ts,
        severity: severity.to_string(),
        r#type: "cpu_high".to_string(),
        entity,
        evidence,
        suggestion: None,
        status: "open".to_string(),
    }
}

fn create_memory_event(mem_percent: f32, used: u64, total: u64, severity: &str, sys: &System) -> Event {
    use chrono::Utc;
    use serde_json::json;
    
    let event_id = format!("mem_{}", Utc::now().timestamp_millis());
    let ts = Utc::now().to_rfc3339();
    
    // Find top memory processes
    let mut procs: Vec<_> = sys.processes().values().collect();
    procs.sort_by(|a, b| b.memory().cmp(&a.memory()));
    let top_procs: Vec<_> = procs.iter().take(5).map(|p| json!({
        "name": p.name(),
        "pid": p.pid().as_u32(),
        "memory_mb": p.memory() / 1024 / 1024
    })).collect();
    
    let entity = json!({
        "memory_percent": mem_percent,
        "used_mb": used / 1024 / 1024,
        "total_mb": total / 1024 / 1024,
        "type": "system_memory",
        "top_processes": top_procs
    });
    
    let evidence = json!({
        "threshold": if severity == "CRITICAL" { 95.0 } else { 85.0 },
        "timestamp": ts.clone()
    });
    
    Event {
        event_id,
        ts,
        severity: severity.to_string(),
        r#type: "memory_high".to_string(),
        entity,
        evidence,
        suggestion: None,
        status: "open".to_string(),
    }
}
