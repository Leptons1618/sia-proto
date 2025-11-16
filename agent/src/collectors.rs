use sysinfo::{System, SystemExt, ProcessExt, CpuExt, PidExt};
use tokio::time::{sleep, Duration};
use tokio::sync::mpsc;
use common::{Event, ThresholdsConfig};
use log::{info, warn};
use crate::ipc::MetricsHandle;

pub async fn start_collectors(tx: mpsc::Sender<Event>, cpu_interval: u64, metrics: MetricsHandle, thresholds: ThresholdsConfig) -> anyhow::Result<()> {
    info!("Starting collectors with {}s CPU interval", cpu_interval);
    info!("Thresholds: CPU warning={}% critical={}%, Memory warning={}% critical={}%", 
          thresholds.cpu_warning, thresholds.cpu_critical, 
          thresholds.memory_warning, thresholds.memory_critical);
    
    // Clone thresholds for CPU collector
    let thresholds_cpu = thresholds.clone();
    
    // CPU collector
    let tx_cpu = tx.clone();
    let metrics_cpu = metrics.clone();
    tokio::spawn(async move {
        let mut sys = System::new_all();
        let mut high_cpu_count = 0;
        
        // Initial refresh to initialize CPU info
        sys.refresh_cpu();
        sleep(Duration::from_millis(250)).await; // Wait for accurate measurement
        
        loop {
            // Refresh CPU info and wait for accurate measurement
            sys.refresh_cpu();
            sleep(Duration::from_millis(250)).await;
            sys.refresh_cpu();
            
            let cpu_usage = sys.global_cpu_info().cpu_usage();
            
            // Update metrics
            {
                let mut metrics_guard = metrics_cpu.write().await;
                metrics_guard.cpu_usage = cpu_usage;
            }
            
            // Generate events based on configurable thresholds
            if cpu_usage > thresholds_cpu.cpu_critical {
                warn!("Critical CPU usage: {:.1}% (threshold: {:.1}%)", cpu_usage, thresholds_cpu.cpu_critical);
                let event = create_cpu_event(cpu_usage, "CRITICAL", &sys, thresholds_cpu.cpu_critical);
                let _ = tx_cpu.send(event).await;
                high_cpu_count = 0;
            } else if cpu_usage > thresholds_cpu.cpu_warning {
                high_cpu_count += 1;
                if high_cpu_count >= thresholds_cpu.cpu_sustained_count {
                    warn!("High CPU usage: {:.1}% (threshold: {:.1}%)", cpu_usage, thresholds_cpu.cpu_warning);
                    let event = create_cpu_event(cpu_usage, "WARNING", &sys, thresholds_cpu.cpu_warning);
                    let _ = tx_cpu.send(event).await;
                }
            } else {
                high_cpu_count = 0;
            }
            
            // Sleep for the remaining interval (minus the 250ms we already waited)
            let sleep_duration = if cpu_interval > 0 {
                Duration::from_secs(cpu_interval).saturating_sub(Duration::from_millis(250))
            } else {
                Duration::from_secs(1)
            };
            sleep(sleep_duration).await;
        }
    });
    
    // Memory collector
    let tx_mem = tx.clone();
    let metrics_mem = metrics.clone();
    let thresholds_mem = thresholds.clone();
    tokio::spawn(async move {
        let mut sys = System::new_all();
        
        loop {
            sys.refresh_memory();
            
            let total_mem = sys.total_memory();
            let used_mem = sys.used_memory();
            let mem_percent = (used_mem as f32 / total_mem as f32) * 100.0;
            
            // Update metrics
            {
                let mut metrics_guard = metrics_mem.write().await;
                metrics_guard.memory_used_mb = used_mem / 1024 / 1024;
                metrics_guard.memory_total_mb = total_mem / 1024 / 1024;
                metrics_guard.memory_percent = mem_percent;
            }
            
            // Generate events based on configurable thresholds
            if mem_percent > thresholds_mem.memory_critical {
                warn!("Critical memory usage: {:.1}% (threshold: {:.1}%)", mem_percent, thresholds_mem.memory_critical);
                let event = create_memory_event(mem_percent, used_mem, total_mem, "CRITICAL", &sys, thresholds_mem.memory_critical);
                let _ = tx_mem.send(event).await;
            } else if mem_percent > thresholds_mem.memory_warning {
                warn!("High memory usage: {:.1}% (threshold: {:.1}%)", mem_percent, thresholds_mem.memory_warning);
                let event = create_memory_event(mem_percent, used_mem, total_mem, "WARNING", &sys, thresholds_mem.memory_warning);
                let _ = tx_mem.send(event).await;
            }
            
            sleep(Duration::from_secs(cpu_interval)).await;
        }
    });
    
    Ok(())
}

fn create_cpu_event(cpu_usage: f32, severity: &str, sys: &System, threshold: f32) -> Event {
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
        "threshold": threshold,
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

fn create_memory_event(mem_percent: f32, used: u64, total: u64, severity: &str, sys: &System, threshold: f32) -> Event {
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
        "threshold": threshold,
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
