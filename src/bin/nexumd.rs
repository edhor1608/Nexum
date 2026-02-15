use std::path::PathBuf;

use nexum::routing::{default_socket_path, serve_unix_socket};
use tokio::sync::oneshot;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let subcommand = args.next().unwrap_or_else(|| "serve".to_string());

    if subcommand != "serve" {
        eprintln!("unsupported subcommand: {subcommand}");
        std::process::exit(2);
    }

    let mut socket = default_socket_path();
    while let Some(arg) = args.next() {
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

    let (_tx, rx) = oneshot::channel::<()>();
    serve_unix_socket(&socket, rx).await?;
    Ok(())
}
