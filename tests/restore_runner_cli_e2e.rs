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
    assert!(
        value["shell_script"]
            .as_str()
            .unwrap()
            .contains("export NEXUM_CAPSULE_ID=cap-run-cli")
    );
    assert!(
        value["shell_script"]
            .as_str()
            .unwrap()
            .contains("export NEXUM_PROCESS_LABEL=nexum-terminal-cap-run-cli")
    );
    assert_eq!(value["run_mode"], Value::String("host_default".into()));
    assert_eq!(value["degraded"], Value::Bool(false));
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
    assert_eq!(
        value["run_mode"],
        Value::String("isolated_nix_shell".into())
    );

    daemon.kill().unwrap();
    let _ = daemon.wait();
}

#[test]
fn nexumctl_run_restore_escalates_to_isolated_mode_for_high_risk_secret_workflow() {
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
        .arg("cap-run-cli-secret")
        .arg("--name")
        .arg("Runner Secret")
        .arg("--workspace")
        .arg("11")
        .arg("--signal")
        .arg("needs_decision")
        .arg("--terminal")
        .arg("cd /workspace/secret && nix develop")
        .arg("--editor")
        .arg("/workspace/secret")
        .arg("--browser")
        .arg("https://runner-secret.nexum.local")
        .arg("--upstream")
        .arg("127.0.0.1:4760")
        .arg("--routing-socket")
        .arg(&socket)
        .arg("--high-risk-secret")
        .arg("true")
        .arg("--tls-dir")
        .arg(&tls_dir)
        .arg("--events-db")
        .arg(&events_db)
        .output()
        .unwrap();
    assert!(out.status.success());

    let value: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        value["run_mode"],
        Value::String("isolated_nix_shell".into())
    );

    daemon.kill().unwrap();
    let _ = daemon.wait();
}

#[test]
fn nexumctl_run_restore_returns_degraded_summary_when_routing_socket_is_unavailable() {
    let dir = tempdir().unwrap();
    let missing_socket = dir.path().join("missing.sock");
    let tls_dir = dir.path().join("tls");
    let events_db = dir.path().join("events.sqlite3");

    let nexumctl = assert_cmd::cargo::cargo_bin!("nexumctl");
    let out = Command::new(nexumctl)
        .arg("run")
        .arg("restore")
        .arg("--capsule-id")
        .arg("cap-run-cli-degraded")
        .arg("--name")
        .arg("Runner Degraded")
        .arg("--workspace")
        .arg("12")
        .arg("--signal")
        .arg("needs_decision")
        .arg("--terminal")
        .arg("cd /workspace/degraded && nix develop")
        .arg("--editor")
        .arg("/workspace/degraded")
        .arg("--browser")
        .arg("https://runner-degraded.nexum.local")
        .arg("--upstream")
        .arg("127.0.0.1:4765")
        .arg("--routing-socket")
        .arg(&missing_socket)
        .arg("--tls-dir")
        .arg(&tls_dir)
        .arg("--events-db")
        .arg(&events_db)
        .output()
        .unwrap();
    assert!(out.status.success());

    let value: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(value["degraded"], Value::Bool(true));
    assert!(
        value["degraded_reason"]
            .as_str()
            .unwrap()
            .contains("route_unavailable")
    );
}

#[test]
fn nexumctl_run_restore_marks_capsule_ready_in_store_on_success() {
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
        .arg("cap-state-ready")
        .arg("--name")
        .arg("Capsule State Ready")
        .arg("--workspace")
        .arg("14")
        .arg("--mode")
        .arg("host_default")
        .output()
        .unwrap();
    assert!(create.status.success());

    let set_archived = Command::new(nexumctl)
        .arg("capsule")
        .arg("set-state")
        .arg("--db")
        .arg(&capsule_db)
        .arg("--id")
        .arg("cap-state-ready")
        .arg("--state")
        .arg("archived")
        .output()
        .unwrap();
    assert!(set_archived.status.success());

    let restore = Command::new(nexumctl)
        .arg("run")
        .arg("restore")
        .arg("--capsule-id")
        .arg("cap-state-ready")
        .arg("--name")
        .arg("Capsule State Ready")
        .arg("--workspace")
        .arg("14")
        .arg("--signal")
        .arg("needs_decision")
        .arg("--terminal")
        .arg("cd /workspace/state-ready && nix develop")
        .arg("--editor")
        .arg("/workspace/state-ready")
        .arg("--browser")
        .arg("https://state-ready.nexum.local")
        .arg("--upstream")
        .arg("127.0.0.1:4770")
        .arg("--capsule-db")
        .arg(&capsule_db)
        .arg("--tls-dir")
        .arg(&tls_dir)
        .arg("--events-db")
        .arg(&events_db)
        .output()
        .unwrap();
    assert!(restore.status.success());

    let list = Command::new(nexumctl)
        .arg("capsule")
        .arg("list")
        .arg("--db")
        .arg(&capsule_db)
        .output()
        .unwrap();
    assert!(list.status.success());
    let payload: Value = serde_json::from_slice(&list.stdout).unwrap();
    assert_eq!(
        payload[0]["capsule_id"],
        Value::String("cap-state-ready".into())
    );
    assert_eq!(payload[0]["state"], Value::String("ready".into()));
}

#[test]
fn nexumctl_run_restore_marks_capsule_degraded_in_store_on_route_unavailable() {
    let dir = tempdir().unwrap();
    let capsule_db = dir.path().join("capsules.sqlite3");
    let missing_socket = dir.path().join("missing.sock");
    let tls_dir = dir.path().join("tls");
    let events_db = dir.path().join("events.sqlite3");
    let nexumctl = assert_cmd::cargo::cargo_bin!("nexumctl");

    let create = Command::new(nexumctl)
        .arg("capsule")
        .arg("create")
        .arg("--db")
        .arg(&capsule_db)
        .arg("--id")
        .arg("cap-state-degraded")
        .arg("--name")
        .arg("Capsule State Degraded")
        .arg("--workspace")
        .arg("15")
        .arg("--mode")
        .arg("host_default")
        .output()
        .unwrap();
    assert!(create.status.success());

    let restore = Command::new(nexumctl)
        .arg("run")
        .arg("restore")
        .arg("--capsule-id")
        .arg("cap-state-degraded")
        .arg("--name")
        .arg("Capsule State Degraded")
        .arg("--workspace")
        .arg("15")
        .arg("--signal")
        .arg("needs_decision")
        .arg("--terminal")
        .arg("cd /workspace/state-degraded && nix develop")
        .arg("--editor")
        .arg("/workspace/state-degraded")
        .arg("--browser")
        .arg("https://state-degraded.nexum.local")
        .arg("--upstream")
        .arg("127.0.0.1:4771")
        .arg("--routing-socket")
        .arg(&missing_socket)
        .arg("--capsule-db")
        .arg(&capsule_db)
        .arg("--tls-dir")
        .arg(&tls_dir)
        .arg("--events-db")
        .arg(&events_db)
        .output()
        .unwrap();
    assert!(restore.status.success());

    let list = Command::new(nexumctl)
        .arg("capsule")
        .arg("list")
        .arg("--db")
        .arg(&capsule_db)
        .output()
        .unwrap();
    assert!(list.status.success());
    let payload: Value = serde_json::from_slice(&list.stdout).unwrap();
    assert_eq!(
        payload[0]["capsule_id"],
        Value::String("cap-state-degraded".into())
    );
    assert_eq!(payload[0]["state"], Value::String("degraded".into()));
}
