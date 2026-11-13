//! Exemplo 07 (Avançado - Daemon IPC): Comunicação assíncrona via Unix Domain Socket (Ping, Status, Capture).
//!
//! Uso:
//!   cargo run -p hu20-cli --example 07_daemon_ipc

use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

struct DaemonGuard {
    child: Child,
    socket_path: std::path::PathBuf,
}

impl Drop for DaemonGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 07. Daemon IPC via Unix Domain Socket ===");

    let temp_dir = tempfile::tempdir()?;
    let socket_path = temp_dir.path().join("hu20_example.sock");
    let exe = std::env::current_exe()?
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("hu20ctl"))
        .ok_or("não foi possível localizar o binário hu20ctl")?;

    if !exe.exists() {
        return Err(format!(
            "binário {} não encontrado; execute `cargo build -p hu20-cli` antes",
            exe.display()
        )
        .into());
    }

    let child = Command::new(&exe)
        .args([
            "--mock",
            "match",
            "serve",
            "--socket-path",
            socket_path.to_str().unwrap(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    let _guard = DaemonGuard {
        child,
        socket_path: socket_path.clone(),
    };

    let deadline = Instant::now();
    while deadline.elapsed() < Duration::from_secs(3) {
        if socket_path.exists() && UnixStream::connect(&socket_path).await.is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(40)).await;
    }

    // 1. Ping
    let ping_resp = send_once(&socket_path, r#"{"command":"ping"}"#).await?;
    println!("1) Ping   -> {}", ping_resp.trim());

    // 2. Status
    let status_resp = send_once(&socket_path, r#"{"command":"status"}"#).await?;
    println!("2) Status -> {}", status_resp.trim());

    // 3. Capture streaming
    println!("3) Capture (streaming de eventos):");
    let stream = UnixStream::connect(&socket_path).await?;
    let (reader, mut writer) = stream.into_split();
    writer
        .write_all(b"{\"command\":\"capture\",\"timeout_ms\":2000,\"min_quality\":50}\n")
        .await?;
    writer.flush().await?;

    let mut lines = BufReader::new(reader).lines();
    while let Some(line) = lines.next_line().await? {
        let v: serde_json::Value = serde_json::from_str(&line)?;
        let event = v["event"].as_str().unwrap_or("unknown");
        if event == "completed" {
            println!(
                "   [completed] quality={} | {}x{} | sha256={}...",
                v["quality"],
                v["width"],
                v["height"],
                &v["sha256_raw_frame"].as_str().unwrap_or("")[..16]
            );
            break;
        } else {
            println!("   [{event}] {line}");
        }
    }

    Ok(())
}

async fn send_once(
    socket_path: &std::path::Path,
    payload: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let stream = UnixStream::connect(socket_path).await?;
    let (reader, mut writer) = stream.into_split();
    writer.write_all(payload.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    let mut lines = BufReader::new(reader).lines();
    Ok(lines.next_line().await?.unwrap_or_default())
}
