use sqlx::SqlitePool;
use anyhow::Result;


#[derive(Clone)]
pub struct Storage { pool: SqlitePool }

#[derive(Debug)]
pub struct StoredEvent {
    pub event_id: String,
    pub ts: i64,
    pub severity: String,
    pub type_: String,
    pub service_id: String,
    pub snapshot: Vec<u8>,
    pub status: String,
}


impl Storage {
pub async fn new(path: &str) -> Result<Self> {
let url = format!("sqlite://{}", path);
let pool = SqlitePool::connect(&url).await?;
// run migrations / schema
sqlx::query(include_str!("../../sql/schema.sql")).execute(&pool).await?;
Ok(Self { pool })
}


// simple insert stub
pub async fn insert_event(&self, id: &str, ts: i64, severity: &str, ty: &str, service: &str, snapshot: &[u8]) -> Result<()> {
sqlx::query("INSERT INTO events(event_id, ts, severity, type, service_id, snapshot, status) VALUES (?, ?, ?, ?, ?, ?, 'open')")
.bind(id)
.bind(ts)
.bind(severity)
.bind(ty)
.bind(service)
.bind(snapshot)
.execute(&self.pool).await?;
Ok(())
}

pub async fn get_recent_events(&self, limit: i32) -> Result<Vec<StoredEvent>> {
    let rows = sqlx::query_as::<_, (String, i64, String, String, String, Vec<u8>, String)>(
        "SELECT event_id, ts, severity, type, service_id, snapshot, status FROM events ORDER BY ts DESC LIMIT ?"
    )
    .bind(limit)
    .fetch_all(&self.pool)
    .await?;
    
    Ok(rows.into_iter().map(|(event_id, ts, severity, type_, service_id, snapshot, status)| {
        StoredEvent { event_id, ts, severity, type_, service_id, snapshot, status }
    }).collect())
}

pub async fn get_event_by_id(&self, id: &str) -> Result<Option<StoredEvent>> {
    let row = sqlx::query_as::<_, (String, i64, String, String, String, Vec<u8>, String)>(
        "SELECT event_id, ts, severity, type, service_id, snapshot, status FROM events WHERE event_id = ?"
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;
    
    Ok(row.map(|(event_id, ts, severity, type_, service_id, snapshot, status)| {
        StoredEvent { event_id, ts, severity, type_, service_id, snapshot, status }
    }))
}

pub async fn get_event_counts(&self) -> Result<(i64, i64, i64)> {
    let critical: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM events WHERE severity = 'CRITICAL' AND status = 'open'"
    )
    .fetch_one(&self.pool)
    .await?;
    
    let warning: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM events WHERE severity = 'WARNING' AND status = 'open'"
    )
    .fetch_one(&self.pool)
    .await?;
    
    let info: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM events WHERE severity = 'INFO' AND status = 'open'"
    )
    .fetch_one(&self.pool)
    .await?;
    
    Ok((critical.0, warning.0, info.0))
}
}