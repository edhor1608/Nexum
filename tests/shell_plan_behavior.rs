use nexum::{
    capsule::{Capsule, CapsuleMode},
    restore::{RestoreRequest, RestoreSurfaces, SignalType, build_restore_plan},
    shell::{NiriShellCommand, build_niri_shell_plan},
};

#[test]
fn niri_shell_plan_follows_restore_sequence() {
    let request = RestoreRequest {
        capsule: Capsule::new(
            "cap-shell-1",
            "Payments Panel",
            CapsuleMode::HostDefault,
            10,
        ),
        signal: SignalType::NeedsDecision,
        surfaces: RestoreSurfaces {
            terminal_cmd: "cd /workspace/payments && nix develop".into(),
            editor_target: "/workspace/payments".into(),
            browser_url: "https://payments-panel.nexum.local".into(),
        },
    };

    let restore = build_restore_plan(&request);
    let shell = build_niri_shell_plan(&restore);

    assert_eq!(shell.workspace, 10);
    assert_eq!(
        shell.commands,
        vec![
            NiriShellCommand::FocusWorkspace(10),
            NiriShellCommand::SpawnTerminal("cd /workspace/payments && nix develop".into()),
            NiriShellCommand::SpawnEditor("/workspace/payments".into()),
            NiriShellCommand::SpawnBrowser("https://payments-panel.nexum.local".into()),
            NiriShellCommand::RaiseAttention("needs_decision".into()),
        ]
    );
}
