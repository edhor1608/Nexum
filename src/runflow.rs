use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    capsule::{Capsule, CapsuleMode, CapsuleState},
    events::{EventError, EventStore, RuntimeEvent},
    identity::browser_launch_command,
    isolation::{IsolationInput, select_capsule_mode},
    restore::{RestoreRequest, RestoreSurfaces, SignalType, build_restore_plan},
    routing::{RouteCommand, RouteOutcome, RouterState, send_command},
    runtime_meta::{capsule_runtime_env, terminal_process_label},
    shell::{build_niri_shell_plan, render_shell_script},
    store::StoreError,
    tls::{TlsError, ensure_self_signed_cert},
};

#[derive(Debug, Clone)]
pub struct RestoreRunInput {
    pub capsule_id: String,
    pub display_name: String,
    pub workspace: u16,
    pub signal: SignalType,
    pub terminal_cmd: String,
    pub editor_target: String,
    pub browser_url: String,
    pub route_upstream: String,
    pub routing_socket: Option<PathBuf>,
    pub identity_collision: bool,
    pub high_risk_secret_workflow: bool,
    pub force_isolated_mode: bool,
    pub capsule_db: Option<PathBuf>,
    pub tls_dir: PathBuf,
    pub events_db: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreRunSummary {
    pub capsule_id: String,
    pub domain: String,
    pub run_mode: String,
    pub degraded: bool,
    pub degraded_reason: Option<String>,
    pub target_budget_ms: u64,
    pub shell_script: String,
    pub tls_fingerprint_sha256: String,
    pub events_written: u32,
}

#[derive(Debug, Error)]
pub enum RunFlowError {
    #[error("tls: {0}")]
    Tls(#[from] TlsError),
    #[error("events: {0}")]
    Events(#[from] EventError),
    #[error("store: {0}")]
    Store(#[from] StoreError),
    #[error("routing failed: {0}")]
    Routing(String),
}

pub fn run_restore_flow(input: RestoreRunInput) -> Result<RestoreRunSummary, RunFlowError> {
    transition_capsule_state(
        input.capsule_db.as_ref(),
        &input.capsule_id,
        CapsuleState::Restoring,
    )?;

    let mode = select_capsule_mode(&IsolationInput {
        identity_collision_detected: input.identity_collision,
        high_risk_secret_workflow: input.high_risk_secret_workflow,
        force_isolated_mode: input.force_isolated_mode,
    });

    let capsule = Capsule::new(
        &input.capsule_id,
        &input.display_name,
        mode,
        input.workspace,
    );

    let request = RestoreRequest {
        capsule: capsule.clone(),
        signal: input.signal.clone(),
        surfaces: RestoreSurfaces {
            terminal_cmd: input.terminal_cmd.clone(),
            editor_target: input.editor_target.clone(),
            browser_url: input.browser_url.clone(),
        },
    };

    let restore = build_restore_plan(&request);

    let tls = ensure_self_signed_cert(&input.tls_dir, &capsule.domain(), 30)?;

    let route_status = match ensure_route(&capsule, &input) {
        Ok(status) => status,
        Err(error) => {
            transition_capsule_state(
                input.capsule_db.as_ref(),
                &input.capsule_id,
                CapsuleState::Degraded,
            )?;
            return Err(error);
        }
    };

    let shell_script = apply_browser_launch_policy(
        render_shell_script(&build_niri_shell_plan(&restore)),
        &input.browser_url,
        &browser_launch_command(
            &input.browser_url,
            &input.capsule_id,
            input.identity_collision,
        ),
    );
    let shell_script = apply_runtime_metadata(shell_script, &capsule);

    let (degraded, degraded_reason, routing_level, routing_message) = match route_status {
        RouteEnsureStatus::Ready => (
            false,
            None,
            "info".to_string(),
            format!("route ensured for {}", capsule.domain()),
        ),
        RouteEnsureStatus::Degraded(reason) => (
            true,
            Some(reason.clone()),
            "warn".to_string(),
            format!("degraded route for {}: {}", capsule.domain(), reason),
        ),
    };

    let mut events = EventStore::open(&input.events_db)?;
    events.append(RuntimeEvent {
        capsule_id: capsule.capsule_id.clone(),
        component: "runflow".into(),
        level: "info".into(),
        message: "restore start".into(),
        ts_unix_ms: now_unix_ms(),
    })?;
    events.append(RuntimeEvent {
        capsule_id: capsule.capsule_id.clone(),
        component: "routing".into(),
        level: routing_level,
        message: routing_message,
        ts_unix_ms: now_unix_ms(),
    })?;
    events.append(RuntimeEvent {
        capsule_id: capsule.capsule_id.clone(),
        component: "runflow".into(),
        level: "info".into(),
        message: "restore plan ready".into(),
        ts_unix_ms: now_unix_ms(),
    })?;

    transition_capsule_state(
        input.capsule_db.as_ref(),
        &input.capsule_id,
        if degraded {
            CapsuleState::Degraded
        } else {
            CapsuleState::Ready
        },
    )?;

    Ok(RestoreRunSummary {
        capsule_id: capsule.capsule_id.clone(),
        domain: capsule.domain(),
        run_mode: mode_to_str(capsule.mode).to_string(),
        degraded,
        degraded_reason,
        target_budget_ms: restore.target_budget_ms,
        shell_script,
        tls_fingerprint_sha256: tls.fingerprint_sha256,
        events_written: 3,
    })
}

fn transition_capsule_state(
    capsule_db: Option<&PathBuf>,
    capsule_id: &str,
    state: CapsuleState,
) -> Result<(), RunFlowError> {
    if let Some(path) = capsule_db {
        let mut store = crate::store::CapsuleStore::open(path)?;
        store.transition_state(capsule_id, state)?;
    }
    Ok(())
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before epoch")
        .as_millis() as u64
}

#[derive(Debug, Clone)]
enum RouteEnsureStatus {
    Ready,
    Degraded(String),
}

fn ensure_route(
    capsule: &Capsule,
    input: &RestoreRunInput,
) -> Result<RouteEnsureStatus, RunFlowError> {
    if let Some(socket) = &input.routing_socket {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .map_err(|error| RunFlowError::Routing(error.to_string()))?;

        let outcome = match runtime.block_on(send_command(
            socket,
            RouteCommand::Register {
                capsule_id: capsule.capsule_id.clone(),
                domain: capsule.domain(),
                upstream: input.route_upstream.clone(),
            },
        )) {
            Ok(outcome) => outcome,
            Err(error) => {
                return Ok(RouteEnsureStatus::Degraded(format!(
                    "route_unavailable: {}",
                    error
                )));
            }
        };

        return match outcome {
            RouteOutcome::Registered { .. } => Ok(RouteEnsureStatus::Ready),
            RouteOutcome::Error { code, message } => {
                if code == "domain_conflict" {
                    return Err(RunFlowError::Routing(format!("{code}: {message}")));
                }
                Ok(RouteEnsureStatus::Degraded(format!(
                    "route_unavailable: {code}: {message}"
                )))
            }
            other => Ok(RouteEnsureStatus::Degraded(format!(
                "route_unavailable: unexpected daemon outcome: {:?}",
                other
            ))),
        };
    }

    let mut router = RouterState::default();
    let route = router.handle(RouteCommand::Register {
        capsule_id: capsule.capsule_id.clone(),
        domain: capsule.domain(),
        upstream: input.route_upstream.clone(),
    });
    if let RouteOutcome::Error { message, .. } = route {
        return Err(RunFlowError::Routing(message));
    }

    Ok(RouteEnsureStatus::Ready)
}

fn apply_browser_launch_policy(script: String, browser_url: &str, launch_cmd: &str) -> String {
    let default = format!("xdg-open {browser_url}");
    script.replacen(&default, launch_cmd, 1)
}

fn mode_to_str(mode: CapsuleMode) -> &'static str {
    match mode {
        CapsuleMode::HostDefault => "host_default",
        CapsuleMode::IsolatedNixShell => "isolated_nix_shell",
    }
}

fn apply_runtime_metadata(script: String, capsule: &Capsule) -> String {
    let mut lines = capsule_runtime_env(capsule)
        .into_iter()
        .map(|(key, value)| format!("export {key}={value}"))
        .collect::<Vec<_>>();
    lines.push(format!(
        "export NEXUM_PROCESS_LABEL={}",
        terminal_process_label(&capsule.capsule_id)
    ));
    lines.push(script);
    lines.join("\n")
}
