use std::process::Command;

#[test]
fn nexumctl_can_allocate_and_release_capsule_ports() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("capsules.sqlite3");
    let nexumctl = assert_cmd::cargo::cargo_bin!("nexumctl");

    let created = Command::new(nexumctl)
        .arg("capsule")
        .arg("create")
        .arg("--db")
        .arg(&db)
        .arg("--id")
        .arg("cap-cli-port")
        .arg("--name")
        .arg("Port Capsule")
        .arg("--workspace")
        .arg("10")
        .arg("--mode")
        .arg("host_default")
        .output()
        .unwrap();
    assert!(created.status.success());

    let allocated = Command::new(nexumctl)
        .arg("capsule")
        .arg("allocate-port")
        .arg("--db")
        .arg(&db)
        .arg("--id")
        .arg("cap-cli-port")
        .arg("--start")
        .arg("6200")
        .arg("--end")
        .arg("6202")
        .output()
        .unwrap();
    assert!(allocated.status.success());
    let alloc_stdout = String::from_utf8(allocated.stdout).unwrap();
    assert!(alloc_stdout.contains("\"port\":6200"));

    let listed = Command::new(nexumctl)
        .arg("capsule")
        .arg("list")
        .arg("--db")
        .arg(&db)
        .output()
        .unwrap();
    assert!(listed.status.success());
    let list_stdout = String::from_utf8(listed.stdout).unwrap();
    assert!(list_stdout.contains("\"allocated_ports\":[6200]"));

    let released = Command::new(nexumctl)
        .arg("capsule")
        .arg("release-ports")
        .arg("--db")
        .arg(&db)
        .arg("--id")
        .arg("cap-cli-port")
        .output()
        .unwrap();
    assert!(released.status.success());
    let release_stdout = String::from_utf8(released.stdout).unwrap();
    assert!(release_stdout.contains("\"released\":1"));
}
