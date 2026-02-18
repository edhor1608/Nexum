use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    capsule::{Capsule, CapsuleMode},
    events::{EventError, EventStore, RuntimeEvent},
    restore::{RestoreRequest, RestoreSurfaces, SignalType, build_restore_plan},
    routing::{RouteCommand, RouteOutcome, RouterState},
    shell::{build_niri_shell_plan, render_shell_script},
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
    pub tls_dir: PathBuf,
    pub events_db: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreRunSummary {
    pub capsule_id: String,
    pub domain: String,
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
    #[error("routing failed: {0}")]
    Routing(String),
    #[error("time: {0}")]
    Time(#[from] std::time::SystemTimeError),
}

pub fn run_restore_flow(input: RestoreRunInput) -> Result<RestoreRunSummary, RunFlowError> {
    let capsule = Capsule::new(
        &input.capsule_id,
        &input.display_name,
        CapsuleMode::HostDefault,
        input.workspace,
    );

    let request = RestoreRequest {
        capsule: capsule.clone(),
        signal: input.signal,
        surfaces: RestoreSurfaces {
            terminal_cmd: input.terminal_cmd,
            editor_target: input.editor_target,
            browser_url: input.browser_url,
        },
    };

    let restore = build_restore_plan(&request);

    let tls = ensure_self_signed_cert(&input.tls_dir, &capsule.domain(), 30)?;

    let mut router = RouterState::default();
    let route = router.handle(RouteCommand::Register {
        capsule_id: capsule.capsule_id.clone(),
        domain: capsule.domain(),
        upstream: input.route_upstream,
    });
    if let RouteOutcome::Error { message, .. } = route {
        return Err(RunFlowError::Routing(message));
    }

    let shell_script = render_shell_script(&build_niri_shell_plan(&restore));

    let mut events = EventStore::open(&input.events_db)?;
    let events_payloads = vec![
        RuntimeEvent {
            capsule_id: capsule.capsule_id.clone(),
            component: "runflow".into(),
            level: "info".into(),
            message: "restore start".into(),
            ts_unix_ms: now_unix_ms()?,
        },
        RuntimeEvent {
            capsule_id: capsule.capsule_id.clone(),
            component: "routing".into(),
            level: "info".into(),
            message: format!("route ensured for {}", capsule.domain()),
            ts_unix_ms: now_unix_ms()?,
        },
        RuntimeEvent {
            capsule_id: capsule.capsule_id.clone(),
            component: "runflow".into(),
            level: "info".into(),
            message: "restore plan ready".into(),
            ts_unix_ms: now_unix_ms()?,
        },
    ];
    let events_written = events_payloads.len() as u32;
    for event in events_payloads {
        events.append(event)?;
    }

    Ok(RestoreRunSummary {
        capsule_id: capsule.capsule_id.clone(),
        domain: capsule.domain(),
        target_budget_ms: restore.target_budget_ms,
        shell_script,
        tls_fingerprint_sha256: tls.fingerprint_sha256,
        events_written,
    })
}

fn now_unix_ms() -> Result<u64, std::time::SystemTimeError> {
    Ok(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis() as u64,
    )
}
