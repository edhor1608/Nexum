use std::process::Command;

use nexum::events::{EventStore, RuntimeEvent};
use serde_json::Value;
use tempfile::tempdir;

#[test]
fn nexumctl_events_summary_aggregates_totals_and_capsule_counts() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("events.sqlite3");

    let mut store = EventStore::open(&db).unwrap();
    store
        .append(RuntimeEvent {
            capsule_id: "cap-ev-1".into(),
            component: "restore".into(),
            level: "info".into(),
            message: "restore ready".into(),
            ts_unix_ms: 1000,
        })
        .unwrap();
    store
        .append(RuntimeEvent {
            capsule_id: "cap-ev-1".into(),
            component: "runflow".into(),
            level: "critical".into(),
            message: "restore failed".into(),
            ts_unix_ms: 1010,
        })
        .unwrap();
    store
        .append(RuntimeEvent {
            capsule_id: "cap-ev-2".into(),
            component: "routing".into(),
            level: "warn".into(),
            message: "route degraded".into(),
            ts_unix_ms: 1020,
        })
        .unwrap();

    let nexumctl = assert_cmd::cargo::cargo_bin!("nexumctl");
    let out = Command::new(nexumctl)
        .arg("events")
        .arg("summary")
        .arg("--db")
        .arg(&db)
        .output()
        .unwrap();

    assert!(out.status.success());
    let payload: Value = serde_json::from_slice(&out.stdout).unwrap();

    assert_eq!(payload["total_events"], Value::Number(3u64.into()));
    assert_eq!(payload["critical_events"], Value::Number(1u64.into()));
    assert_eq!(
        payload["capsules"][0]["capsule_id"],
        Value::String("cap-ev-1".into())
    );
    assert_eq!(
        payload["capsules"][0]["critical_events"],
        Value::Number(1u64.into())
    );
    assert_eq!(
        payload["capsules"][1]["capsule_id"],
        Value::String("cap-ev-2".into())
    );
}
