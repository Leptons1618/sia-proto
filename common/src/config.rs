use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub agent: AgentConfig,
    pub ipc: IpcConfig,
    pub llm: LlmConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub memory_budget: usize,
    pub disk_quota: usize,
    pub cpu_interval: u64,
    pub proc_interval: u64,
    pub event_ring_capacity: usize,
    #[serde(default = "default_thresholds")]
    pub thresholds: ThresholdsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdsConfig {
    #[serde(default = "default_cpu_warning")]
    pub cpu_warning: f32,
    #[serde(default = "default_cpu_critical")]
    pub cpu_critical: f32,
    #[serde(default = "default_memory_warning")]
    pub memory_warning: f32,
    #[serde(default = "default_memory_critical")]
    pub memory_critical: f32,
    #[serde(default = "default_cpu_sustained_count")]
    pub cpu_sustained_count: u32,
}

fn default_thresholds() -> ThresholdsConfig {
    ThresholdsConfig {
        cpu_warning: 80.0,
        cpu_critical: 95.0,
        memory_warning: 85.0,
        memory_critical: 95.0,
        cpu_sustained_count: 2,
    }
}

fn default_cpu_warning() -> f32 { 80.0 }
fn default_cpu_critical() -> f32 { 95.0 }
fn default_memory_warning() -> f32 { 85.0 }
fn default_memory_critical() -> f32 { 95.0 }
fn default_cpu_sustained_count() -> u32 { 2 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcConfig {
    pub socket_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub ollama_url: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub db_path: String,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn default_path() -> String {
        std::env::var("SIA_CONFIG")
            .unwrap_or_else(|_| "./config/default.toml".to_string())
    }
}
