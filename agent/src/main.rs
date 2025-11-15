use anyhow::Result;
use tokio::signal;
use tokio::sync::mpsc;
use log::info;

mod collectors;
mod analyzer;
mod storage;
mod ipc;
mod llm;

use collectors::start_collectors;
use analyzer::start_analyzer;
use ipc::start_ipc_server;
use storage::Storage;
use llm::LlmClient;
use common::Config;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    info!("Starting SIA agent (MVP prototype)");
    
    // Load configuration
    let config_path = Config::default_path();
    let config = Config::load(&config_path)?;
    info!("Config loaded from {}", config_path);
    
    // Initialize storage
    let storage = Storage::new(&config.storage.db_path).await?;
    info!("Storage initialized at {}", config.storage.db_path);
    
    // Initialize LLM client (optional)
    let llm_client = LlmClient::new(
        config.llm.ollama_url.clone(),
        config.llm.model.clone(),
    );
    
    let llm_available = if let Ok(connected) = llm_client.test_connection().await {
        if connected {
            info!("LLM client ready ({})", config.llm.ollama_url);
            Some(llm_client)
        } else {
            info!("LLM not available, continuing without AI suggestions");
            None
        }
    } else {
        info!("LLM connection test failed, continuing without AI suggestions");
        None
    };
    
    // Create event channel
    let (tx, rx) = mpsc::channel(config.agent.event_ring_capacity);
    
    // Start collectors
    start_collectors(tx, config.agent.cpu_interval).await?;
    info!("Collectors started");
    
    // Start analyzer
    start_analyzer(rx, storage.clone(), llm_available).await?;
    info!("Analyzer started");
    
    // Start IPC server
    start_ipc_server(storage.clone(), config.ipc.socket_path.clone()).await?;
    info!("IPC server started on {}", config.ipc.socket_path);
    
    info!("SIA agent is running");
    
    // Wait for ctrl-c
    signal::ctrl_c().await?;
    info!("Shutting down");
    Ok(())
}