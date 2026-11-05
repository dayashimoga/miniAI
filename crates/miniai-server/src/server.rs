use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

use crate::error::ServerError;
use crate::ipc::{IpcBridge, PrivacyPolicyUpdatePayload};

/// Localhost-only HTTP and SSE IPC server protected by session authentication nonces.
pub struct IpcServer {
    bridge: IpcBridge,
    listener: TcpListener,
    local_addr: SocketAddr,
}

impl IpcServer {
    /// Binds an `IpcServer` strictly to a localhost loopback address (e.g., `127.0.0.1:0`).
    /// Rejects non-loopback bindings to prevent unintended network exposure.
    pub async fn bind(bridge: IpcBridge, bind_addr: &str) -> Result<Self, ServerError> {
        if !bind_addr.starts_with("127.0.0.1") && !bind_addr.starts_with("localhost") {
            return Err(ServerError::BadRequest {
                message: "MiniAI IPC Server must bind strictly to 127.0.0.1 localhost loopback"
                    .to_string(),
            });
        }

        let listener = TcpListener::bind(bind_addr)
            .await
            .map_err(|e| ServerError::IoError {
                message: format!("Failed to bind IPC TCP listener to '{bind_addr}': {e}"),
            })?;

        let local_addr = listener.local_addr().map_err(|e| ServerError::IoError {
            message: format!("Failed to retrieve local address: {e}"),
        })?;

        Ok(Self {
            bridge,
            listener,
            local_addr,
        })
    }

    /// Returns the local socket address this server is listening on.
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Spawns the server loop onto a background Tokio task.
    pub fn spawn(self) -> (SocketAddr, JoinHandle<()>) {
        let addr = self.local_addr;
        let handle = tokio::spawn(async move {
            let _ = self.run().await;
        });
        (addr, handle)
    }

    /// Runs the accept loop, serving incoming HTTP/1.1 and SSE connections.
    pub async fn run(self) -> Result<(), ServerError> {
        loop {
            let (stream, _) = self
                .listener
                .accept()
                .await
                .map_err(|e| ServerError::IoError {
                    message: format!("Accept error: {e}"),
                })?;

            let bridge = self.bridge.clone();
            tokio::spawn(async move {
                let _ = Self::handle_connection(stream, bridge).await;
            });
        }
    }

