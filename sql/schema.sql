PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;


CREATE TABLE IF NOT EXISTS events (
event_id TEXT PRIMARY KEY,
ts INTEGER,
severity TEXT,
type TEXT,
service_id TEXT,
fingerprint TEXT,
snapshot BLOB,
status TEXT
);


CREATE INDEX IF NOT EXISTS idx_events_ts ON events(ts);
CREATE INDEX IF NOT EXISTS idx_events_sev ON events(severity);
CREATE INDEX IF NOT EXISTS idx_events_service ON events(service_id);


CREATE TABLE IF NOT EXISTS grants (
id TEXT PRIMARY KEY,
service_id TEXT,
scopes TEXT,
expires_at INTEGER,
token TEXT
);


CREATE TABLE IF NOT EXISTS audits (
id INTEGER PRIMARY KEY AUTOINCREMENT,
ts INTEGER,
kind TEXT,
payload BLOB
);