use std::{process::Command, time::Duration};

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

#[test]
fn nexumctl_run_restore_registers_route_via_daemon_socket() {
    let dir = tempdir().unwrap();
    let socket = dir.path().join("nexumd.sock");
    let tls_dir = dir.path().join("tls");
    let events_db = dir.path().join("events.sqlite3");

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
    let out = Command::new(nexumctl)
        .arg("run")
        .arg("restore")
        .arg("--capsule-id")
        .arg("cap-run-cli-daemon")
        .arg("--name")
        .arg("Runner CLI Daemon")
        .arg("--workspace")
        .arg("7")
        .arg("--signal")
        .arg("needs_decision")
        .arg("--terminal")
        .arg("cd /workspace/cli-daemon && nix develop")
        .arg("--editor")
        .arg("/workspace/cli-daemon")
        .arg("--browser")
        .arg("https://runner-cli-daemon.nexum.local")
        .arg("--upstream")
        .arg("127.0.0.1:4740")
        .arg("--routing-socket")
        .arg(&socket)
        .arg("--tls-dir")
        .arg(&tls_dir)
        .arg("--events-db")
        .arg(&events_db)
        .output()
        .unwrap();
    assert!(out.status.success());

    let resolve = Command::new(nexumctl)
        .arg("routing")
        .arg("resolve")
        .arg("--socket")
        .arg(&socket)
        .arg("--domain")
        .arg("runner-cli-daemon.nexum.local")
        .output()
        .unwrap();
    assert!(resolve.status.success());
    let resolved: Value = serde_json::from_slice(&resolve.stdout).unwrap();
    assert_eq!(resolved["kind"], Value::String("resolved".into()));
    assert_eq!(
        resolved["route"]["upstream"],
        Value::String("127.0.0.1:4740".into())
    );

    daemon.kill().unwrap();
    let _ = daemon.wait();
}

#[test]
fn nexumctl_run_restore_uses_profile_fallback_when_identity_collision_flag_is_set() {
    let dir = tempdir().unwrap();
    let socket = dir.path().join("nexumd.sock");
    let tls_dir = dir.path().join("tls");
    let events_db = dir.path().join("events.sqlite3");

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
    let out = Command::new(nexumctl)
        .arg("run")
        .arg("restore")
        .arg("--capsule-id")
        .arg("cap-run-cli-collision")
        .arg("--name")
        .arg("Runner Collision")
        .arg("--workspace")
        .arg("3")
        .arg("--signal")
        .arg("needs_decision")
        .arg("--terminal")
        .arg("cd /workspace/collision && nix develop")
        .arg("--editor")
        .arg("/workspace/collision")
        .arg("--browser")
        .arg("https://runner-collision.nexum.local")
        .arg("--upstream")
        .arg("127.0.0.1:4750")
        .arg("--routing-socket")
        .arg(&socket)
        .arg("--identity-collision")
        .arg("true")
        .arg("--tls-dir")
        .arg(&tls_dir)
        .arg("--events-db")
        .arg(&events_db)
        .output()
        .unwrap();
    assert!(out.status.success());

    let value: Value = serde_json::from_slice(&out.stdout).unwrap();
    let script = value["shell_script"].as_str().unwrap();
    assert!(script.contains("firefox --profile"));
    assert!(script.contains("runner-collision.nexum.local"));

    daemon.kill().unwrap();
    let _ = daemon.wait();
}
