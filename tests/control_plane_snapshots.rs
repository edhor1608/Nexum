use nexum::{
    capsule::{Capsule, CapsuleMode},
    control_plane::build_execution_plan,
    restore::{RestoreRequest, RestoreSurfaces, SignalType},
};

#[test]
fn snapshot_control_plane_execution_contract() {
    let request = RestoreRequest {
        capsule: Capsule::new(
            "cap-cp-snap",
            "Agent Gateway",
            CapsuleMode::IsolatedNixShell,
            3,
        ),
        signal: SignalType::NeedsDecision,
        surfaces: RestoreSurfaces {
            terminal_cmd: "cd /workspace/agent-gateway && nix develop".into(),
            editor_target: "/workspace/agent-gateway".into(),
            browser_url: "https://agent-gateway.nexum.local/dashboard".into(),
        },
    };

    let execution = build_execution_plan(&request);
    insta::assert_yaml_snapshot!("control_plane_execution_contract", execution);
}
