use crate::storage::Storage;
use crate::llm::LlmClient;
use common::{Event, ThresholdsConfig};
use anyhow::Result;
use tokio::net::{UnixListener, UnixStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Deserialize, Serialize};
use log::{info, error, warn};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};

static START_TIME: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
static LLM_CONNECTION_CACHE: std::sync::OnceLock<Arc<AtomicBool>> = std::sync::OnceLock::new();

#[derive(Clone, Default)]
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub memory_percent: f32,
}

pub type MetricsHandle = Arc<RwLock<SystemMetrics>>;

#[derive(Deserialize)]
#[serde(tag = "method")]
enum IpcRequest {
    #[serde(rename = "status")]
    Status,
    #[serde(rename = "list")]
    List { limit: Option<i32> },
    #[serde(rename = "show")]
    Show { event_id: String },
    #[serde(rename = "analyze")]
    Analyze { event_id: String },
}

#[derive(Serialize)]
struct IpcResponse {
    success: bool,
    data: serde_json::Value,
}

pub async fn start_ipc_server(storage: Storage, socket_path: String, metrics: MetricsHandle, llm_client: Option<LlmClient>, thresholds: ThresholdsConfig) -> Result<()> {
    // Remove old socket if exists
    let _ = std::fs::remove_file(&socket_path);
    
    let listener = UnixListener::bind(&socket_path)?;
    
    // Set socket permissions to 0666 (read/write for all users)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o666);
        std::fs::set_permissions(&socket_path, perms)?;
    }
    
    info!("IPC server listening on {}", socket_path);
    
    // Store start time
    START_TIME.get_or_init(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });
    
    // Initialize LLM connection cache and test connection once
    if let Some(ref client) = llm_client {
        let cache = LLM_CONNECTION_CACHE.get_or_init(|| Arc::new(AtomicBool::new(false)));
        let cache_clone = cache.clone();
        let client_clone = client.clone();
        tokio::spawn(async move {
            // Test connection once on startup
            if let Ok(connected) = client_clone.test_connection().await {
                cache_clone.store(connected, Ordering::Relaxed);
                if connected {
                    info!("LLM connection verified and cached");
                }
            }
            // Periodically refresh connection status (every 30 seconds)
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                if let Ok(connected) = client_clone.test_connection().await {
                    cache_clone.store(connected, Ordering::Relaxed);
                }
            }
        });
    }
    
    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let storage = storage.clone();
                    let metrics = metrics.clone();
                    let llm_client = llm_client.clone();
                    let thresholds = thresholds.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_client(stream, storage, metrics, llm_client, thresholds).await {
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

async fn handle_client(mut stream: UnixStream, storage: Storage, metrics: MetricsHandle, llm_client: Option<LlmClient>, thresholds: ThresholdsConfig) -> Result<()> {
    let mut buffer = vec![0u8; 8192];
    let n = stream.read(&mut buffer).await?;
    
    if n == 0 {
        return Ok(());
    }
    
    let request_str = String::from_utf8_lossy(&buffer[..n]);
    info!("IPC request: {}", request_str.trim());
    
    let response = match serde_json::from_str::<IpcRequest>(&request_str) {
        Ok(req) => handle_request(req, &storage, &metrics, llm_client.as_ref(), &thresholds).await,
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

async fn handle_request(req: IpcRequest, storage: &Storage, metrics: &MetricsHandle, llm_client: Option<&LlmClient>, thresholds: &ThresholdsConfig) -> IpcResponse {
    match req {
        IpcRequest::Status => handle_status(storage, metrics, llm_client, thresholds).await,
        IpcRequest::List { limit } => handle_list(storage, limit.unwrap_or(20)).await,
        IpcRequest::Show { event_id } => handle_show(storage, &event_id).await,
        IpcRequest::Analyze { event_id } => handle_analyze(storage, &event_id, llm_client).await,
    }
}

async fn handle_status(storage: &Storage, metrics: &MetricsHandle, llm_client: Option<&LlmClient>, thresholds: &ThresholdsConfig) -> IpcResponse {
    let uptime_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() - START_TIME.get().unwrap_or(&0);
    
    let (critical, warning, info) = storage.get_event_counts().await.unwrap_or((0, 0, 0));
    
    let metrics_guard = metrics.read().await;
    
    // Get LLM info if available (use cached connection status for performance)
    let llm_info = if let Some(client) = llm_client {
        let connected = LLM_CONNECTION_CACHE
            .get()
            .map(|cache| cache.load(Ordering::Relaxed))
            .unwrap_or(false);
        serde_json::json!({
            "available": connected,
            "model": client.model(),
            "url": client.base_url()
        })
    } else {
        serde_json::json!({
            "available": false,
            "model": null,
            "url": null
        })
    };
    
    IpcResponse {
        success: true,
        data: serde_json::json!({
            "status": "running",
            "uptime_seconds": uptime_secs,
            "collectors": {
                "cpu": "active",
                "memory": "active"
            },
            "metrics": {
                "cpu_usage": metrics_guard.cpu_usage,
                "memory_used_mb": metrics_guard.memory_used_mb,
                "memory_total_mb": metrics_guard.memory_total_mb,
                "memory_percent": metrics_guard.memory_percent
            },
            "events": {
                "critical": critical,
                "warning": warning,
                "info": info
            },
            "llm": llm_info,
            "thresholds": {
                "cpu_warning": thresholds.cpu_warning,
                "cpu_critical": thresholds.cpu_critical,
                "memory_warning": thresholds.memory_warning,
                "memory_critical": thresholds.memory_critical,
                "cpu_sustained_count": thresholds.cpu_sustained_count
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

async fn handle_analyze(storage: &Storage, event_id: &str, llm_client: Option<&LlmClient>) -> IpcResponse {
    // First get the event
    let event_result = storage.get_event_by_id(event_id).await;
    
    let event = match event_result {
        Ok(Some(e)) => e,
        Ok(None) => {
            return IpcResponse {
                success: false,
                data: serde_json::json!({"error": format!("Event {} not found", event_id)}),
            };
        }
        Err(e) => {
            return IpcResponse {
                success: false,
                data: serde_json::json!({"error": format!("Database error: {}", e)}),
            };
        }
    };
    
    // Check if LLM is available
    let llm_client = match llm_client {
        Some(client) => client,
        None => {
            return IpcResponse {
                success: false,
                data: serde_json::json!({"error": "LLM client not available. Make sure Ollama is running and configured."}),
            };
        }
    };
    
    // Parse event data - snapshot contains concatenated JSON bytes
    // Try to parse as single JSON first, if that fails, try to split
    let snapshot_str = String::from_utf8_lossy(&event.snapshot);
    let snapshot: serde_json::Value = serde_json::from_str(&snapshot_str)
        .unwrap_or_else(|_| serde_json::json!({}));
    
    // Extract entity and evidence from snapshot
    // The snapshot may have entity/evidence fields or be the entity itself
    let entity = snapshot.get("entity").cloned()
        .or_else(|| {
            // If no entity field, the snapshot itself might be the entity
            if snapshot.is_object() && (snapshot.get("cpu_usage").is_some() || 
                                        snapshot.get("memory_percent").is_some() ||
                                        snapshot.get("type").is_some()) {
                Some(snapshot.clone())
            } else {
                Some(serde_json::json!({}))
            }
        })
        .unwrap_or_else(|| serde_json::json!({}));
    
    let evidence = snapshot.get("evidence").cloned()
        .unwrap_or_else(|| serde_json::json!({
            "timestamp": event.ts,
            "threshold": "unknown"
        }));
    
    // Reconstruct Event struct for LLM
    let event_for_llm = Event {
        event_id: event.event_id.clone(),
        ts: event.ts.to_string(),
        severity: event.severity.clone(),
        r#type: event.type_.clone(),
        entity,
        evidence,
        suggestion: None,
        status: event.status.clone(),
    };
    
    // Get LLM analysis
    match llm_client.analyze_event(&event_for_llm).await {
        Ok(suggestion) => {
            IpcResponse {
                success: true,
                data: serde_json::json!({
                    "event_id": event.event_id,
                    "suggestion": suggestion,
                }),
            }
        }
        Err(e) => {
            IpcResponse {
                success: false,
                data: serde_json::json!({"error": format!("LLM analysis failed: {}", e)}),
            }
        }
    }
}
