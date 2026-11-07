use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

use miniai_bridge::BridgeEngine;

#[derive(Debug, Error)]
pub enum DaemonError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid bind address: {0} (daemon must bind strictly to 127.0.0.1 loopback)")]
    InvalidAddress(String),

    #[error("Bridge error: {0}")]
    Bridge(#[from] miniai_bridge::BridgeError),
}

/// Localhost-only HTTP daemon enabling Web and local IPC clients to communicate with MiniAI Rust core.
pub struct DaemonServer {
    listener: TcpListener,
    local_addr: SocketAddr,
    engine: Arc<BridgeEngine>,
}

impl DaemonServer {
    /// Binds the daemon server strictly to a localhost address (e.g. `127.0.0.1:4140` or `127.0.0.1:0`).
    pub async fn bind(bind_addr: &str, storage_path: Option<&Path>) -> Result<Self, DaemonError> {
        if !bind_addr.starts_with("127.0.0.1") && !bind_addr.starts_with("localhost") {
            return Err(DaemonError::InvalidAddress(bind_addr.to_string()));
        }

        let listener = TcpListener::bind(bind_addr).await?;
        let local_addr = listener.local_addr()?;

        if !local_addr.ip().is_loopback() {
            return Err(DaemonError::InvalidAddress(local_addr.to_string()));
        }

        let engine = match storage_path {
            Some(path) => Arc::new(BridgeEngine::open(path)?),
            None => Arc::new(BridgeEngine::new_in_memory()?),
        };

        Ok(Self {
            listener,
            local_addr,
            engine,
        })
    }

    /// Returns the local address the daemon is bound to.
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Spawns the daemon accept loop onto a background Tokio task.
    pub fn spawn(self) -> (SocketAddr, JoinHandle<()>) {
        let addr = self.local_addr;
        let handle = tokio::spawn(async move {
            let _ = self.run().await;
        });
        (addr, handle)
    }

    /// Runs the accept loop for the daemon.
    pub async fn run(self) -> Result<(), DaemonError> {
        loop {
            let (stream, _) = self.listener.accept().await?;
            let engine = self.engine.clone();
            tokio::spawn(async move {
                let _ = Self::handle_connection(stream, engine).await;
            });
        }
    }

    async fn handle_connection(
        mut stream: TcpStream,
        engine: Arc<BridgeEngine>,
    ) -> Result<(), DaemonError> {
        let mut buffer = vec![0u8; 16384];
        let bytes_read = stream.read(&mut buffer).await?;
        if bytes_read == 0 {
            return Ok(());
        }

        let request_str = String::from_utf8_lossy(&buffer[..bytes_read]);
        let mut lines = request_str.lines();
        let request_line = match lines.next() {
            Some(l) => l,
            None => return Ok(()),
        };

        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap_or("GET");
        let path = parts.next().unwrap_or("/");

        // CORS Preflight
        if method == "OPTIONS" {
            let response = "HTTP/1.1 204 No Content\r\n\
                            Access-Control-Allow-Origin: *\r\n\
                            Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
                            Access-Control-Allow-Headers: Authorization, Content-Type, X-Session-Nonce\r\n\
                            Connection: close\r\n\
                            \r\n";
            stream.write_all(response.as_bytes()).await?;
            return Ok(());
        }

        // Parse body if present
        let mut content_length = 0;
        for line in lines {
            let lower = line.to_ascii_lowercase();
            if lower.starts_with("content-length:") {
                if let Ok(len) = line["content-length:".len()..].trim().parse::<usize>() {
                    content_length = len;
                }
            }
        }

        let mut body = String::new();
        if let Some(pos) = request_str.find("\r\n\r\n") {
            let start = pos + 4;
            if start < bytes_read {
                let available = bytes_read - start;
                let take = available.min(content_length);
                body = request_str[start..start + take].to_string();
            }
        }

        match (method, path) {
            ("GET", "/v1/health") => {
                let resp_body = serde_json::json!({
                    "status": "ok",
                    "daemon": "miniai-daemon",
                    "version": "0.1.0"
                })
                .to_string();
                Self::send_response(&mut stream, 200, "OK", &resp_body).await?;
            }

            ("POST", "/v1/bridge/dispatch") => {
                let resp_body = engine.dispatch_json(&body);
                Self::send_response(&mut stream, 200, "OK", &resp_body).await?;
            }

            _ => {
                let err_body = serde_json::json!({
                    "error": "NOT_FOUND",
                    "message": format!("Route not found: {} {}", method, path)
                })
                .to_string();
                Self::send_response(&mut stream, 404, "Not Found", &err_body).await?;
            }
        }

        Ok(())
    }

    async fn send_response(
        stream: &mut TcpStream,
        status: u16,
        status_text: &str,
        body: &str,
    ) -> Result<(), DaemonError> {
        let response = format!(
            "HTTP/1.1 {status} {status_text}\r\n\
             Content-Type: application/json\r\n\
             Access-Control-Allow-Origin: *\r\n\
             Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
             Access-Control-Allow-Headers: Authorization, Content-Type, X-Session-Nonce\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\
             \r\n\
             {}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).await?;
        Ok(())
    }
}
