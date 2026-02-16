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
    assert_eq!(
        payload["attention_plan"]["active"],
        Value::Number(2u64.into())
    );
    assert_eq!(
        payload["attention_plan"]["blocking"],
        Value::Number(0u64.into())
    );
    assert_eq!(
        payload["attention_plan"]["passive"],
        Value::Number(0u64.into())
    );
    assert_eq!(
        payload["attention_plan"]["requires_ack_count"],
        Value::Number(2u64.into())
    );
    assert_eq!(
        payload["attention_plan"]["focus_capsule_id"],
        Value::String("cap-stead-batch-1".into())
    );
    assert!(
        payload["results"][1]["error"]
            .as_str()
            .unwrap()
            .contains("unknown capsule")
    );
}

#[test]
fn nexumctl_stead_dispatch_batch_writes_report_file() {
    let dir = tempdir().unwrap();
    let capsule_db = dir.path().join("capsules.sqlite3");
    let tls_dir = dir.path().join("tls");
    let events_db = dir.path().join("events.sqlite3");
    let report_file = dir.path().join("batch-report.json");
    let nexumctl = assert_cmd::cargo::cargo_bin!("nexumctl");

    let create = Command::new(nexumctl)
        .arg("capsule")
        .arg("create")
        .arg("--db")
        .arg(&capsule_db)
        .arg("--id")
        .arg("cap-report")
        .arg("--name")
        .arg("Report Capsule")
        .arg("--workspace")
        .arg("23")
        .arg("--mode")
        .arg("host_default")
        .arg("--repo-path")
        .arg("/workspace/report")
        .output()
        .unwrap();
    assert!(create.status.success());

    let events = r#"[{"capsule_id":"cap-report","signal":"needs_decision","upstream":"127.0.0.1:4794"}]"#;
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
        .arg("--report-file")
        .arg(&report_file)
        .output()
        .unwrap();
    assert!(out.status.success());

    let stdout_payload: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(stdout_payload["processed"], Value::Number(1u64.into()));

    let file_bytes = std::fs::read(&report_file).unwrap();
    let file_payload: Value = serde_json::from_slice(&file_bytes).unwrap();
    assert_eq!(file_payload, stdout_payload);
}

#[test]
fn nexumctl_stead_dispatch_batch_dry_run_has_no_restore_side_effects() {
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
        .arg("cap-dry-run")
        .arg("--name")
        .arg("Dry Run Capsule")
        .arg("--workspace")
        .arg("25")
        .arg("--mode")
        .arg("host_default")
        .arg("--repo-path")
        .arg("/workspace/dry-run")
        .output()
        .unwrap();
    assert!(create.status.success());

    let events = r#"[{"capsule_id":"cap-dry-run","signal":"needs_decision","upstream":"127.0.0.1:4795"},{"capsule_id":"cap-dry-run-missing","signal":"critical_failure","upstream":"127.0.0.1:4796"}]"#;
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
        .arg("--dry-run")
        .arg("true")
        .output()
        .unwrap();
    assert!(out.status.success());

    let payload: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(payload["dry_run"], Value::Bool(true));
    assert_eq!(payload["processed"], Value::Number(2u64.into()));
    assert_eq!(payload["succeeded"], Value::Number(1u64.into()));
    assert_eq!(payload["failed"], Value::Number(1u64.into()));

    let summary = Command::new(nexumctl)
        .arg("events")
        .arg("summary")
        .arg("--db")
        .arg(&events_db)
        .output()
        .unwrap();
    assert!(summary.status.success());
    let summary_payload: Value = serde_json::from_slice(&summary.stdout).unwrap();
    assert_eq!(summary_payload["total_events"], Value::Number(0u64.into()));
}

#[test]
fn nexumctl_stead_dispatch_batch_fail_on_missing_capsules_aborts_without_side_effects() {
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
        .arg("cap-stead-batch-ok")
        .arg("--name")
        .arg("Stead Batch OK")
        .arg("--workspace")
        .arg("22")
        .arg("--mode")
        .arg("host_default")
        .arg("--repo-path")
        .arg("/workspace/stead-batch-ok")
        .output()
        .unwrap();
    assert!(create.status.success());

    let events = r#"[{"capsule_id":"cap-stead-batch-ok","signal":"needs_decision","upstream":"127.0.0.1:4792"},{"capsule_id":"cap-stead-batch-missing","signal":"needs_decision","upstream":"127.0.0.1:4793"}]"#;
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
        .arg("--fail-on-missing-capsules")
        .arg("true")
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("unknown capsules in batch"));
    assert!(stderr.contains("cap-stead-batch-missing"));

    let summary = Command::new(nexumctl)
        .arg("events")
        .arg("summary")
        .arg("--db")
        .arg(&events_db)
        .output()
        .unwrap();
    assert!(summary.status.success());
    let payload: Value = serde_json::from_slice(&summary.stdout).unwrap();
    assert_eq!(payload["total_events"], Value::Number(0u64.into()));
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

#[test]
fn nexumctl_stead_attention_plan_routes_priorities_and_focus() {
    let nexumctl = assert_cmd::cargo::cargo_bin!("nexumctl");
    let events = r#"[{"capsule_id":"cap-passive","signal":"passive_completion","upstream":"127.0.0.1:5100"},{"capsule_id":"cap-active","signal":"needs_decision","upstream":"127.0.0.1:5101"},{"capsule_id":"cap-critical","signal":"critical_failure","upstream":"127.0.0.1:5102"}]"#;

    let out = Command::new(nexumctl)
        .arg("stead")
        .arg("attention-plan")
        .arg("--events-json")
        .arg(events)
        .output()
        .unwrap();
    assert!(out.status.success());

    let payload: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(payload["blocking"], Value::Number(1u64.into()));
    assert_eq!(payload["active"], Value::Number(1u64.into()));
    assert_eq!(payload["passive"], Value::Number(1u64.into()));
    assert_eq!(payload["requires_ack_count"], Value::Number(2u64.into()));
    assert_eq!(
        payload["focus_capsule_id"],
        Value::String("cap-critical".into())
    );
    assert_eq!(payload["routes"][0]["priority"], Value::String("passive".into()));
    assert_eq!(payload["routes"][1]["priority"], Value::String("active".into()));
    assert_eq!(payload["routes"][2]["priority"], Value::String("blocking".into()));
}
