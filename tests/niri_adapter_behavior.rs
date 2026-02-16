use nexum::shell::{
    NiriAdapter, NiriShellCommand, NiriShellPlan, ShellAdapterError, execute_shell_plan,
};

#[derive(Default)]
struct RecordingAdapter {
    calls: Vec<String>,
    fail_on: Option<&'static str>,
}

impl NiriAdapter for RecordingAdapter {
    fn focus_workspace(&mut self, workspace: u16) -> Result<(), ShellAdapterError> {
        self.calls.push(format!("focus:{workspace}"));
        if self.fail_on == Some("focus") {
            return Err(ShellAdapterError::CommandFailed("focus".into()));
        }
        Ok(())
    }

    fn spawn_terminal(&mut self, command: &str) -> Result<(), ShellAdapterError> {
        self.calls.push(format!("terminal:{command}"));
        if self.fail_on == Some("terminal") {
            return Err(ShellAdapterError::CommandFailed("terminal".into()));
        }
        Ok(())
    }

    fn spawn_editor(&mut self, target: &str) -> Result<(), ShellAdapterError> {
        self.calls.push(format!("editor:{target}"));
        if self.fail_on == Some("editor") {
            return Err(ShellAdapterError::CommandFailed("editor".into()));
        }
        Ok(())
    }

    fn spawn_browser(&mut self, url: &str) -> Result<(), ShellAdapterError> {
        self.calls.push(format!("browser:{url}"));
        if self.fail_on == Some("browser") {
            return Err(ShellAdapterError::CommandFailed("browser".into()));
        }
        Ok(())
    }

    fn raise_attention(&mut self, level: &str) -> Result<(), ShellAdapterError> {
        self.calls.push(format!("attention:{level}"));
        if self.fail_on == Some("attention") {
            return Err(ShellAdapterError::CommandFailed("attention".into()));
        }
        Ok(())
    }
}

fn sample_plan() -> NiriShellPlan {
    NiriShellPlan {
        workspace: 4,
        commands: vec![
            NiriShellCommand::FocusWorkspace(4),
            NiriShellCommand::SpawnTerminal("cd /workspace/core && nix develop".into()),
            NiriShellCommand::SpawnEditor("/workspace/core".into()),
            NiriShellCommand::SpawnBrowser("https://core.nexum.local".into()),
            NiriShellCommand::RaiseAttention("needs_decision".into()),
        ],
    }
}

#[test]
fn executes_shell_commands_in_order() {
    let plan = sample_plan();
    let mut adapter = RecordingAdapter::default();

    let report = execute_shell_plan(&plan, &mut adapter).expect("execution should succeed");

    assert_eq!(report.executed, plan.commands);
    assert_eq!(
        adapter.calls,
        vec![
            "focus:4",
            "terminal:cd /workspace/core && nix develop",
            "editor:/workspace/core",
            "browser:https://core.nexum.local",
            "attention:needs_decision",
        ]
    );
}

#[test]
fn aborts_on_first_failure() {
    let plan = sample_plan();
    let mut adapter = RecordingAdapter {
        fail_on: Some("editor"),
        ..Default::default()
    };

    let err = execute_shell_plan(&plan, &mut adapter).expect_err("execution should fail");
    assert!(matches!(err, ShellAdapterError::CommandFailed(name) if name == "editor"));

    assert_eq!(
        adapter.calls,
        vec![
            "focus:4",
            "terminal:cd /workspace/core && nix develop",
            "editor:/workspace/core",
        ]
    );
}
