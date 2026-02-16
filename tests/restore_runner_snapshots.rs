use nexum::{
    restore::SignalType,
    runflow::{RestoreRunInput, run_restore_flow},
};
use tempfile::tempdir;

#[test]
fn snapshot_restore_runner_summary_contract() {
    let dir = tempdir().unwrap();
    let tls_dir = dir.path().join("tls");
    let events_db = dir.path().join("events.sqlite3");

    let mut summary = run_restore_flow(RestoreRunInput {
        capsule_id: "cap-run-snap".into(),
        display_name: "Runner Snap".into(),
        workspace: 5,
        signal: SignalType::CriticalFailure,
        terminal_cmd: "cd /workspace/snap && nix develop".into(),
        editor_target: "/workspace/snap".into(),
        browser_url: "https://runner-snap.nexum.local".into(),
        route_upstream: "127.0.0.1:4710".into(),
        routing_socket: None,
        identity_collision: false,
        tls_dir,
        events_db,
    })
    .unwrap();

    summary.tls_fingerprint_sha256 = "<fingerprint>".into();
    insta::assert_yaml_snapshot!("restore_runner_summary_contract", summary);
}
