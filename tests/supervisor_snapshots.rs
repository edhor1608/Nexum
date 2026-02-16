use std::process::Command;

use nexum::{
    capsule::{Capsule, CapsuleMode, CapsuleState},
    events::{EventStore, RuntimeEvent},
    flags::CutoverFlags,
    store::CapsuleStore,
};
use serde_json::Value;
use tempfile::tempdir;

fn seed_supervisor_fixtures(
    capsule_db: &std::path::Path,
    events_db: &std::path::Path,
    flags_file: &std::path::Path,
) {
    let mut store = CapsuleStore::open(capsule_db).unwrap();
    store
        .upsert(Capsule::new("cap-snap-1", "Snap One", CapsuleMode::HostDefault, 41))
        .unwrap();
    store
        .upsert(Capsule::new(
            "cap-snap-2",
            "Snap Two",
            CapsuleMode::IsolatedNixShell,
            42,
        ))
        .unwrap();
    store
        .transition_state("cap-snap-2", CapsuleState::Degraded)
        .unwrap();

    let mut flags = CutoverFlags::default();
    flags.routing_control_plane = true;
    flags.save(flags_file).unwrap();

    let mut events = EventStore::open(events_db).unwrap();
    events
        .append(RuntimeEvent {
            capsule_id: "cap-snap-1".into(),
            component: "restore".into(),
            level: "info".into(),
            message: "ok".into(),
            ts_unix_ms: 1000,
        })
        .unwrap();
    events
        .append(RuntimeEvent {
            capsule_id: "cap-snap-2".into(),
            component: "restore".into(),
            level: "critical".into(),
            message: "failed".into(),
            ts_unix_ms: 1100,
        })
        .unwrap();
}

#[test]
fn snapshot_supervisor_status_contract() {
    let dir = tempdir().unwrap();
    let capsule_db = dir.path().join("capsules.sqlite3");
    let events_db = dir.path().join("events.sqlite3");
    let flags_file = dir.path().join("flags.toml");
    seed_supervisor_fixtures(&capsule_db, &events_db, &flags_file);

    let out = Command::new(assert_cmd::cargo::cargo_bin!("nexumctl"))
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
    insta::assert_yaml_snapshot!("supervisor_status_contract", payload);
}

#[test]
fn snapshot_supervisor_blockers_contract() {
    let dir = tempdir().unwrap();
    let capsule_db = dir.path().join("capsules.sqlite3");
    let events_db = dir.path().join("events.sqlite3");
    let flags_file = dir.path().join("flags.toml");
    seed_supervisor_fixtures(&capsule_db, &events_db, &flags_file);

    let out = Command::new(assert_cmd::cargo::cargo_bin!("nexumctl"))
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
    insta::assert_yaml_snapshot!("supervisor_blockers_contract", payload);
}
