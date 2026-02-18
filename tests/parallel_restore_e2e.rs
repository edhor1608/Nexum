use std::{
    process::Command,
    time::{Duration, Instant},
};

use serde_json::Value;
use tempfile::tempdir;

fn wait_for_socket(socket: &std::path::Path) {
    for _ in 0..40 {
        if socket.exists() {
            return;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!("socket not ready: {}", socket.display());
}

#[test]
fn five_parallel_capsules_restore_and_register_routes() {
    let dir = tempdir().unwrap();
    let socket = dir.path().join("nexumd.sock");

    let mut daemon = Command::new(assert_cmd::cargo::cargo_bin!("nexumd"))
        .arg("serve")
        .arg("--socket")
        .arg(&socket)
        .spawn()
        .unwrap();
    wait_for_socket(&socket);

    let nexumctl = assert_cmd::cargo::cargo_bin!("nexumctl");
    let mut handles = Vec::new();

    for idx in 0..5 {
        let socket = socket.clone();
        let nexumctl = nexumctl.to_path_buf();
        let tls_dir = dir.path().join(format!("tls-{idx}"));
        let events_db = dir.path().join(format!("events-{idx}.sqlite3"));

        handles.push(std::thread::spawn(move || {
            Command::new(nexumctl)
                .arg("run")
                .arg("restore")
                .arg("--capsule-id")
                .arg(format!("cap-parallel-{idx}"))
                .arg("--name")
                .arg(format!("Parallel Capsule {idx}"))
                .arg("--workspace")
                .arg(format!("{}", idx + 1))
                .arg("--signal")
                .arg("needs_decision")
                .arg("--terminal")
                .arg(format!("cd /workspace/parallel-{idx} && nix develop"))
                .arg("--editor")
                .arg(format!("/workspace/parallel-{idx}"))
                .arg("--browser")
                .arg(format!("https://parallel-capsule-{idx}.nexum.local"))
                .arg("--upstream")
                .arg(format!("127.0.0.1:48{}0", idx))
                .arg("--routing-socket")
                .arg(&socket)
                .arg("--tls-dir")
                .arg(&tls_dir)
                .arg("--events-db")
                .arg(&events_db)
                .output()
                .unwrap()
        }));
    }

    for handle in handles {
        let output = handle.join().unwrap();
        assert!(output.status.success(), "{:?}", output);
    }

    let list = Command::new(nexumctl)
        .arg("routing")
        .arg("list")
        .arg("--socket")
        .arg(&socket)
        .output()
        .unwrap();
    assert!(list.status.success());
    let json: Value = serde_json::from_slice(&list.stdout).unwrap();
    assert_eq!(json["kind"], Value::String("listed".into()));

    let routes = json["routes"].as_array().unwrap();
    assert_eq!(routes.len(), 5);
    assert_eq!(
        routes
            .iter()
            .map(|route| route["domain"].as_str().unwrap().to_string())
            .collect::<Vec<_>>(),
        vec![
            "parallel-capsule-0.nexum.local".to_string(),
            "parallel-capsule-1.nexum.local".to_string(),
            "parallel-capsule-2.nexum.local".to_string(),
            "parallel-capsule-3.nexum.local".to_string(),
            "parallel-capsule-4.nexum.local".to_string(),
        ]
    );

    daemon.kill().unwrap();
    let _ = daemon.wait();
}

#[test]
fn restore_command_completes_under_ten_seconds() {
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
    wait_for_socket(&socket);

    let nexumctl = assert_cmd::cargo::cargo_bin!("nexumctl");
    let started = Instant::now();
    let output = Command::new(nexumctl)
        .arg("run")
        .arg("restore")
        .arg("--capsule-id")
        .arg("cap-budget")
        .arg("--name")
        .arg("Budget Capsule")
        .arg("--workspace")
        .arg("9")
        .arg("--signal")
        .arg("needs_decision")
        .arg("--terminal")
        .arg("cd /workspace/budget && nix develop")
        .arg("--editor")
        .arg("/workspace/budget")
        .arg("--browser")
        .arg("https://budget-capsule.nexum.local")
        .arg("--upstream")
        .arg("127.0.0.1:4890")
        .arg("--routing-socket")
        .arg(&socket)
        .arg("--tls-dir")
        .arg(&tls_dir)
        .arg("--events-db")
        .arg(&events_db)
        .output()
        .unwrap();
    let elapsed = started.elapsed();

    assert!(output.status.success());
    assert!(elapsed < Duration::from_secs(10), "elapsed: {:?}", elapsed);

    daemon.kill().unwrap();
    let _ = daemon.wait();
}
