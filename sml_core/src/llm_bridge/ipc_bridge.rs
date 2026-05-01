use crate::parser::parse_sml_token;
use crate::executor::dispatch;
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;
use std::fs;

const SOCKET_PATH: &str = "/tmp/sml_ollama_bridge.sock";

pub async fn start_ipc_bridge() -> Result<(), Box<dyn std::error::Error>> {
    // Limpiar socket anterior si existe
    if Path::new(SOCKET_PATH).exists() {
        fs::remove_file(SOCKET_PATH)?;
    }

    let listener = UnixListener::bind(SOCKET_PATH)?;
    println!("[IPC Bridge] Rust Micro-Bridge listening safely on {}", SOCKET_PATH);

    loop {
        let (mut stream, _) = listener.accept().await?;
        
        tokio::spawn(async move {
            let mut buf = vec![0; 4096];
            match stream.read(&mut buf).await {
                Ok(n) if n > 0 => {
                    let request = String::from_utf8_lossy(&buf[..n]).to_string();
                    if request.starts_with("@[") {
                        if let Some(cmd) = parse_sml_token(&request) {
                            let result = dispatch(cmd).await;
                            let _ = stream.write_all(result.as_bytes()).await;
                        } else {
                            let _ = stream.write_all(b"[ERR:INVALID_SYNTAX]").await;
                        }
                    } else {
                        let _ = stream.write_all(b"[ERR:NOT_SML]").await;
                    }
                }
                _ => {}
            }
        });
    }
}
