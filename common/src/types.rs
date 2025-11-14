use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Service {
pub id: String,
pub name: String,
pub discovery: Option<serde_json::Value>,
pub default_scopes: Vec<String>,
pub requested_scopes: Vec<String>,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Grant {
pub id: String,
pub service_id: String,
pub scopes: Vec<String>,
pub expires_at: String,
pub token: String,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
pub event_id: String,
pub ts: String,
pub severity: String,
pub r#type: String,
pub entity: serde_json::Value,
pub evidence: serde_json::Value,
pub suggestion: Option<serde_json::Value>,
pub status: String,
}