use nexum::{
    capsule::CapsuleMode,
    isolation::{IsolationInput, select_capsule_mode},
};

#[test]
fn defaults_to_host_mode_without_escalation_signals() {
    let mode = select_capsule_mode(&IsolationInput {
        identity_collision_detected: false,
        high_risk_secret_workflow: false,
        force_isolated_mode: false,
    });
    assert_eq!(mode, CapsuleMode::HostDefault);
}

#[test]
fn collision_escalates_to_isolated_mode() {
    let mode = select_capsule_mode(&IsolationInput {
        identity_collision_detected: true,
        high_risk_secret_workflow: false,
        force_isolated_mode: false,
    });
    assert_eq!(mode, CapsuleMode::IsolatedNixShell);
}

#[test]
fn high_risk_secret_workflow_escalates_to_isolated_mode() {
    let mode = select_capsule_mode(&IsolationInput {
        identity_collision_detected: false,
        high_risk_secret_workflow: true,
        force_isolated_mode: false,
    });
    assert_eq!(mode, CapsuleMode::IsolatedNixShell);
}

#[test]
fn explicit_override_escalates_to_isolated_mode() {
    let mode = select_capsule_mode(&IsolationInput {
        identity_collision_detected: false,
        high_risk_secret_workflow: false,
        force_isolated_mode: true,
    });
    assert_eq!(mode, CapsuleMode::IsolatedNixShell);
}
