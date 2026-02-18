use serde::{Deserialize, Serialize};
use thiserror::Error;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellExecutionReport {
    pub workspace: u16,
    pub executed: Vec<NiriShellCommand>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ShellAdapterError {
    #[error("command failed: {0}")]
    CommandFailed(String),
}

pub trait NiriAdapter {
    fn focus_workspace(&mut self, workspace: u16) -> Result<(), ShellAdapterError>;
    fn spawn_terminal(&mut self, command: &str) -> Result<(), ShellAdapterError>;
    fn spawn_editor(&mut self, target: &str) -> Result<(), ShellAdapterError>;
    fn spawn_browser(&mut self, url: &str) -> Result<(), ShellAdapterError>;
    fn raise_attention(&mut self, level: &str) -> Result<(), ShellAdapterError>;
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

pub fn execute_shell_plan<A: NiriAdapter>(
    plan: &NiriShellPlan,
    adapter: &mut A,
) -> Result<ShellExecutionReport, ShellAdapterError> {
    let mut executed = Vec::new();

    for command in &plan.commands {
        match command {
            NiriShellCommand::FocusWorkspace(id) => adapter.focus_workspace(*id)?,
            NiriShellCommand::SpawnTerminal(cmd) => adapter.spawn_terminal(cmd)?,
            NiriShellCommand::SpawnEditor(target) => adapter.spawn_editor(target)?,
            NiriShellCommand::SpawnBrowser(url) => adapter.spawn_browser(url)?,
            NiriShellCommand::RaiseAttention(level) => adapter.raise_attention(level)?,
        }
        executed.push(command.clone());
    }

    Ok(ShellExecutionReport {
        workspace: plan.workspace,
        executed,
    })
}

pub fn render_shell_script(plan: &NiriShellPlan) -> String {
    let mut lines = Vec::new();

    for command in &plan.commands {
        let line = match command {
            NiriShellCommand::FocusWorkspace(id) => format!("niri msg action focus-workspace {id}"),
            NiriShellCommand::SpawnTerminal(cmd) => {
                format!("wezterm start -- bash -lc {}", shell_quote(cmd))
            }
            NiriShellCommand::SpawnEditor(target) => format!("code {}", shell_quote(target)),
            NiriShellCommand::SpawnBrowser(url) => format!("xdg-open {}", shell_quote(url)),
            NiriShellCommand::RaiseAttention(level) => {
                format!("notify-send 'Nexum Attention' {}", shell_quote(level))
            }
        };
        lines.push(line);
    }

    lines.join("\n")
}

fn shell_quote(input: &str) -> String {
    format!("'{}'", escape_single_quotes(input))
}

fn escape_single_quotes(input: &str) -> String {
    input.replace('\'', "'\"'\"'")
}
