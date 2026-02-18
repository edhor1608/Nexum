use std::process::Command;

use nexum::events::{EventStore, RuntimeEvent};
use serde_json::Value;
use tempfile::tempdir;

#[test]
fn nexumctl_supervisor_status_reports_capsule_and_event_health() {
    let dir = tempdir().unwrap();
    let capsule_db = dir.path().join("capsules.sqlite3");
    let events_db = dir.path().join("events.sqlite3");
    let flags_file = dir.path().join("flags.toml");
    let nexumctl = assert_cmd::cargo::cargo_bin!("nexumctl");

    let create_a = Command::new(nexumctl)
        .arg("capsule")
        .arg("create")
        .arg("--db")
        .arg(&capsule_db)
        .arg("--id")
        .arg("cap-sup-1")
        .arg("--name")
        .arg("Supervisor One")
        .arg("--workspace")
        .arg("30")
        .arg("--mode")
        .arg("host_default")
        .output()
        .unwrap();
    assert!(create_a.status.success());

    let create_b = Command::new(nexumctl)
        .arg("capsule")
        .arg("create")
        .arg("--db")
        .arg(&capsule_db)
        .arg("--id")
        .arg("cap-sup-2")
        .arg("--name")
        .arg("Supervisor Two")
        .arg("--workspace")
        .arg("31")
        .arg("--mode")
        .arg("host_default")
        .output()
        .unwrap();
    assert!(create_b.status.success());

    let degrade_b = Command::new(nexumctl)
        .arg("capsule")
        .arg("set-state")
        .arg("--db")
        .arg(&capsule_db)
        .arg("--id")
        .arg("cap-sup-2")
        .arg("--state")
        .arg("degraded")
        .output()
        .unwrap();
    assert!(degrade_b.status.success());

    let flags = Command::new(nexumctl)
        .arg("flags")
        .arg("set")
        .arg("--file")
        .arg(&flags_file)
        .arg("--routing")
        .arg("true")
        .arg("--restore")
        .arg("true")
        .output()
        .unwrap();
    assert!(flags.status.success());

    let mut events = EventStore::open(&events_db).unwrap();
    events
        .append(RuntimeEvent {
            capsule_id: "cap-sup-1".into(),
            component: "restore".into(),
            level: "info".into(),
            message: "restore ok".into(),
            ts_unix_ms: 1000,
        })
        .unwrap();
    events
        .append(RuntimeEvent {
            capsule_id: "cap-sup-2".into(),
            component: "restore".into(),
            level: "critical".into(),
            message: "restore failed".into(),
            ts_unix_ms: 1010,
        })
        .unwrap();

    let out = Command::new(nexumctl)
        .arg("supervisor")
        .arg("status")
        .arg("--capsule-db")
        .arg(&capsule_db)
        .arg("--events-db")
        .arg(&events_db)
        .arg("--flags-file")
        .arg(&flags_file)
        .output()
        .unwrap();
    assert!(out.status.success());

    let payload: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(payload["total_capsules"], Value::Number(2u64.into()));
    assert_eq!(payload["degraded_capsules"], Value::Number(1u64.into()));
    assert_eq!(payload["critical_events"], Value::Number(1u64.into()));
    assert_eq!(payload["flags"]["routing_control_plane"], Value::Bool(true));
    assert_eq!(payload["flags"]["restore_control_plane"], Value::Bool(true));

    assert_eq!(
        payload["capsules"][0]["capsule_id"],
        Value::String("cap-sup-1".into())
    );
    assert_eq!(
        payload["capsules"][0]["critical_events"],
        Value::Number(0u64.into())
    );
    assert_eq!(
        payload["capsules"][1]["capsule_id"],
        Value::String("cap-sup-2".into())
    );
    assert_eq!(
        payload["capsules"][1]["critical_events"],
        Value::Number(1u64.into())
    );
    assert_eq!(
        payload["capsules"][1]["last_event_level"],
        Value::String("critical".into())
    );
}

#[test]
fn nexumctl_supervisor_blockers_lists_degraded_or_critical_capsules() {
    let dir = tempdir().unwrap();
    let capsule_db = dir.path().join("capsules.sqlite3");
    let events_db = dir.path().join("events.sqlite3");
    let nexumctl = assert_cmd::cargo::cargo_bin!("nexumctl");

    let create_a = Command::new(nexumctl)
        .arg("capsule")
        .arg("create")
        .arg("--db")
        .arg(&capsule_db)
        .arg("--id")
        .arg("cap-block-1")
        .arg("--name")
        .arg("Block One")
        .arg("--workspace")
        .arg("32")
        .arg("--mode")
        .arg("host_default")
        .output()
        .unwrap();
    assert!(create_a.status.success());

    let create_b = Command::new(nexumctl)
        .arg("capsule")
        .arg("create")
        .arg("--db")
        .arg(&capsule_db)
        .arg("--id")
        .arg("cap-block-2")
        .arg("--name")
        .arg("Block Two")
        .arg("--workspace")
        .arg("33")
        .arg("--mode")
        .arg("host_default")
        .output()
        .unwrap();
    assert!(create_b.status.success());

    let degrade_b = Command::new(nexumctl)
        .arg("capsule")
        .arg("set-state")
        .arg("--db")
        .arg(&capsule_db)
        .arg("--id")
        .arg("cap-block-2")
        .arg("--state")
        .arg("degraded")
        .output()
        .unwrap();
    assert!(degrade_b.status.success());

    let mut events = EventStore::open(&events_db).unwrap();
    events
        .append(RuntimeEvent {
            capsule_id: "cap-block-2".into(),
            component: "restore".into(),
            level: "critical".into(),
            message: "restore failed".into(),
            ts_unix_ms: 2000,
        })
        .unwrap();

    let out = Command::new(nexumctl)
        .arg("supervisor")
        .arg("blockers")
        .arg("--capsule-db")
        .arg(&capsule_db)
        .arg("--events-db")
        .arg(&events_db)
        .arg("--critical-threshold")
        .arg("1")
        .output()
        .unwrap();
    assert!(out.status.success());

    let payload: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(payload.as_array().unwrap().len(), 1);
    assert_eq!(
        payload[0]["capsule_id"],
        Value::String("cap-block-2".into())
    );
    assert_eq!(payload[0]["critical_events"], Value::Number(1u64.into()));
    assert_eq!(
        payload[0]["reasons"][0],
        Value::String("state_degraded".into())
    );
    assert_eq!(
        payload[0]["reasons"][1],
        Value::String("critical_events_threshold".into())
    );
}
