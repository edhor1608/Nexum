use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};
use thiserror::Error;

use crate::capsule::{Capsule, CapsuleMode};

#[derive(Debug)]
pub struct CapsuleStore {
    conn: Connection,
}

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("db: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("yaml: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

impl CapsuleStore {
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS capsules (
                capsule_id TEXT PRIMARY KEY,
                slug TEXT NOT NULL,
                display_name TEXT NOT NULL,
                mode TEXT NOT NULL,
                workspace INTEGER NOT NULL
            );
            ",
        )?;

        Ok(Self { conn })
    }

    pub fn upsert(&mut self, capsule: Capsule) -> Result<(), StoreError> {
        self.conn.execute(
            "
            INSERT INTO capsules (capsule_id, slug, display_name, mode, workspace)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(capsule_id) DO UPDATE SET
                slug = excluded.slug,
                display_name = excluded.display_name,
                mode = excluded.mode,
                workspace = excluded.workspace
            ",
            params![
                capsule.capsule_id,
                capsule.slug,
                capsule.display_name,
                mode_to_str(capsule.mode),
                capsule.workspace
            ],
        )?;
        Ok(())
    }

    pub fn get(&self, capsule_id: &str) -> Result<Option<Capsule>, StoreError> {
        self.conn
            .query_row(
                "SELECT capsule_id, slug, display_name, mode, workspace FROM capsules WHERE capsule_id = ?1",
                params![capsule_id],
                row_to_capsule,
            )
            .optional()
            .map_err(StoreError::from)
    }

    pub fn list(&self) -> Result<Vec<Capsule>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT capsule_id, slug, display_name, mode, workspace FROM capsules ORDER BY capsule_id ASC",
        )?;

        let rows = stmt
            .query_map([], row_to_capsule)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    pub fn export_yaml(&self) -> Result<String, StoreError> {
        let capsules = self.list()?;
        Ok(serde_yaml::to_string(&capsules)?)
    }
}

fn row_to_capsule(row: &rusqlite::Row<'_>) -> rusqlite::Result<Capsule> {
    let mode: String = row.get(3)?;
    Ok(Capsule {
        capsule_id: row.get(0)?,
        slug: row.get(1)?,
        display_name: row.get(2)?,
        mode: parse_mode(&mode),
        workspace: row.get(4)?,
    })
}

fn parse_mode(value: &str) -> CapsuleMode {
    match value {
        "isolated_nix_shell" => CapsuleMode::IsolatedNixShell,
        _ => CapsuleMode::HostDefault,
    }
}

fn mode_to_str(mode: CapsuleMode) -> &'static str {
    match mode {
        CapsuleMode::HostDefault => "host_default",
        CapsuleMode::IsolatedNixShell => "isolated_nix_shell",
    }
}
