use nexum::capsule::{Capsule, CapsuleMode};
use nexum::restore::{
    RestoreRequest, RestoreStep, RestoreSurfaces, SignalType, build_restore_plan,
};

fn sample_request(signal: SignalType) -> RestoreRequest {
    RestoreRequest {
        capsule: Capsule::new(
            "cap-restore-1",
            "Search Platform",
            CapsuleMode::HostDefault,
            7,
        ),
        signal,
        surfaces: RestoreSurfaces {
            terminal_cmd: "cd /workspace/search && nix develop".into(),
            editor_target: "/workspace/search".into(),
            browser_url: "https://search-platform.nexum.local/dashboard".into(),
        },
    }
}

#[test]
fn restore_plan_for_needs_decision_has_required_step_order() {
    let plan = build_restore_plan(&sample_request(SignalType::NeedsDecision));

    assert_eq!(plan.capsule_id, "cap-restore-1");
    assert!(plan.target_budget_ms <= 10_000);
    assert_eq!(
        plan.steps,
        vec![
            RestoreStep::EnsureRouting("search-platform.nexum.local".into()),
            RestoreStep::FocusWorkspace(7),
            RestoreStep::LaunchTerminal("cd /workspace/search && nix develop".into()),
            RestoreStep::LaunchEditor("/workspace/search".into()),
            RestoreStep::LaunchBrowser("https://search-platform.nexum.local/dashboard".into()),
            RestoreStep::PresentAttention("needs_decision".into()),
        ]
    );
}

#[test]
fn restore_plan_for_passive_completion_uses_low_urgency_attention() {
    let plan = build_restore_plan(&sample_request(SignalType::PassiveCompletion));

    assert!(
        matches!(plan.steps.last(), Some(RestoreStep::PresentAttention(level)) if level == "passive_completion")
    );
}
