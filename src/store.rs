use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};
use thiserror::Error;

use crate::capsule::{Capsule, CapsuleMode, CapsuleState, parse_state, state_to_str};

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
                repo_path TEXT NOT NULL DEFAULT '',
                mode TEXT NOT NULL,
                state TEXT NOT NULL DEFAULT 'ready',
                workspace INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS capsule_ports (
                capsule_id TEXT NOT NULL,
                port INTEGER NOT NULL PRIMARY KEY
            );
            ",
        )?;
        add_state_column_if_missing(&conn)?;
        add_repo_path_column_if_missing(&conn)?;

        Ok(Self { conn })
    }

    pub fn upsert(&mut self, capsule: Capsule) -> Result<(), StoreError> {
        self.conn.execute(
            "
            INSERT INTO capsules (capsule_id, slug, display_name, repo_path, mode, state, workspace)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(capsule_id) DO UPDATE SET
                slug = excluded.slug,
                display_name = excluded.display_name,
                repo_path = excluded.repo_path,
                mode = excluded.mode,
                state = excluded.state,
                workspace = excluded.workspace
            ",
            params![
                capsule.capsule_id,
                capsule.slug,
                capsule.display_name,
                capsule.repo_path,
                mode_to_str(capsule.mode),
                state_to_str(capsule.state),
                capsule.workspace
            ],
        )?;
        Ok(())
    }

    pub fn get(&self, capsule_id: &str) -> Result<Option<Capsule>, StoreError> {
        self.conn
            .query_row(
                "SELECT capsule_id, slug, display_name, repo_path, mode, state, workspace FROM capsules WHERE capsule_id = ?1",
                params![capsule_id],
                row_to_capsule,
            )
            .optional()
            .map_err(StoreError::from)
    }

    pub fn list(&self) -> Result<Vec<Capsule>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT capsule_id, slug, display_name, repo_path, mode, state, workspace FROM capsules ORDER BY capsule_id ASC",
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

    pub fn transition_state(
        &mut self,
        capsule_id: &str,
        state: CapsuleState,
    ) -> Result<(), StoreError> {
        self.conn.execute(
            "UPDATE capsules SET state = ?1 WHERE capsule_id = ?2",
            params![state_to_str(state), capsule_id],
        )?;
        Ok(())
    }

    pub fn rename_display_name(
        &mut self,
        capsule_id: &str,
        display_name: &str,
    ) -> Result<(), StoreError> {
        self.conn.execute(
            "UPDATE capsules SET display_name = ?1 WHERE capsule_id = ?2",
            params![display_name, capsule_id],
        )?;
        Ok(())
    }

    pub fn set_repo_path(&mut self, capsule_id: &str, repo_path: &str) -> Result<(), StoreError> {
        self.conn.execute(
            "UPDATE capsules SET repo_path = ?1 WHERE capsule_id = ?2",
            params![repo_path, capsule_id],
        )?;
        Ok(())
    }

    pub fn list_ports(&self, capsule_id: &str) -> Result<Vec<u16>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT port FROM capsule_ports WHERE capsule_id = ?1 ORDER BY port ASC")?;
        let ports = stmt
            .query_map(params![capsule_id], |row| row.get::<_, u16>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ports)
    }

    pub fn allocate_port(
        &mut self,
        capsule_id: &str,
        start: u16,
        end: u16,
    ) -> Result<Option<u16>, StoreError> {
        if let Some(existing) = self
            .conn
            .query_row(
                "SELECT port FROM capsule_ports WHERE capsule_id = ?1 ORDER BY port ASC LIMIT 1",
                params![capsule_id],
                |row| row.get::<_, u16>(0),
            )
            .optional()?
        {
            return Ok(Some(existing));
        }

        for candidate in start..=end {
            let inserted = self.conn.execute(
                "INSERT OR IGNORE INTO capsule_ports (capsule_id, port) VALUES (?1, ?2)",
                params![capsule_id, candidate],
            )?;
            if inserted == 1 {
                return Ok(Some(candidate));
            }
        }

        Ok(None)
    }

    pub fn release_ports(&mut self, capsule_id: &str) -> Result<u32, StoreError> {
        let released = self.conn.execute(
            "DELETE FROM capsule_ports WHERE capsule_id = ?1",
            params![capsule_id],
        )?;
        Ok(released as u32)
    }
}

fn row_to_capsule(row: &rusqlite::Row<'_>) -> rusqlite::Result<Capsule> {
    let mode: String = row.get(4)?;
    let state: String = row.get(5)?;
    Ok(Capsule {
        capsule_id: row.get(0)?,
        slug: row.get(1)?,
        display_name: row.get(2)?,
        repo_path: row.get(3)?,
        mode: parse_mode(&mode),
        state: parse_state(&state).unwrap_or(CapsuleState::Ready),
        workspace: row.get(6)?,
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

fn add_state_column_if_missing(conn: &Connection) -> Result<(), StoreError> {
    match conn.execute(
        "ALTER TABLE capsules ADD COLUMN state TEXT NOT NULL DEFAULT 'ready'",
        [],
    ) {
        Ok(_) => Ok(()),
        Err(rusqlite::Error::SqliteFailure(_, Some(message)))
            if message.contains("duplicate column name") =>
        {
            Ok(())
        }
        Err(error) => Err(StoreError::Db(error)),
    }
}

fn add_repo_path_column_if_missing(conn: &Connection) -> Result<(), StoreError> {
    match conn.execute(
        "ALTER TABLE capsules ADD COLUMN repo_path TEXT NOT NULL DEFAULT ''",
        [],
    ) {
        Ok(_) => Ok(()),
        Err(rusqlite::Error::SqliteFailure(_, Some(message)))
            if message.contains("duplicate column name") =>
        {
            Ok(())
        }
        Err(error) => Err(StoreError::Db(error)),
    }
}
