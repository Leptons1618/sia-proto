use crate::storage::Storage;
use anyhow::Result;
use tokio::net::{UnixListener, UnixStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Deserialize, Serialize};
use log::{info, error, warn};
use std::time::{SystemTime, UNIX_EPOCH};

static START_TIME: std::sync::OnceLock<u64> = std::sync::OnceLock::new();

#[derive(Deserialize)]
#[serde(tag = "method")]
enum IpcRequest {
    #[serde(rename = "status")]
    Status,
    #[serde(rename = "list")]
    List { limit: Option<i32> },
    #[serde(rename = "show")]
    Show { event_id: String },
}

#[derive(Serialize)]
struct IpcResponse {
    success: bool,
    data: serde_json::Value,
}

pub async fn start_ipc_server(storage: Storage, socket_path: String) -> Result<()> {
    // Remove old socket if exists
    let _ = std::fs::remove_file(&socket_path);
    
    let listener = UnixListener::bind(&socket_path)?;
    info!("IPC server listening on {}", socket_path);
    
    // Store start time
    START_TIME.get_or_init(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });
    
    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let storage = storage.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_client(stream, storage).await {
                            error!("Client error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Accept error: {}", e);
                }
            }
        }
    });
    
    Ok(())
}

async fn handle_client(mut stream: UnixStream, storage: Storage) -> Result<()> {
    let mut buffer = vec![0u8; 8192];
    let n = stream.read(&mut buffer).await?;
    
    if n == 0 {
        return Ok(());
    }
    
    let request_str = String::from_utf8_lossy(&buffer[..n]);
    info!("IPC request: {}", request_str.trim());
    
    let response = match serde_json::from_str::<IpcRequest>(&request_str) {
        Ok(req) => handle_request(req, &storage).await,
        Err(e) => {
            warn!("Invalid request: {}", e);
            IpcResponse {
                success: false,
                data: serde_json::json!({"error": format!("Invalid request: {}", e)}),
            }
        }
    };
    
    let response_json = serde_json::to_string(&response)?;
    stream.write_all(response_json.as_bytes()).await?;
    stream.write_all(b"\n").await?;
    
    Ok(())
}

async fn handle_request(req: IpcRequest, storage: &Storage) -> IpcResponse {
    match req {
        IpcRequest::Status => handle_status(storage).await,
        IpcRequest::List { limit } => handle_list(storage, limit.unwrap_or(20)).await,
        IpcRequest::Show { event_id } => handle_show(storage, &event_id).await,
    }
}

async fn handle_status(storage: &Storage) -> IpcResponse {
    let uptime_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() - START_TIME.get().unwrap_or(&0);
    
    let (critical, warning, info) = storage.get_event_counts().await.unwrap_or((0, 0, 0));
    
    IpcResponse {
        success: true,
        data: serde_json::json!({
            "status": "running",
            "uptime_seconds": uptime_secs,
            "collectors": {
                "cpu": "active",
                "memory": "active"
            },
            "events": {
                "critical": critical,
                "warning": warning,
                "info": info
            }
        }),
    }
}

async fn handle_list(storage: &Storage, limit: i32) -> IpcResponse {
    match storage.get_recent_events(limit).await {
        Ok(events) => {
            let events_json: Vec<_> = events.iter().map(|e| {
                serde_json::json!({
                    "event_id": e.event_id,
                    "ts": e.ts,
                    "severity": e.severity,
                    "type": e.type_,
                    "status": e.status,
                })
            }).collect();
            
            IpcResponse {
                success: true,
                data: serde_json::json!({ "events": events_json }),
            }
        }
        Err(e) => {
            IpcResponse {
                success: false,
                data: serde_json::json!({"error": format!("Failed to fetch events: {}", e)}),
            }
        }
    }
}

async fn handle_show(storage: &Storage, event_id: &str) -> IpcResponse {
    match storage.get_event_by_id(event_id).await {
        Ok(Some(event)) => {
            // Try to parse snapshot as JSON
            let snapshot: serde_json::Value = serde_json::from_slice(&event.snapshot)
                .unwrap_or_else(|_| serde_json::json!({}));
            
            IpcResponse {
                success: true,
                data: serde_json::json!({
                    "event_id": event.event_id,
                    "ts": event.ts,
                    "severity": event.severity,
                    "type": event.type_,
                    "service_id": event.service_id,
                    "status": event.status,
                    "snapshot": snapshot,
                }),
            }
        }
        Ok(None) => {
            IpcResponse {
                success: false,
                data: serde_json::json!({"error": format!("Event {} not found", event_id)}),
            }
        }
        Err(e) => {
            IpcResponse {
                success: false,
                data: serde_json::json!({"error": format!("Database error: {}", e)}),
            }
        }
    }
}
