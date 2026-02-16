use std::process::Command;

use serde_json::Value;
use tempfile::tempdir;

#[test]
fn nexumctl_run_restore_executes_end_to_end_plan() {
    let dir = tempdir().unwrap();
    let tls_dir = dir.path().join("tls");
    let events_db = dir.path().join("events.sqlite3");
    let nexumctl = assert_cmd::cargo::cargo_bin!("nexumctl");

    let out = Command::new(nexumctl)
        .arg("run")
        .arg("restore")
        .arg("--capsule-id")
        .arg("cap-run-cli")
        .arg("--name")
        .arg("Runner CLI")
        .arg("--workspace")
        .arg("6")
        .arg("--signal")
        .arg("needs_decision")
        .arg("--terminal")
        .arg("cd /workspace/cli && nix develop")
        .arg("--editor")
        .arg("/workspace/cli")
        .arg("--browser")
        .arg("https://runner-cli.nexum.local")
        .arg("--upstream")
        .arg("127.0.0.1:4720")
        .arg("--tls-dir")
        .arg(&tls_dir)
        .arg("--events-db")
        .arg(&events_db)
        .output()
        .unwrap();

    assert!(out.status.success());
    let value: Value = serde_json::from_slice(&out.stdout).unwrap();

    assert_eq!(value["capsule_id"], Value::String("cap-run-cli".into()));
    assert!(value["target_budget_ms"].as_u64().unwrap() <= 10_000);
    assert!(
        value["shell_script"]
            .as_str()
            .unwrap()
            .contains("xdg-open https://runner-cli.nexum.local")
    );
    assert_eq!(value["events_written"], Value::Number(3u64.into()));
}
