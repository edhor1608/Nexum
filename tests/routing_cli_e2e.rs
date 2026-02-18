use std::{process::Command, time::Duration};

use serde_json::Value;
use tempfile::tempdir;

#[test]
fn nexumctl_routing_health_register_resolve_remove() {
    let dir = tempdir().unwrap();
    let socket = dir.path().join("nexumd.sock");

    let mut daemon = Command::new(assert_cmd::cargo::cargo_bin!("nexumd"))
        .arg("serve")
        .arg("--socket")
        .arg(&socket)
        .spawn()
        .unwrap();

    for _ in 0..40 {
        if socket.exists() {
            break;
        }
        std::thread::sleep(Duration::from_millis(25));
    }

    let nexumctl = assert_cmd::cargo::cargo_bin!("nexumctl");

    let health = Command::new(nexumctl)
        .arg("routing")
        .arg("health")
        .arg("--socket")
        .arg(&socket)
        .output()
        .unwrap();
    assert!(health.status.success());
    let health_json: Value = serde_json::from_slice(&health.stdout).unwrap();
    assert_eq!(health_json["kind"], Value::String("health".into()));

    let register = Command::new(nexumctl)
        .arg("routing")
        .arg("register")
        .arg("--socket")
        .arg(&socket)
        .arg("--capsule-id")
        .arg("cap-route-cli")
        .arg("--domain")
        .arg("route-cli.nexum.local")
        .arg("--upstream")
        .arg("127.0.0.1:4800")
        .output()
        .unwrap();
    assert!(register.status.success());

    let resolve = Command::new(nexumctl)
        .arg("routing")
        .arg("resolve")
        .arg("--socket")
        .arg(&socket)
        .arg("--domain")
        .arg("route-cli.nexum.local")
        .output()
        .unwrap();
    assert!(resolve.status.success());
    let resolve_json: Value = serde_json::from_slice(&resolve.stdout).unwrap();
    assert_eq!(resolve_json["kind"], Value::String("resolved".into()));
    assert_eq!(
        resolve_json["route"]["upstream"],
        Value::String("127.0.0.1:4800".into())
    );

    let remove = Command::new(nexumctl)
        .arg("routing")
        .arg("remove")
        .arg("--socket")
        .arg(&socket)
        .arg("--domain")
        .arg("route-cli.nexum.local")
        .output()
        .unwrap();
    assert!(remove.status.success());

    daemon.kill().unwrap();
    let _ = daemon.wait();
}
