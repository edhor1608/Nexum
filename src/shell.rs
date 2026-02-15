use serde::{Deserialize, Serialize};

use crate::restore::{RestorePlan, RestoreStep};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum NiriShellCommand {
    FocusWorkspace(u16),
    SpawnTerminal(String),
    SpawnEditor(String),
    SpawnBrowser(String),
    RaiseAttention(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NiriShellPlan {
    pub workspace: u16,
    pub commands: Vec<NiriShellCommand>,
}

pub fn build_niri_shell_plan(restore: &RestorePlan) -> NiriShellPlan {
    let mut workspace = 1;
    let mut commands = Vec::new();

    for step in &restore.steps {
        match step {
            RestoreStep::FocusWorkspace(id) => {
                workspace = *id;
                commands.push(NiriShellCommand::FocusWorkspace(*id));
            }
            RestoreStep::LaunchTerminal(cmd) => {
                commands.push(NiriShellCommand::SpawnTerminal(cmd.clone()));
            }
            RestoreStep::LaunchEditor(target) => {
                commands.push(NiriShellCommand::SpawnEditor(target.clone()));
            }
            RestoreStep::LaunchBrowser(url) => {
                commands.push(NiriShellCommand::SpawnBrowser(url.clone()));
            }
            RestoreStep::PresentAttention(level) => {
                commands.push(NiriShellCommand::RaiseAttention(level.clone()));
            }
            RestoreStep::EnsureRouting(_) => {}
        }
    }

    NiriShellPlan {
        workspace,
        commands,
    }
}
