use std::process::Command;

use nexum::events::{EventStore, RuntimeEvent};
use serde_json::Value;
use tempfile::tempdir;

#[test]
fn cutover_apply_from_events_allows_when_critical_event_count_is_within_threshold() {
    let dir = tempdir().unwrap();
    let flags = dir.path().join("flags.toml");
    let events_db = dir.path().join("events.sqlite3");

    let mut events = EventStore::open(&events_db).unwrap();
    events
        .append(RuntimeEvent {
            capsule_id: "cap-cutover-events-ok".into(),
            component: "restore".into(),
            level: "info".into(),
            message: "restore ready".into(),
            ts_unix_ms: 1000,
        })
        .unwrap();

    let nexumctl = assert_cmd::cargo::cargo_bin!("nexumctl");
    let out = Command::new(nexumctl)
        .arg("cutover")
        .arg("apply-from-events")
        .arg("--file")
        .arg(&flags)
        .arg("--capability")
        .arg("routing")
        .arg("--parity-score")
        .arg("1.0")
        .arg("--min-parity-score")
        .arg("0.95")
        .arg("--events-db")
        .arg(&events_db)
        .arg("--capsule-id")
        .arg("cap-cutover-events-ok")
        .arg("--max-critical-events")
        .arg("0")
        .arg("--shadow-mode")
        .arg("true")
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("\"allowed\":true"));

    let show = Command::new(nexumctl)
        .arg("flags")
        .arg("show")
        .arg("--file")
        .arg(&flags)
        .output()
        .unwrap();
    assert!(show.status.success());
    let flags_stdout = String::from_utf8(show.stdout).unwrap();
    assert!(flags_stdout.contains("\"routing_control_plane\":true"));
}

#[test]
fn cutover_apply_from_events_denies_when_critical_events_exceed_threshold() {
    let dir = tempdir().unwrap();
    let flags = dir.path().join("flags.toml");
    let events_db = dir.path().join("events.sqlite3");

    let mut events = EventStore::open(&events_db).unwrap();
    events
        .append(RuntimeEvent {
            capsule_id: "cap-cutover-events-block".into(),
            component: "runflow".into(),
            level: "critical".into(),
            message: "routing broken".into(),
            ts_unix_ms: 1000,
        })
        .unwrap();

    let nexumctl = assert_cmd::cargo::cargo_bin!("nexumctl");
    let out = Command::new(nexumctl)
        .arg("cutover")
        .arg("apply-from-events")
        .arg("--file")
        .arg(&flags)
        .arg("--capability")
        .arg("routing")
        .arg("--parity-score")
        .arg("1.0")
        .arg("--min-parity-score")
        .arg("0.95")
        .arg("--events-db")
        .arg(&events_db)
        .arg("--capsule-id")
        .arg("cap-cutover-events-block")
        .arg("--max-critical-events")
        .arg("0")
        .arg("--shadow-mode")
        .arg("true")
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("\"allowed\":false"));
    assert!(stdout.contains("critical events"));
}

#[test]
fn cutover_apply_from_summary_allows_when_global_critical_events_are_within_threshold() {
    let dir = tempdir().unwrap();
    let flags = dir.path().join("flags.toml");
    let events_db = dir.path().join("events.sqlite3");
    let nexumctl = assert_cmd::cargo::cargo_bin!("nexumctl");

    let mut events = EventStore::open(&events_db).unwrap();
    events
        .append(RuntimeEvent {
            capsule_id: "cap-sum-a".into(),
            component: "runflow".into(),
            level: "critical".into(),
            message: "one failure".into(),
            ts_unix_ms: 1000,
        })
        .unwrap();

    let out = Command::new(nexumctl)
        .arg("cutover")
        .arg("apply-from-summary")
        .arg("--file")
        .arg(&flags)
        .arg("--capability")
        .arg("restore")
        .arg("--parity-score")
        .arg("1.0")
        .arg("--min-parity-score")
        .arg("0.95")
        .arg("--events-db")
        .arg(&events_db)
        .arg("--max-critical-events")
        .arg("2")
        .arg("--shadow-mode")
        .arg("true")
        .output()
        .unwrap();
    assert!(out.status.success());
    let payload: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(payload["allowed"], Value::Bool(true));
    assert_eq!(
        payload["flag_to_enable"],
        Value::String("restore_control_plane".into())
    );

    let show = Command::new(nexumctl)
        .arg("flags")
        .arg("show")
        .arg("--file")
        .arg(&flags)
        .output()
        .unwrap();
    assert!(show.status.success());
    let flags_payload: Value = serde_json::from_slice(&show.stdout).unwrap();
    assert_eq!(flags_payload["restore_control_plane"], Value::Bool(true));
}

#[test]
fn cutover_apply_from_summary_denies_when_global_critical_events_exceed_threshold() {
    let dir = tempdir().unwrap();
    let flags = dir.path().join("flags.toml");
    let events_db = dir.path().join("events.sqlite3");
    let nexumctl = assert_cmd::cargo::cargo_bin!("nexumctl");

    let mut events = EventStore::open(&events_db).unwrap();
    events
        .append(RuntimeEvent {
            capsule_id: "cap-sum-a".into(),
            component: "runflow".into(),
            level: "critical".into(),
            message: "failure a".into(),
            ts_unix_ms: 1000,
        })
        .unwrap();
    events
        .append(RuntimeEvent {
            capsule_id: "cap-sum-b".into(),
            component: "runflow".into(),
            level: "critical".into(),
            message: "failure b".into(),
            ts_unix_ms: 1010,
        })
        .unwrap();

    let out = Command::new(nexumctl)
        .arg("cutover")
        .arg("apply-from-summary")
        .arg("--file")
        .arg(&flags)
        .arg("--capability")
        .arg("restore")
        .arg("--parity-score")
        .arg("1.0")
        .arg("--min-parity-score")
        .arg("0.95")
        .arg("--events-db")
        .arg(&events_db)
        .arg("--max-critical-events")
        .arg("1")
        .arg("--shadow-mode")
        .arg("true")
        .output()
        .unwrap();
    assert!(out.status.success());
    let payload: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(payload["allowed"], Value::Bool(false));
    assert!(
        payload["reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reason| reason.as_str().unwrap().contains("critical events"))
    );

    let show = Command::new(nexumctl)
        .arg("flags")
        .arg("show")
        .arg("--file")
        .arg(&flags)
        .output()
        .unwrap();
    assert!(show.status.success());
    let flags_payload: Value = serde_json::from_slice(&show.stdout).unwrap();
    assert_eq!(flags_payload["restore_control_plane"], Value::Bool(false));
}
