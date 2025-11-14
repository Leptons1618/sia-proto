use clap::{Parser, Subcommand};
use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "sia-cli")]
#[command(about = "SIA System Insight Agent CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show agent status and metrics
    Status,
    /// List recent events
    List {
        #[arg(short, long, default_value = "20")]
        limit: i32,
    },
    /// Show detailed event information
    Show {
        /// Event ID to display
        event_id: String,
    },
}

#[derive(Serialize)]
struct IpcRequest {
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    event_id: Option<String>,
}

#[derive(Deserialize)]
struct IpcResponse {
    success: bool,
    data: serde_json::Value,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.cmd {
        Commands::Status => {
            let response = send_request("status", None, None).await?;
            print_status(response);
        }
        Commands::List { limit } => {
            let response = send_request("list", Some(limit), None).await?;
            print_list(response);
        }
        Commands::Show { event_id } => {
            let response = send_request("show", None, Some(event_id)).await?;
            print_show(response);
        }
    }
    
    Ok(())
}

async fn send_request(method: &str, limit: Option<i32>, event_id: Option<String>) -> Result<IpcResponse> {
    let socket_path = "/tmp/sia.sock";
    let mut stream = UnixStream::connect(socket_path).await?;
    
    let request = IpcRequest {
        method: method.to_string(),
        limit,
        event_id,
    };
    
    let request_json = serde_json::to_string(&request)?;
    stream.write_all(request_json.as_bytes()).await?;
    stream.shutdown().await?;
    
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).await?;
    
    let response: IpcResponse = serde_json::from_slice(&buffer)?;
    Ok(response)
}

fn print_status(response: IpcResponse) {
    if !response.success {
        eprintln!("Error: {}", response.data.get("error").unwrap_or(&serde_json::json!("Unknown error")));
        return;
    }
    
    let data = &response.data;
    let uptime = data["uptime_seconds"].as_u64().unwrap_or(0);
    let uptime_str = format_uptime(uptime);
    
    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║                    SIA Agent Status                           ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║ Uptime:     {:49} ║", uptime_str);
    println!("║ Status:     {:49} ║", data["status"].as_str().unwrap_or("unknown"));
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║ Collectors:                                                   ║");
    println!("║   CPU:      {:49} ║", format!("✓ {}", data["collectors"]["cpu"].as_str().unwrap_or("unknown")));
    println!("║   Memory:   {:49} ║", format!("✓ {}", data["collectors"]["memory"].as_str().unwrap_or("unknown")));
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║ Events (open):                                                ║");
    println!("║   Critical: {:49} ║", data["events"]["critical"].as_i64().unwrap_or(0));
    println!("║   Warning:  {:49} ║", data["events"]["warning"].as_i64().unwrap_or(0));
    println!("║   Info:     {:49} ║", data["events"]["info"].as_i64().unwrap_or(0));
    println!("╚═══════════════════════════════════════════════════════════════╝\n");
}

fn print_list(response: IpcResponse) {
    if !response.success {
        eprintln!("Error: {}", response.data.get("error").unwrap_or(&serde_json::json!("Unknown error")));
        return;
    }
    
    let empty_vec = vec![];
    let events = response.data["events"].as_array().unwrap_or(&empty_vec);
    
    if events.is_empty() {
        println!("\nNo events found.\n");
        return;
    }
    
    println!("\n┌──────────────────────────┬────────────────────────┬──────────┬───────────────┬────────┐");
    println!("│ Event ID                 │ Timestamp              │ Severity │ Type          │ Status │");
    println!("├──────────────────────────┼────────────────────────┼──────────┼───────────────┼────────┤");
    
    for event in events {
        let event_id = event["event_id"].as_str().unwrap_or("?");
        let ts = event["ts"].as_i64().unwrap_or(0);
        let ts_str = format_timestamp(ts);
        let severity = event["severity"].as_str().unwrap_or("?");
        let type_ = event["type"].as_str().unwrap_or("?");
        let status = event["status"].as_str().unwrap_or("?");
        
        println!("│ {:24} │ {:22} │ {:8} │ {:13} │ {:6} │",
            truncate(event_id, 24),
            ts_str,
            severity,
            truncate(type_, 13),
            status
        );
    }
    
    println!("└──────────────────────────┴────────────────────────┴──────────┴───────────────┴────────┘\n");
}

fn print_show(response: IpcResponse) {
    if !response.success {
        eprintln!("Error: {}", response.data.get("error").unwrap_or(&serde_json::json!("Unknown error")));
        return;
    }
    
    let event = &response.data;
    let ts = event["ts"].as_i64().unwrap_or(0);
    let ts_str = format_timestamp(ts);
    
    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║                      Event Details                            ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║ ID:         {:49} ║", event["event_id"].as_str().unwrap_or("?"));
    println!("║ Time:       {:49} ║", ts_str);
    println!("║ Severity:   {:49} ║", event["severity"].as_str().unwrap_or("?"));
    println!("║ Type:       {:49} ║", event["type"].as_str().unwrap_or("?"));
    println!("║ Service:    {:49} ║", event["service_id"].as_str().unwrap_or("?"));
    println!("║ Status:     {:49} ║", event["status"].as_str().unwrap_or("?"));
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║ Snapshot Data:                                                ║");
    
    if let Some(snapshot) = event.get("snapshot") {
        let snapshot_str = serde_json::to_string_pretty(snapshot).unwrap_or_default();
        for line in snapshot_str.lines().take(20) {
            println!("║ {:61} ║", truncate(line, 61));
        }
    }
    
    println!("╚═══════════════════════════════════════════════════════════════╝\n");
}

fn format_uptime(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    
    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

fn format_timestamp(ts: i64) -> String {
    use chrono::{DateTime, Utc, TimeZone};
    
    if let Some(dt) = Utc.timestamp_opt(ts, 0).single() {
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    } else {
        "Invalid timestamp".to_string()
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        format!("{:width$}", s, width = max_len)
    } else {
        format!("{}..", &s[..max_len-2])
    }
}
