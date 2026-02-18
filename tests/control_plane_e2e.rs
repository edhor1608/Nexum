use std::process::Command;

use tempfile::tempdir;

#[test]
fn nexumctl_can_create_and_list_capsules_via_sqlite_store() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("capsules.sqlite3");
    let nexumctl = assert_cmd::cargo::cargo_bin!("nexumctl");

    let status = Command::new(nexumctl)
        .arg("capsule")
        .arg("create")
        .arg("--db")
        .arg(&db)
        .arg("--id")
        .arg("cap-cli-1")
        .arg("--name")
        .arg("Payments API")
        .arg("--workspace")
        .arg("6")
        .arg("--mode")
        .arg("host_default")
        .status()
        .unwrap();
    assert!(status.success());

    let output = Command::new(nexumctl)
        .arg("capsule")
        .arg("list")
        .arg("--db")
        .arg(&db)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("cap-cli-1"));
    assert!(stdout.contains("payments-api.nexum.local"));
}
