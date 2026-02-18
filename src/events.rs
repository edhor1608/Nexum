use std::path::Path;

use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeEvent {
    pub capsule_id: String,
    pub component: String,
    pub level: String,
    pub message: String,
    pub ts_unix_ms: u64,
}

#[derive(Debug, Error)]
pub enum EventError {
    #[error("db: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct EventStore {
    conn: Connection,
}

const DEFAULT_LIST_LIMIT: u32 = 500;

impl EventStore {
    pub fn open(path: &Path) -> Result<Self, EventError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS runtime_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                capsule_id TEXT NOT NULL,
                component TEXT NOT NULL,
                level TEXT NOT NULL,
                message TEXT NOT NULL,
                ts_unix_ms INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_runtime_events_capsule_id ON runtime_events(capsule_id);
            ",
        )?;

        Ok(Self { conn })
    }

    pub fn append(&mut self, event: RuntimeEvent) -> Result<(), EventError> {
        self.conn.execute(
            "
            INSERT INTO runtime_events (capsule_id, component, level, message, ts_unix_ms)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ",
            params![
                event.capsule_id,
                event.component,
                event.level,
                event.message,
                event.ts_unix_ms,
            ],
        )?;

        Ok(())
    }

    pub fn list_for_capsule(&self, capsule_id: &str) -> Result<Vec<RuntimeEvent>, EventError> {
        self.list_for_capsule_paginated(capsule_id, DEFAULT_LIST_LIMIT, 0)
    }

    pub fn list_for_capsule_paginated(
        &self,
        capsule_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<RuntimeEvent>, EventError> {
        let mut stmt = self.conn.prepare(
            "
            SELECT capsule_id, component, level, message, ts_unix_ms
            FROM runtime_events
            WHERE capsule_id = ?1
            ORDER BY id ASC
            LIMIT ?2
            OFFSET ?3
            ",
        )?;

        let rows = stmt
            .query_map(params![capsule_id, limit, offset], |row| {
                Ok(RuntimeEvent {
                    capsule_id: row.get(0)?,
                    component: row.get(1)?,
                    level: row.get(2)?,
                    message: row.get(3)?,
                    ts_unix_ms: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rows)
    }
}
