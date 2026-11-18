use rusqlite::{params, Connection};
use sekonic_schema::error::SekonicError;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct OutboxItem {
    pub id: i64,
    pub idempotency_key: String,
    pub session_id: String,
    pub seq_no: u64,
    pub payload_json: String,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct QueueStats {
    pub pending: usize,
    pub synced: usize,
    pub failed: usize,
}

#[derive(Clone)]
pub struct WalOutboxQueue {
    conn: Arc<Mutex<Connection>>,
}

impl WalOutboxQueue {
    pub fn open<P: AsRef<Path>>(db_path: P) -> Result<Self, SekonicError> {
        let conn = Connection::open(db_path)
            .map_err(|e| SekonicError::Sync(format!("sqlite open: {e}")))?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             CREATE TABLE IF NOT EXISTS cached_exams (
                 form_id TEXT PRIMARY KEY,
                 schema_json TEXT NOT NULL,
                 roster_json TEXT NOT NULL,
                 updated_at_ms INTEGER NOT NULL
             );
             CREATE TABLE IF NOT EXISTS scanned_sheets_outbox (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 idempotency_key TEXT NOT NULL UNIQUE,
                 session_id TEXT NOT NULL,
                 seq_no INTEGER NOT NULL,
                 payload_json TEXT NOT NULL,
                 sync_status TEXT NOT NULL DEFAULT 'pending',
                 created_at_ms INTEGER NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_outbox_status ON scanned_sheets_outbox(sync_status, id);",
        )
        .map_err(|e| SekonicError::Sync(format!("sqlite schema init: {e}")))?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn enqueue(
        &self,
        session_id: &str,
        seq_no: u64,
        payload_json: &str,
        created_at_ms: u64,
    ) -> Result<(), SekonicError> {
        let key = format!("{session_id}:{seq_no}");
        let guard = self
            .conn
            .lock()
            .map_err(|_| SekonicError::Sync("sqlite mutex poisoned".into()))?;
        guard
            .execute(
                "INSERT OR IGNORE INTO scanned_sheets_outbox
                 (idempotency_key, session_id, seq_no, payload_json, sync_status, created_at_ms)
                 VALUES (?1, ?2, ?3, ?4, 'pending', ?5)",
                params![key, session_id, seq_no as i64, payload_json, created_at_ms as i64],
            )
            .map_err(|e| SekonicError::Sync(format!("sqlite enqueue: {e}")))?;
        Ok(())
    }

    pub fn fetch_pending(&self, limit: usize) -> Result<Vec<OutboxItem>, SekonicError> {
        let guard = self
            .conn
            .lock()
            .map_err(|_| SekonicError::Sync("sqlite mutex poisoned".into()))?;
        let mut stmt = guard
            .prepare(
                "SELECT id, idempotency_key, session_id, seq_no, payload_json
                 FROM scanned_sheets_outbox
                 WHERE sync_status = 'pending'
                 ORDER BY id ASC
                 LIMIT ?1",
            )
            .map_err(|e| SekonicError::Sync(format!("sqlite prepare: {e}")))?;

        let rows = stmt
            .query_map(params![limit as i64], |r| {
                Ok(OutboxItem {
                    id: r.get(0)?,
                    idempotency_key: r.get(1)?,
                    session_id: r.get(2)?,
                    seq_no: r.get::<_, i64>(3)? as u64,
                    payload_json: r.get(4)?,
                })
            })
            .map_err(|e| SekonicError::Sync(format!("sqlite query: {e}")))?;

        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| SekonicError::Sync(format!("sqlite row: {e}")))?);
        }
        Ok(out)
    }

    pub fn mark_synced(&self, id: i64) -> Result<(), SekonicError> {
        let guard = self
            .conn
            .lock()
            .map_err(|_| SekonicError::Sync("sqlite mutex poisoned".into()))?;
        guard
            .execute(
                "UPDATE scanned_sheets_outbox SET sync_status = 'synced' WHERE id = ?1",
                params![id],
            )
            .map_err(|e| SekonicError::Sync(format!("sqlite mark_synced: {e}")))?;
        Ok(())
    }

    pub fn stats(&self) -> Result<QueueStats, SekonicError> {
        let guard = self
            .conn
            .lock()
            .map_err(|_| SekonicError::Sync("sqlite mutex poisoned".into()))?;
        let mut stmt = guard
            .prepare("SELECT sync_status, COUNT(*) FROM scanned_sheets_outbox GROUP BY sync_status")
            .map_err(|e| SekonicError::Sync(format!("sqlite stats: {e}")))?;
        let mut s = QueueStats::default();
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
            .map_err(|e| SekonicError::Sync(format!("sqlite stats query: {e}")))?;
        for row in rows {
            if let Ok((st, cnt)) = row {
                match st.as_str() {
                    "pending" => s.pending = cnt as usize,
                    "synced" => s.synced = cnt as usize,
                    "failed" => s.failed = cnt as usize,
                    _ => {}
                }
            }
        }
        Ok(s)
    }
}