    /// Handles a single incoming TCP connection.
    async fn handle_connection(
        mut stream: TcpStream,
        bridge: IpcBridge,
    ) -> Result<(), ServerError> {
        let mut buffer = vec![0u8; 16384];
        let bytes_read = stream
            .read(&mut buffer)
            .await
            .map_err(|e| ServerError::IoError {
                message: e.to_string(),
            })?;

        if bytes_read == 0 {
            return Ok(());
        }

        let request_str = String::from_utf8_lossy(&buffer[..bytes_read]);
        let mut lines = request_str.lines();
        let request_line = match lines.next() {
            Some(line) => line,
            None => return Ok(()),
        };

        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap_or("GET");
        let path = parts.next().unwrap_or("/");

        // Collect HTTP headers
        let mut auth_header: Option<String> = None;
        let mut content_length: usize = 0;
        let mut body_start_index: Option<usize> = None;

        for line in lines {
            if line.is_empty() {
                // Header-body separator reached
                break;
            }
            let lower = line.to_ascii_lowercase();
            if lower.starts_with("authorization:") {
                auth_header = Some(line["authorization:".len()..].trim().to_string());
            } else if lower.starts_with("x-session-nonce:") {
                auth_header = Some(line["x-session-nonce:".len()..].trim().to_string());
            } else if lower.starts_with("content-length:") {
                if let Ok(len) = line["content-length:".len()..].trim().parse::<usize>() {
                    content_length = len;
                }
            }
        }

        // Locate start of body in buffer
        if let Some(pos) = request_str.find("\r\n\r\n") {
            body_start_index = Some(pos + 4);
        } else if let Some(pos) = request_str.find("\n\n") {
            body_start_index = Some(pos + 2);
        }

        let body = if let Some(start) = body_start_index {
            if start < bytes_read {
                let available = bytes_read - start;
                let take = available.min(content_length);
                request_str[start..start + take].to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // Authentication enforcement (all routes require nonce except /v1/health)
        if path != "/v1/health" {
            let candidate = auth_header.as_deref().unwrap_or("");
            if bridge.verify_nonce(candidate).is_err() {
                let err_json = serde_json::json!({
                    "error": "UNAUTHORIZED",
                    "message": "Missing or invalid session authentication nonce in Authorization header",
                });
                return Self::send_json_response(&mut stream, 401, "Unauthorized", &err_json).await;
            }
        }

        // Route matching
        match (method, path) {
            ("GET", "/v1/health") => {
                let health = bridge.get_health();
                Self::send_json_response(&mut stream, 200, "OK", &health).await
            }
            ("GET", "/v1/system/snapshot") => match bridge.get_system_snapshot().await {
                Ok(snapshot) => {
                    let val = serde_json::to_value(&snapshot).unwrap_or_default();
                    Self::send_json_response(&mut stream, 200, "OK", &val).await
                }
                Err(err) => {
                    let err_json = serde_json::json!({
                        "error": "INTERNAL_ERROR",
                        "message": format!("{err}"),
                    });
                    Self::send_json_response(&mut stream, 500, "Internal Server Error", &err_json)
                        .await
                }
            },
            ("GET", "/v1/privacy/policy") => {
                let policy = bridge.get_privacy_policy().await;
                let val = serde_json::to_value(&policy).unwrap_or_default();
                Self::send_json_response(&mut stream, 200, "OK", &val).await
            }
            ("POST", "/v1/privacy/policy") => {
                let update_payload: Result<PrivacyPolicyUpdatePayload, _> =
                    serde_json::from_str(&body);
                match update_payload {
                    Ok(payload) => {
                        let epoch_secs = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();

                        match bridge.update_privacy_policy(payload, epoch_secs).await {
                            Ok(new_state) => {
                                let val = serde_json::to_value(&new_state).unwrap_or_default();
                                Self::send_json_response(&mut stream, 200, "OK", &val).await
                            }
                            Err(err) => {
                                let err_json = serde_json::json!({
                                    "error": "BAD_REQUEST",
                                    "message": format!("{err}"),
                                });
                                Self::send_json_response(&mut stream, 400, "Bad Request", &err_json)
                                    .await
                            }
                        }
                    }
                    Err(e) => {
                        let err_json = serde_json::json!({
                            "error": "INVALID_JSON",
                            "message": format!("Failed to parse request JSON payload: {e}"),
                        });
                        Self::send_json_response(&mut stream, 400, "Bad Request", &err_json).await
                    }
                }
            }
            ("GET", "/v1/events/recent") => {
                let events = bridge.get_recent_events().await;
                let val = serde_json::to_value(&events).unwrap_or_default();
                Self::send_json_response(&mut stream, 200, "OK", &val).await
            }
            ("GET", "/v1/events/stream") => Self::handle_sse_stream(stream, bridge).await,
            _ => {
                let err_json = serde_json::json!({
                    "error": "NOT_FOUND",
                    "message": format!("Route not found: {method} {path}"),
                });
                Self::send_json_response(&mut stream, 404, "Not Found", &err_json).await
            }
        }
    }

    /// Sends a standard JSON response envelope.
    async fn send_json_response(
        stream: &mut TcpStream,
        status_code: u16,
        status_text: &str,
        body: &serde_json::Value,
    ) -> Result<(), ServerError> {
        let body_str = serde_json::to_string(body).unwrap_or_default();
        let response = format!(
            "HTTP/1.1 {status_code} {status_text}\r\n\
             Content-Type: application/json\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\
             \r\n\
             {}",
            body_str.len(),
            body_str
        );
        stream
            .write_all(response.as_bytes())
            .await
            .map_err(|e| ServerError::IoError {
                message: e.to_string(),
            })?;
        stream.flush().await.map_err(|e| ServerError::IoError {
            message: e.to_string(),
        })?;
        Ok(())
    }

    /// Handles live Server-Sent Events (SSE) streaming.
    async fn handle_sse_stream(
        mut stream: TcpStream,
        bridge: IpcBridge,
    ) -> Result<(), ServerError> {
        let header = "HTTP/1.1 200 OK\r\n\
                      Content-Type: text/event-stream\r\n\
                      Cache-Control: no-cache\r\n\
                      Connection: keep-alive\r\n\
                      \r\n";
        stream
            .write_all(header.as_bytes())
            .await
            .map_err(|e| ServerError::IoError {
                message: e.to_string(),
            })?;

        let mut subscriber = bridge.event_bus().subscribe();

        loop {
            match subscriber.recv().await {
                Ok(envelope) => {
                    let json = serde_json::to_string(&envelope).unwrap_or_default();
                    let sse_chunk = format!("data: {json}\n\n");
                    if stream.write_all(sse_chunk.as_bytes()).await.is_err() {
                        // Client disconnected
                        break;
                    }
                    let _ = stream.flush().await;
                }
                Err(crate::error::EventBusError::ChannelClosed) => break,
                Err(crate::error::EventBusError::Lagged { missed_events }) => {
                    let warning_chunk = format!(
                        "event: warning\ndata: {{\"lagged_missed_events\":{missed_events}}}\n\n"
                    );
                    if stream.write_all(warning_chunk.as_bytes()).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        Ok(())
    }
}
