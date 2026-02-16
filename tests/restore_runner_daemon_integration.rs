use std::{process::Command, time::Duration};

use nexum::{
    restore::SignalType,
    routing::{RouteCommand, RouteOutcome, send_command},
    runflow::{RestoreRunInput, run_restore_flow},
};
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

fn routing_call(socket: &std::path::Path, command: RouteCommand) -> RouteOutcome {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()
        .unwrap();
    runtime.block_on(send_command(socket, command)).unwrap()
}

#[test]
fn run_restore_flow_registers_route_via_daemon_socket() {
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

    let summary = run_restore_flow(RestoreRunInput {
        capsule_id: "cap-daemon-run".into(),
        display_name: "Runner Daemon".into(),
        workspace: 4,
        signal: SignalType::NeedsDecision,
        terminal_cmd: "cd /workspace/daemon && nix develop".into(),
        editor_target: "/workspace/daemon".into(),
        browser_url: "https://runner-daemon.nexum.local".into(),
        route_upstream: "127.0.0.1:4780".into(),
        routing_socket: Some(socket.clone()),
        identity_collision: false,
        high_risk_secret_workflow: false,
        force_isolated_mode: false,
        capsule_db: None,
        tls_dir,
        events_db,
    })
    .unwrap();

    assert_eq!(summary.domain, "runner-daemon.nexum.local");

    let resolved = routing_call(
        &socket,
        RouteCommand::Resolve {
            domain: "runner-daemon.nexum.local".into(),
        },
    );
    match resolved {
        RouteOutcome::Resolved { route: Some(route) } => {
            assert_eq!(route.capsule_id, "cap-daemon-run");
            assert_eq!(route.upstream, "127.0.0.1:4780");
        }
        other => panic!("unexpected outcome: {:?}", other),
    }

    daemon.kill().unwrap();
    let _ = daemon.wait();
}

#[test]
fn run_restore_flow_returns_error_when_daemon_reports_domain_conflict() {
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

    let register = routing_call(
        &socket,
        RouteCommand::Register {
            capsule_id: "cap-other".into(),
            domain: "conflict-domain.nexum.local".into(),
            upstream: "127.0.0.1:4790".into(),
        },
    );
    assert!(matches!(register, RouteOutcome::Registered { .. }));

    let error = run_restore_flow(RestoreRunInput {
        capsule_id: "cap-conflict".into(),
        display_name: "Conflict Domain".into(),
        workspace: 5,
        signal: SignalType::NeedsDecision,
        terminal_cmd: "cd /workspace/conflict && nix develop".into(),
        editor_target: "/workspace/conflict".into(),
        browser_url: "https://conflict-domain.nexum.local".into(),
        route_upstream: "127.0.0.1:4791".into(),
        routing_socket: Some(socket.clone()),
        identity_collision: false,
        high_risk_secret_workflow: false,
        force_isolated_mode: false,
        capsule_db: None,
        tls_dir,
        events_db,
    })
    .unwrap_err();

    assert!(error.to_string().contains("domain"));
    assert!(error.to_string().contains("claimed"));

    daemon.kill().unwrap();
    let _ = daemon.wait();
}
