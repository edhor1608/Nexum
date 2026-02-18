use serde::{Deserialize, Serialize};

use crate::capsule::Capsule;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalType {
    NeedsDecision,
    CriticalFailure,
    PassiveCompletion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreSurfaces {
    pub terminal_cmd: String,
    pub editor_target: String,
    pub browser_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreRequest {
    pub capsule: Capsule,
    pub signal: SignalType,
    pub surfaces: RestoreSurfaces,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum RestoreStep {
    EnsureRouting(String),
    FocusWorkspace(u16),
    LaunchTerminal(String),
    LaunchEditor(String),
    LaunchBrowser(String),
    PresentAttention(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestorePlan {
    pub capsule_id: String,
    pub signal: SignalType,
    pub target_budget_ms: u64,
    pub steps: Vec<RestoreStep>,
}

pub fn build_restore_plan(request: &RestoreRequest) -> RestorePlan {
    let attention = match request.signal {
        SignalType::NeedsDecision => "needs_decision",
        SignalType::CriticalFailure => "critical_failure",
        SignalType::PassiveCompletion => "passive_completion",
    };

    RestorePlan {
        capsule_id: request.capsule.capsule_id.clone(),
        signal: request.signal,
        target_budget_ms: 9_500,
        steps: vec![
            RestoreStep::EnsureRouting(request.capsule.domain()),
            RestoreStep::FocusWorkspace(request.capsule.workspace),
            RestoreStep::LaunchTerminal(request.surfaces.terminal_cmd.clone()),
            RestoreStep::LaunchEditor(request.surfaces.editor_target.clone()),
            RestoreStep::LaunchBrowser(request.surfaces.browser_url.clone()),
            RestoreStep::PresentAttention(attention.to_string()),
        ],
    }
}
