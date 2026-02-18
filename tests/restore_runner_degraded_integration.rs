use nexum::{
    restore::SignalType,
    runflow::{RestoreRunInput, run_restore_flow},
};
use tempfile::tempdir;

#[test]
fn run_restore_flow_degrades_when_routing_socket_is_unavailable() {
    let dir = tempdir().unwrap();
    let missing_socket = dir.path().join("missing.sock");

    let summary = run_restore_flow(RestoreRunInput {
        capsule_id: "cap-degraded-1".into(),
        display_name: "Degraded Restore".into(),
        workspace: 13,
        signal: SignalType::NeedsDecision,
        terminal_cmd: "cd /workspace/degraded && nix develop".into(),
        editor_target: "/workspace/degraded".into(),
        browser_url: "https://degraded-restore.nexum.local".into(),
        route_upstream: "127.0.0.1:4900".into(),
        routing_socket: Some(missing_socket),
        identity_collision: false,
        high_risk_secret_workflow: false,
        force_isolated_mode: false,
        capsule_db: None,
        tls_dir: dir.path().join("tls"),
        events_db: dir.path().join("events.sqlite3"),
    })
    .unwrap();

    assert!(summary.degraded);
    assert!(
        summary
            .degraded_reason
            .as_deref()
            .unwrap()
            .contains("route_unavailable")
    );
}
