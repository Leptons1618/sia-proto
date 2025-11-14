use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use common::Event;
use anyhow::Result;
use log::{info, warn};

#[derive(Clone)]
pub struct LlmClient {
    client: Client,
    base_url: String,
    model: String,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

impl LlmClient {
    pub fn new(base_url: String, model: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap(),
            base_url,
            model,
        }
    }
    
    pub async fn test_connection(&self) -> Result<bool> {
        let url = format!("{}/api/tags", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                info!("Successfully connected to Ollama at {}", self.base_url);
                Ok(true)
            }
            Ok(resp) => {
                warn!("Ollama returned status {}", resp.status());
                Ok(false)
            }
            Err(e) => {
                warn!("Cannot connect to Ollama: {}", e);
                Ok(false)
            }
        }
    }
    
    pub async fn analyze_event(&self, event: &Event) -> Result<Value> {
        let prompt = self.create_prompt(event);
        
        let request = OllamaRequest {
            model: self.model.clone(),
            prompt,
            stream: false,
        };
        
        let url = format!("{}/api/generate", self.base_url);
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            anyhow::bail!("Ollama request failed with status {}", response.status());
        }
        
        let ollama_response: OllamaResponse = response.json().await?;
        
        // Parse the response into structured suggestion
        let suggestion = json!({
            "analysis": ollama_response.response.trim(),
            "source": "ollama",
            "model": self.model,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        
        Ok(suggestion)
    }
    
    fn create_prompt(&self, event: &Event) -> String {
        format!(
            r#"You are a system administrator AI assistant analyzing a system event.

Event Details:
- Type: {}
- Severity: {}
- Timestamp: {}
- Data: {}
- Evidence: {}

Please provide:
1. Brief analysis of what caused this issue
2. Immediate recommended actions (2-3 steps)
3. Preventive measures for the future

Keep your response concise and actionable (max 200 words)."#,
            event.r#type,
            event.severity,
            event.ts,
            serde_json::to_string_pretty(&event.entity).unwrap_or_default(),
            serde_json::to_string_pretty(&event.evidence).unwrap_or_default()
        )
    }
}
