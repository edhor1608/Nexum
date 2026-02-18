use std::process::Command;

use serde_json::Value;
use tempfile::tempdir;

#[test]
fn nexumctl_tls_ensure_and_rotate_work() {
    let dir = tempdir().unwrap();
    let nexumctl = assert_cmd::cargo::cargo_bin!("nexumctl");

    let ensure_1 = Command::new(nexumctl)
        .arg("tls")
        .arg("ensure")
        .arg("--dir")
        .arg(dir.path())
        .arg("--domain")
        .arg("gateway.nexum.local")
        .arg("--validity-days")
        .arg("1")
        .output()
        .unwrap();
    assert!(ensure_1.status.success());
    let one: Value = serde_json::from_slice(&ensure_1.stdout).unwrap();

    let ensure_2 = Command::new(nexumctl)
        .arg("tls")
        .arg("ensure")
        .arg("--dir")
        .arg(dir.path())
        .arg("--domain")
        .arg("gateway.nexum.local")
        .arg("--validity-days")
        .arg("1")
        .output()
        .unwrap();
    assert!(ensure_2.status.success());
    let two: Value = serde_json::from_slice(&ensure_2.stdout).unwrap();

    assert_eq!(one["fingerprint_sha256"], two["fingerprint_sha256"]);

    let rotate = Command::new(nexumctl)
        .arg("tls")
        .arg("rotate")
        .arg("--dir")
        .arg(dir.path())
        .arg("--domain")
        .arg("gateway.nexum.local")
        .arg("--threshold-days")
        .arg("2")
        .output()
        .unwrap();
    assert!(rotate.status.success());
    let rotated: Value = serde_json::from_slice(&rotate.stdout).unwrap();

    assert_eq!(rotated["rotated"], Value::Bool(true));
    assert_ne!(
        one["fingerprint_sha256"],
        rotated["record"]["fingerprint_sha256"]
    );
}
