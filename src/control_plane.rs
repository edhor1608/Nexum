use serde::{Deserialize, Serialize};

use crate::{
    attention::{AttentionEvent, AttentionPolicy},
    restore::{RestoreRequest, RestoreStep, SignalType, build_restore_plan},
    shell::{NiriShellCommand, build_niri_shell_plan},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ExecutionStep {
    EnsureRoute {
        domain: String,
    },
    ShellFocusWorkspace(u16),
    ShellSpawnTerminal(String),
    ShellSpawnEditor(String),
    ShellSpawnBrowser(String),
    EmitAttention {
        priority: String,
        channel: String,
        requires_ack: bool,
        summary: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub capsule_id: String,
    pub target_budget_ms: u64,
    pub steps: Vec<ExecutionStep>,
}

pub fn build_execution_plan(request: &RestoreRequest) -> ExecutionPlan {
    let restore = build_restore_plan(request);
    let shell = build_niri_shell_plan(&restore);
    let attention = AttentionPolicy.route(&AttentionEvent {
        capsule_id: request.capsule.capsule_id.clone(),
        signal: request.signal,
        summary: summarize_signal(request.signal),
    });

    let mut steps = Vec::new();
    for step in &restore.steps {
        if let RestoreStep::EnsureRouting(domain) = step {
            steps.push(ExecutionStep::EnsureRoute {
                domain: domain.clone(),
            });
        }
    }

    for command in shell.commands {
        match command {
            NiriShellCommand::FocusWorkspace(id) => {
                steps.push(ExecutionStep::ShellFocusWorkspace(id))
            }
            NiriShellCommand::SpawnTerminal(cmd) => {
                steps.push(ExecutionStep::ShellSpawnTerminal(cmd))
            }
            NiriShellCommand::SpawnEditor(target) => {
                steps.push(ExecutionStep::ShellSpawnEditor(target))
            }
            NiriShellCommand::SpawnBrowser(url) => {
                steps.push(ExecutionStep::ShellSpawnBrowser(url))
            }
            NiriShellCommand::RaiseAttention(_) => {}
        }
    }

    steps.push(ExecutionStep::EmitAttention {
        priority: to_priority_label(request.signal),
        channel: to_channel_label(request.signal),
        requires_ack: attention.requires_ack,
        summary: attention.summary,
    });

    ExecutionPlan {
        capsule_id: restore.capsule_id,
        target_budget_ms: restore.target_budget_ms,
        steps,
    }
}

fn summarize_signal(signal: SignalType) -> String {
    match signal {
        SignalType::NeedsDecision => "decision needed".to_string(),
        SignalType::CriticalFailure => "critical failure".to_string(),
        SignalType::PassiveCompletion => "passive completion".to_string(),
    }
}

fn to_priority_label(signal: SignalType) -> String {
    match signal {
        SignalType::CriticalFailure => "blocking".to_string(),
        SignalType::NeedsDecision => "active".to_string(),
        SignalType::PassiveCompletion => "passive".to_string(),
    }
}

fn to_channel_label(signal: SignalType) -> String {
    match signal {
        SignalType::CriticalFailure => "banner_and_sound".to_string(),
        SignalType::NeedsDecision => "banner".to_string(),
        SignalType::PassiveCompletion => "feed".to_string(),
    }
}
