use nexum::{
    events::EventStore,
    restore::SignalType,
    runflow::{RestoreRunInput, run_restore_flow},
};
use tempfile::tempdir;

#[test]
fn run_restore_flow_returns_script_and_persists_events() {
    let dir = tempdir().unwrap();
    let tls_dir = dir.path().join("tls");
    let events_db = dir.path().join("events.sqlite3");

    let summary = run_restore_flow(RestoreRunInput {
        capsule_id: "cap-run-1".into(),
        display_name: "Runner API".into(),
        workspace: 8,
        signal: SignalType::NeedsDecision,
        terminal_cmd: "cd /workspace/runner && nix develop".into(),
        editor_target: "/workspace/runner".into(),
        browser_url: "https://runner-api.nexum.local".into(),
        route_upstream: "127.0.0.1:4700".into(),
        routing_socket: None,
        tls_dir,
        events_db: events_db.clone(),
    })
    .unwrap();

    assert_eq!(summary.capsule_id, "cap-run-1");
    assert!(summary.target_budget_ms <= 10_000);
    assert!(
        summary
            .shell_script
            .contains("niri msg action focus-workspace 8")
    );
    assert!(
        summary
            .shell_script
            .contains("notify-send 'Nexum Attention' 'needs_decision'")
    );
    assert!(summary.tls_fingerprint_sha256.len() >= 16);
    assert_eq!(summary.events_written, 3);

    let store = EventStore::open(&events_db).unwrap();
    let events = store.list_for_capsule("cap-run-1").unwrap();
    assert_eq!(events.len(), 3);
}
