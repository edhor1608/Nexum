use std::process::Command;

use serde_json::Value;
use tempfile::tempdir;

#[test]
fn snapshot_stead_validate_events_contract() {
    let dir = tempdir().unwrap();
    let capsule_db = dir.path().join("capsules.sqlite3");
    let nexumctl = assert_cmd::cargo::cargo_bin!("nexumctl");

    let create = Command::new(nexumctl)
        .arg("capsule")
        .arg("create")
        .arg("--db")
        .arg(&capsule_db)
        .arg("--id")
        .arg("cap-known")
        .arg("--name")
        .arg("Known Capsule")
        .arg("--workspace")
        .arg("30")
        .arg("--mode")
        .arg("host_default")
        .arg("--repo-path")
        .arg("/workspace/known")
        .output()
        .unwrap();
    assert!(create.status.success());

    let events = r#"[{"capsule_id":"cap-known","signal":"needs_decision","upstream":"127.0.0.1:5000"},{"capsule_id":"cap-missing","signal":"critical_failure","upstream":"127.0.0.1:5001"}]"#;
    let out = Command::new(nexumctl)
        .arg("stead")
        .arg("validate-events")
        .arg("--events-json")
        .arg(events)
        .arg("--capsule-db")
        .arg(&capsule_db)
        .output()
        .unwrap();
    assert!(out.status.success());

    let payload: Value = serde_json::from_slice(&out.stdout).unwrap();
    insta::assert_yaml_snapshot!("stead_validate_events_contract", payload);
}
