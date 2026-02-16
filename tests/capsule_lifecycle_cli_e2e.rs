use std::process::Command;

#[test]
fn nexumctl_can_rename_and_transition_capsule_state() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("capsules.sqlite3");
    let nexumctl = assert_cmd::cargo::cargo_bin!("nexumctl");

    let created = Command::new(nexumctl)
        .arg("capsule")
        .arg("create")
        .arg("--db")
        .arg(&db)
        .arg("--id")
        .arg("cap-cli-state")
        .arg("--name")
        .arg("Ops Core")
        .arg("--workspace")
        .arg("4")
        .arg("--mode")
        .arg("host_default")
        .output()
        .unwrap();
    assert!(created.status.success());

    let renamed = Command::new(nexumctl)
        .arg("capsule")
        .arg("rename")
        .arg("--db")
        .arg(&db)
        .arg("--id")
        .arg("cap-cli-state")
        .arg("--name")
        .arg("Ops Core V2")
        .output()
        .unwrap();
    assert!(renamed.status.success());

    let transitioned = Command::new(nexumctl)
        .arg("capsule")
        .arg("set-state")
        .arg("--db")
        .arg(&db)
        .arg("--id")
        .arg("cap-cli-state")
        .arg("--state")
        .arg("degraded")
        .output()
        .unwrap();
    assert!(transitioned.status.success());

    let listed = Command::new(nexumctl)
        .arg("capsule")
        .arg("list")
        .arg("--db")
        .arg(&db)
        .output()
        .unwrap();
    assert!(listed.status.success());

    let stdout = String::from_utf8(listed.stdout).unwrap();
    assert!(stdout.contains("Ops Core V2"));
    assert!(stdout.contains("\"state\":\"degraded\""));
}
