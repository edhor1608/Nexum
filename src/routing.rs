use std::{
    collections::BTreeMap,
    os::unix::fs::FileTypeExt,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
    sync::oneshot,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteEntry {
    pub capsule_id: String,
    pub domain: String,
    pub upstream: String,
    pub tls_mode: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum RouteCommand {
    Health,
    Register {
        capsule_id: String,
        domain: String,
        upstream: String,
    },
    Resolve {
        domain: String,
    },
    Remove {
        domain: String,
    },
    List,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RouteOutcome {
    Health { status: String },
    Registered { domain: String },
    Resolved { route: Option<RouteEntry> },
    Removed { removed: bool },
    Listed { routes: Vec<RouteEntry> },
    Error { code: String, message: String },
}

#[derive(Debug, Default, Clone)]
pub struct RouterState {
    routes: BTreeMap<String, RouteEntry>,
}

impl RouterState {
    pub fn handle(&mut self, command: RouteCommand) -> RouteOutcome {
        match command {
            RouteCommand::Health => RouteOutcome::Health {
                status: "ok".to_string(),
            },
            RouteCommand::Register {
                capsule_id,
                domain,
                upstream,
            } => {
                if let Some(existing) = self.routes.get(&domain) {
                    if existing.capsule_id != capsule_id {
                        return RouteOutcome::Error {
                            code: "domain_conflict".to_string(),
                            message: format!(
                                "domain '{}' already claimed by {}",
                                domain, existing.capsule_id
                            ),
                        };
                    }
                }

                self.routes.insert(
                    domain.clone(),
                    RouteEntry {
                        capsule_id,
                        domain: domain.clone(),
                        upstream,
                        tls_mode: "self_signed".to_string(),
                    },
                );

                RouteOutcome::Registered { domain }
            }
            RouteCommand::Resolve { domain } => RouteOutcome::Resolved {
                route: self.routes.get(&domain).cloned(),
            },
            RouteCommand::Remove { domain } => RouteOutcome::Removed {
                removed: self.routes.remove(&domain).is_some(),
            },
            RouteCommand::List => RouteOutcome::Listed {
                routes: self.routes.values().cloned().collect(),
            },
        }
    }
}

#[derive(Debug, Error)]
pub enum RoutingError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

pub async fn send_command(
    socket_path: &Path,
    command: RouteCommand,
) -> Result<RouteOutcome, RoutingError> {
    let mut stream = UnixStream::connect(socket_path).await?;

    let payload = serde_json::to_string(&command)?;
    stream.write_all(payload.as_bytes()).await?;
    stream.write_all(b"\n").await?;
    stream.flush().await?;

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader.read_line(&mut response).await?;

    Ok(serde_json::from_str(response.trim_end())?)
}

pub async fn serve_unix_socket(
    socket_path: &Path,
    mut shutdown_rx: oneshot::Receiver<()>,
) -> Result<(), RoutingError> {
    if socket_path.exists() {
        let meta = std::fs::symlink_metadata(socket_path)?;
        if meta.file_type().is_socket() {
            std::fs::remove_file(socket_path)?;
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!(
                    "socket path exists and is not a Unix socket: {}",
                    socket_path.display()
                ),
            )
            .into());
        }
    }

    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let listener = UnixListener::bind(socket_path)?;
    let state = Arc::new(Mutex::new(RouterState::default()));

    loop {
        tokio::select! {
            _ = &mut shutdown_rx => {
                break;
            }
            accepted = listener.accept() => {
                let (stream, _) = accepted?;
                let state = Arc::clone(&state);
                tokio::spawn(async move {
                    let _ = handle_connection(stream, state).await;
                });
            }
        }
    }

    let _ = std::fs::remove_file(socket_path);
    Ok(())
}

async fn handle_connection(
    stream: UnixStream,
    state: Arc<Mutex<RouterState>>,
) -> Result<(), RoutingError> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        if reader.read_line(&mut line).await? == 0 {
            break;
        }

        let command = serde_json::from_str::<RouteCommand>(line.trim_end());
        let outcome = match command {
            Ok(command) => state.lock().expect("router mutex poisoned").handle(command),
            Err(error) => RouteOutcome::Error {
                code: "invalid_command".to_string(),
                message: error.to_string(),
            },
        };

        let encoded = serde_json::to_string(&outcome)?;
        writer.write_all(encoded.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
    }

    Ok(())
}

pub fn default_socket_path() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(|dir| PathBuf::from(dir).join("nexumd.sock"))
        .unwrap_or_else(|| std::env::temp_dir().join("nexumd.sock"))
}
