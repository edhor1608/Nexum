use std::path::PathBuf;

use nexum::routing::{default_socket_path, serve_unix_socket};
use tokio::sync::oneshot;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let subcommand = args.next().unwrap_or_else(|| "serve".to_string());
    if subcommand == "--help" || subcommand == "-h" {
        usage();
        return Ok(());
    }

    if subcommand != "serve" {
        eprintln!("unsupported subcommand: {subcommand}");
        std::process::exit(2);
    }

    let mut socket = default_socket_path();
    while let Some(arg) = args.next() {
        if arg == "--help" || arg == "-h" {
            usage();
            return Ok(());
        }
        if arg == "--socket" {
            if let Some(path) = args.next() {
                socket = PathBuf::from(path);
                continue;
            }
            eprintln!("--socket requires a path");
            std::process::exit(2);
        }

        eprintln!("unknown arg: {arg}");
        std::process::exit(2);
    }

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let serve_socket = socket.clone();
    let mut serve_task =
        tokio::spawn(async move { serve_unix_socket(&serve_socket, shutdown_rx).await });

    tokio::select! {
        serve_result = &mut serve_task => {
            serve_result??;
            return Ok(());
        }
        signal_result = wait_for_shutdown_signal() => {
            signal_result?;
        }
    }

    let _ = shutdown_tx.send(());
    serve_task.await??;
    Ok(())
}

#[cfg(unix)]
async fn wait_for_shutdown_signal() -> Result<(), Box<dyn std::error::Error>> {
    use tokio::signal::unix::{SignalKind, signal};

    let mut terminate = signal(SignalKind::terminate())?;
    tokio::select! {
        result = tokio::signal::ctrl_c() => { result?; }
        _ = terminate.recv() => {}
    }
    Ok(())
}

#[cfg(not(unix))]
async fn wait_for_shutdown_signal() -> Result<(), Box<dyn std::error::Error>> {
    tokio::signal::ctrl_c().await?;
    Ok(())
}

fn usage() {
    println!("nexumd serve [--socket <path>]");
}
