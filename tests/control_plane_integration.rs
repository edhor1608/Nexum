use nexum::{
    capsule::{Capsule, CapsuleMode},
    control_plane::{ExecutionStep, build_execution_plan},
    restore::{RestoreRequest, RestoreSurfaces, SignalType},
};

#[test]
fn control_plane_plan_includes_routing_shell_and_attention() {
    let request = RestoreRequest {
        capsule: Capsule::new("cap-cp-1", "Search API", CapsuleMode::HostDefault, 6),
        signal: SignalType::CriticalFailure,
        surfaces: RestoreSurfaces {
            terminal_cmd: "cd /workspace/search-api && nix develop".into(),
            editor_target: "/workspace/search-api".into(),
            browser_url: "https://search-api.nexum.local/ops".into(),
        },
    };

    let execution = build_execution_plan(&request);

    assert!(execution.target_budget_ms <= 10_000);
    assert_eq!(execution.capsule_id, "cap-cp-1");
    assert!(matches!(
        execution.steps[0],
        ExecutionStep::EnsureRoute { .. }
    ));
    assert!(
        execution
            .steps
            .iter()
            .any(|step| matches!(step, ExecutionStep::ShellFocusWorkspace(6)))
    );
    assert!(execution.steps.iter().any(|step| matches!(step, ExecutionStep::ShellSpawnBrowser(url) if url == "https://search-api.nexum.local/ops")));
    assert!(execution.steps.iter().any(|step| matches!(step, ExecutionStep::EmitAttention { priority, .. } if priority == "blocking")));
}
