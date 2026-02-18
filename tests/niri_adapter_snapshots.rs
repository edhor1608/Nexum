use nexum::shell::{NiriShellCommand, NiriShellPlan, render_shell_script};

#[test]
fn snapshot_rendered_niri_shell_script_contract() {
    let plan = NiriShellPlan {
        workspace: 7,
        commands: vec![
            NiriShellCommand::FocusWorkspace(7),
            NiriShellCommand::SpawnTerminal("cd /workspace/search && nix develop".into()),
            NiriShellCommand::SpawnEditor("/workspace/search".into()),
            NiriShellCommand::SpawnBrowser("https://search.nexum.local".into()),
            NiriShellCommand::RaiseAttention("critical_failure".into()),
        ],
    };

    let script = render_shell_script(&plan);
    insta::assert_snapshot!("niri_shell_script_contract", script);
}
