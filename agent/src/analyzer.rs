use tokio::sync::mpsc;
use common::Event;
use crate::storage::Storage;
use crate::llm::LlmClient;
use log::{info, error};

pub async fn start_analyzer(
    mut rx: mpsc::Receiver<Event>,
    storage: Storage,
    llm_client: Option<LlmClient>,
) -> anyhow::Result<()> {
    info!("Starting event analyzer");
    
    tokio::spawn(async move {
        while let Some(mut event) = rx.recv().await {
            info!("Analyzing event: {} ({})", event.event_id, event.severity);
            
            // For critical events, get LLM suggestion
            if event.severity == "CRITICAL" && llm_client.is_some() {
                if let Some(ref client) = llm_client {
                    match client.analyze_event(&event).await {
                        Ok(suggestion) => {
                            event.suggestion = Some(suggestion);
                            info!("LLM suggestion added to event {}", event.event_id);
                        }
                        Err(e) => {
                            error!("LLM analysis failed for {}: {}", event.event_id, e);
                        }
                    }
                }
            }
            
            // Store event in database
            if let Err(e) = store_event(&storage, &event).await {
                error!("Failed to store event {}: {}", event.event_id, e);
            } else {
                info!("Event {} stored successfully", event.event_id);
            }
        }
        
        info!("Analyzer stopped");
    });
    
    Ok(())
}

async fn store_event(storage: &Storage, event: &Event) -> anyhow::Result<()> {
    let ts = chrono::DateTime::parse_from_rfc3339(&event.ts)?
        .timestamp();
    
    let entity_json = serde_json::to_vec(&event.entity)?;
    let evidence_json = serde_json::to_vec(&event.evidence)?;
    
    let mut snapshot = entity_json.clone();
    snapshot.extend_from_slice(&evidence_json);
    
    if let Some(ref suggestion) = event.suggestion {
        let suggestion_json = serde_json::to_vec(suggestion)?;
        snapshot.extend_from_slice(&suggestion_json);
    }
    
    storage.insert_event(
        &event.event_id,
        ts,
        &event.severity,
        &event.r#type,
        "system",
        &snapshot,
    ).await?;
    
    Ok(())
}
