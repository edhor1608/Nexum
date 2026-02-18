use std::process::Command;

use serde_json::Value;
use tempfile::tempdir;

#[test]
fn nexumctl_stead_dispatch_runs_restore_from_event_envelope() {
    let dir = tempdir().unwrap();
    let capsule_db = dir.path().join("capsules.sqlite3");
    let tls_dir = dir.path().join("tls");
    let events_db = dir.path().join("events.sqlite3");
    let nexumctl = assert_cmd::cargo::cargo_bin!("nexumctl");

    let create = Command::new(nexumctl)
        .arg("capsule")
        .arg("create")
        .arg("--db")
        .arg(&capsule_db)
        .arg("--id")
        .arg("cap-stead-1")
        .arg("--name")
        .arg("Stead Capsule")
        .arg("--workspace")
        .arg("20")
        .arg("--mode")
        .arg("host_default")
        .arg("--repo-path")
        .arg("/workspace/stead")
        .output()
        .unwrap();
    assert!(create.status.success());

    let envelope =
        r#"{"capsule_id":"cap-stead-1","signal":"needs_decision","upstream":"127.0.0.1:4788"}"#;

    let out = Command::new(nexumctl)
        .arg("stead")
        .arg("dispatch")
        .arg("--capsule-db")
        .arg(&capsule_db)
        .arg("--event-json")
        .arg(envelope)
        .arg("--tls-dir")
        .arg(&tls_dir)
        .arg("--events-db")
        .arg(&events_db)
        .output()
        .unwrap();
    assert!(out.status.success());

    let payload: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(payload["capsule_id"], Value::String("cap-stead-1".into()));
    let script = payload["shell_script"].as_str().unwrap();
    assert!(script.contains("cd /workspace/stead && nix develop"));
    assert!(script.contains("code /workspace/stead"));
    assert!(script.contains("xdg-open https://stead-capsule.nexum.local"));
}

#[test]
fn nexumctl_stead_dispatch_rejects_invalid_event_payload() {
    let dir = tempdir().unwrap();
    let capsule_db = dir.path().join("capsules.sqlite3");
    let tls_dir = dir.path().join("tls");
    let events_db = dir.path().join("events.sqlite3");
    let nexumctl = assert_cmd::cargo::cargo_bin!("nexumctl");

    let out = Command::new(nexumctl)
        .arg("stead")
        .arg("dispatch")
        .arg("--capsule-db")
        .arg(&capsule_db)
        .arg("--event-json")
        .arg("{\"capsule_id\":\"cap-stead-1\"}")
        .arg("--tls-dir")
        .arg(&tls_dir)
        .arg("--events-db")
        .arg(&events_db)
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("event-json"));
}

#[test]
fn nexumctl_stead_dispatch_batch_reports_per_event_results() {
    let dir = tempdir().unwrap();
    let capsule_db = dir.path().join("capsules.sqlite3");
    let tls_dir = dir.path().join("tls");
    let events_db = dir.path().join("events.sqlite3");
    let nexumctl = assert_cmd::cargo::cargo_bin!("nexumctl");

    let create = Command::new(nexumctl)
        .arg("capsule")
        .arg("create")
        .arg("--db")
        .arg(&capsule_db)
        .arg("--id")
        .arg("cap-stead-batch-1")
        .arg("--name")
        .arg("Stead Batch One")
        .arg("--workspace")
        .arg("21")
        .arg("--mode")
        .arg("host_default")
        .arg("--repo-path")
        .arg("/workspace/stead-batch-1")
        .output()
        .unwrap();
    assert!(create.status.success());

    let events = r#"[{"capsule_id":"cap-stead-batch-1","signal":"needs_decision","upstream":"127.0.0.1:4790"},{"capsule_id":"missing-cap","signal":"needs_decision","upstream":"127.0.0.1:4791"}]"#;
    let out = Command::new(nexumctl)
        .arg("stead")
        .arg("dispatch-batch")
        .arg("--capsule-db")
        .arg(&capsule_db)
        .arg("--events-json")
        .arg(events)
        .arg("--tls-dir")
        .arg(&tls_dir)
        .arg("--events-db")
        .arg(&events_db)
        .output()
        .unwrap();
    assert!(out.status.success());

    let payload: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(payload["processed"], Value::Number(2u64.into()));
    assert_eq!(payload["succeeded"], Value::Number(1u64.into()));
    assert_eq!(payload["failed"], Value::Number(1u64.into()));
    assert_eq!(
        payload["results"][0]["capsule_id"],
        Value::String("cap-stead-batch-1".into())
    );
    assert_eq!(payload["results"][0]["ok"], Value::Bool(true));
    assert_eq!(
        payload["results"][1]["capsule_id"],
        Value::String("missing-cap".into())
    );
    assert_eq!(payload["results"][1]["ok"], Value::Bool(false));
    assert!(
        payload["results"][1]["error"]
            .as_str()
            .unwrap()
            .contains("unknown capsule")
    );
}

#[test]
fn nexumctl_stead_validate_events_reports_batch_shape() {
    let nexumctl = assert_cmd::cargo::cargo_bin!("nexumctl");
    let events = r#"[{"capsule_id":"cap-a","signal":"needs_decision","upstream":"127.0.0.1:4800"},{"capsule_id":"cap-b","signal":"critical_failure","upstream":"127.0.0.1:4801"}]"#;

    let out = Command::new(nexumctl)
        .arg("stead")
        .arg("validate-events")
        .arg("--events-json")
        .arg(events)
        .output()
        .unwrap();
    assert!(out.status.success());

    let payload: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(payload["valid"], Value::Bool(true));
    assert_eq!(payload["event_count"], Value::Number(2u64.into()));
    assert_eq!(payload["capsule_ids"][0], Value::String("cap-a".into()));
    assert_eq!(payload["capsule_ids"][1], Value::String("cap-b".into()));
    assert_eq!(payload["missing_capsule_ids"], Value::Array(vec![]));
}

#[test]
fn nexumctl_stead_validate_events_reports_missing_capsules_when_db_is_provided() {
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
        .arg("24")
        .arg("--mode")
        .arg("host_default")
        .arg("--repo-path")
        .arg("/workspace/known")
        .output()
        .unwrap();
    assert!(create.status.success());

    let events = r#"[{"capsule_id":"cap-known","signal":"needs_decision","upstream":"127.0.0.1:4900"},{"capsule_id":"cap-missing","signal":"critical_failure","upstream":"127.0.0.1:4901"},{"capsule_id":"cap-missing","signal":"passive_completion","upstream":"127.0.0.1:4902"}]"#;
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
    assert_eq!(payload["valid"], Value::Bool(false));
    assert_eq!(payload["event_count"], Value::Number(3u64.into()));
    assert_eq!(payload["capsule_ids"][0], Value::String("cap-known".into()));
    assert_eq!(payload["capsule_ids"][1], Value::String("cap-missing".into()));
    assert_eq!(
        payload["missing_capsule_ids"],
        Value::Array(vec![Value::String("cap-missing".into())])
    );
}
