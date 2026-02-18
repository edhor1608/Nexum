use std::process::Command;

use tempfile::tempdir;

#[test]
fn nexumctl_can_set_and_show_cutover_flags() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("flags.toml");
    let nexumctl = assert_cmd::cargo::cargo_bin!("nexumctl");

    let status = Command::new(nexumctl)
        .arg("flags")
        .arg("set")
        .arg("--file")
        .arg(&file)
        .arg("--routing")
        .arg("true")
        .arg("--restore")
        .arg("true")
        .status()
        .unwrap();
    assert!(status.success());

    let output = Command::new(nexumctl)
        .arg("flags")
        .arg("show")
        .arg("--file")
        .arg(&file)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"shadow_mode\":true"));
    assert!(stdout.contains("\"routing_control_plane\":true"));
    assert!(stdout.contains("\"restore_control_plane\":true"));
}
