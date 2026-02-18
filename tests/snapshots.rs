use nexum::capsule::{Capsule, CapsuleMode};
use nexum::restore::{RestoreRequest, RestoreSurfaces, SignalType, build_restore_plan};

#[test]
fn snapshot_capsule_record_contract() {
    let capsule = Capsule::new(
        "cap-snap-1",
        "Payments Control Plane",
        CapsuleMode::HostDefault,
        11,
    );
    insta::assert_yaml_snapshot!("capsule_record_contract", capsule);
}

#[test]
fn snapshot_restore_plan_contract() {
    let request = RestoreRequest {
        capsule: Capsule::new(
            "cap-snap-2",
            "Auth Gateway",
            CapsuleMode::IsolatedNixShell,
            9,
        ),
        signal: SignalType::CriticalFailure,
        surfaces: RestoreSurfaces {
            terminal_cmd: "cd /workspace/auth-gateway && nix develop".into(),
            editor_target: "/workspace/auth-gateway".into(),
            browser_url: "https://auth-gateway.nexum.local/ops".into(),
        },
    };

    let plan = build_restore_plan(&request);
    insta::assert_yaml_snapshot!("restore_plan_contract", plan);
}
