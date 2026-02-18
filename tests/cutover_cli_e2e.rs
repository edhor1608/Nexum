use std::process::Command;

use serde_json::Value;
use tempfile::tempdir;

#[test]
fn nexumctl_cutover_apply_updates_flags_when_allowed() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("flags.toml");
    let nexumctl = assert_cmd::cargo::cargo_bin!("nexumctl");

    let out = Command::new(nexumctl)
        .arg("cutover")
        .arg("apply")
        .arg("--file")
        .arg(&file)
        .arg("--capability")
        .arg("routing")
        .arg("--parity-score")
        .arg("0.99")
        .arg("--min-parity-score")
        .arg("0.95")
        .arg("--critical-events")
        .arg("0")
        .arg("--max-critical-events")
        .arg("0")
        .arg("--shadow-mode")
        .arg("true")
        .output()
        .unwrap();

    assert!(out.status.success());
    let decision: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(decision["allowed"], Value::Bool(true));

    let show = Command::new(nexumctl)
        .arg("flags")
        .arg("show")
        .arg("--file")
        .arg(&file)
        .output()
        .unwrap();

    assert!(show.status.success());
    let flags = String::from_utf8(show.stdout).unwrap();
    assert!(flags.contains("\"routing_control_plane\":true"));
}

#[test]
fn nexumctl_cutover_rollback_disables_capability_flag() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("flags.toml");
    let nexumctl = assert_cmd::cargo::cargo_bin!("nexumctl");

    let set = Command::new(nexumctl)
        .arg("flags")
        .arg("set")
        .arg("--file")
        .arg(&file)
        .arg("--routing")
        .arg("true")
        .arg("--restore")
        .arg("true")
        .output()
        .unwrap();
    assert!(set.status.success());

    let out = Command::new(nexumctl)
        .arg("cutover")
        .arg("rollback")
        .arg("--file")
        .arg(&file)
        .arg("--capability")
        .arg("routing")
        .output()
        .unwrap();
    assert!(out.status.success());
    let payload: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(payload["rolled_back"], Value::Bool(true));
    assert_eq!(
        payload["flag"],
        Value::String("routing_control_plane".into())
    );

    let show = Command::new(nexumctl)
        .arg("flags")
        .arg("show")
        .arg("--file")
        .arg(&file)
        .output()
        .unwrap();
    assert!(show.status.success());
    let flags: Value = serde_json::from_slice(&show.stdout).unwrap();
    assert_eq!(flags["routing_control_plane"], Value::Bool(false));
    assert_eq!(flags["restore_control_plane"], Value::Bool(true));
}
