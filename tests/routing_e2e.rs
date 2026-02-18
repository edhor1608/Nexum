use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    time::Duration,
};

use serde_json::json;
use tempfile::tempdir;

#[test]
fn daemon_binary_serves_json_over_unix_socket() {
    let dir = tempdir().unwrap();
    let socket = dir.path().join("nexumd.sock");

    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_nexumd"))
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

    let mut stream = UnixStream::connect(&socket).unwrap();
    stream
        .write_all(
            format!(
                "{}\n",
                json!({"cmd":"register","capsule_id":"cap-e2e","domain":"cap-e2e.nexum.local","upstream":"127.0.0.1:4500"})
            )
            .as_bytes(),
        )
        .unwrap();

    let mut line = String::new();
    BufReader::new(stream.try_clone().unwrap())
        .read_line(&mut line)
        .unwrap();
    assert!(line.contains("registered"));

    stream
        .write_all(format!("{}\n", json!({"cmd":"health"})).as_bytes())
        .unwrap();
    line.clear();
    BufReader::new(stream).read_line(&mut line).unwrap();
    assert!(line.contains("ok"));

    child.kill().unwrap();
    let _ = child.wait();
}
