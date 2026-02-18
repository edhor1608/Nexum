use std::time::Duration;

use nexum::routing::{RouteCommand, RouteOutcome, send_command, serve_unix_socket};
use tempfile::tempdir;
use tokio::{sync::oneshot, task::JoinHandle};

async fn start_server(socket: &std::path::Path) -> (oneshot::Sender<()>, JoinHandle<()>) {
    let (tx, rx) = oneshot::channel();
    let socket_path = socket.to_path_buf();
    let task_socket = socket_path.clone();
    let handle = tokio::spawn(async move {
        serve_unix_socket(&task_socket, rx)
            .await
            .expect("server should run");
    });

    for _ in 0..20 {
        if socket_path.exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }

    (tx, handle)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn socket_api_supports_health_register_resolve_remove() {
    let dir = tempdir().unwrap();
    let socket = dir.path().join("nexumd.sock");
    let (shutdown_tx, handle) = start_server(&socket).await;

    let health = send_command(&socket, RouteCommand::Health).await.unwrap();
    assert!(matches!(health, RouteOutcome::Health { status } if status == "ok"));

    let register = send_command(
        &socket,
        RouteCommand::Register {
            capsule_id: "cap-int".into(),
            domain: "cap-int.nexum.local".into(),
            upstream: "127.0.0.1:4400".into(),
        },
    )
    .await
    .unwrap();
    assert!(matches!(register, RouteOutcome::Registered { .. }));

    let resolved = send_command(
        &socket,
        RouteCommand::Resolve {
            domain: "cap-int.nexum.local".into(),
        },
    )
    .await
    .unwrap();
    assert!(matches!(
        resolved,
        RouteOutcome::Resolved { route: Some(_) }
    ));

    let removed = send_command(
        &socket,
        RouteCommand::Remove {
            domain: "cap-int.nexum.local".into(),
        },
    )
    .await
    .unwrap();
    assert!(matches!(removed, RouteOutcome::Removed { removed: true }));

    let _ = shutdown_tx.send(());
    handle.await.unwrap();
}
