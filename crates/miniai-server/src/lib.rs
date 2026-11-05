#![forbid(unsafe_code)]
//! # MiniAI Server & Event Bus (`miniai-server`)
//!
//! Strongly typed asynchronous event bus, system telemetry streaming, bounded backpressure
//! queues, in-process IPC bridge, and authenticated localhost loopback API server for MiniAI.

pub mod bus;
pub mod error;
pub mod events;
pub mod ipc;
pub mod server;

pub use bus::{EventBus, EventSubscriber};
pub use error::{EventBusError, ServerError};
pub use events::{EventEnvelope, SystemEvent};
pub use ipc::{IpcBridge, PrivacyPolicyUpdatePayload};
pub use server::IpcServer;

#[cfg(test)]
mod tests {
    use super::*;
    use miniai_core::{AcceleratorBackend, HardwareProfile, PrivacyPolicyState};
    use miniai_storage::{DiagnosticEventRecord, StorageEngine};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    #[test]
    fn test_system_event_types_and_json_serialization() {
        let events = vec![
            SystemEvent::HardwareChanged {
                profile: HardwareProfile {
                    os_family: "windows".to_string(),
                    arch: "x86_64".to_string(),
                    logical_cpu_cores: 8,
                    total_memory_bytes: 16_000,
                    available_memory_bytes: 8_000,
                    detected_backends: vec![AcceleratorBackend::Cpu],
                },
            },
            SystemEvent::PrivacyPolicyUpdated {
                state: PrivacyPolicyState {
                    local_only_mode: true,
                    force_cpu_only: false,
                    active_consent: None,
                },
            },
            SystemEvent::DiagnosticAlert {
                record: DiagnosticEventRecord {
                    id: Some(1),
                    event_type: "ALERT".to_string(),
                    severity: "WARN".to_string(),
                    source: "test".to_string(),
                    message: "Diagnostic alert".to_string(),
                    payload_json: None,
                    created_at_epoch_secs: 1000,
                },
            },
            SystemEvent::ModelDownloadProgress {
                model_id: "tiny-llama".to_string(),
                bytes_downloaded: 50,
                total_bytes: 100,
                percentage: 50.0,
            },
            SystemEvent::InferenceTokenStream {
                session_id: "s-1".to_string(),
                token: "hello".to_string(),
                is_final: false,
            },
            SystemEvent::WorkspaceChanged {
                active_workspace_id: "ws-1".to_string(),
            },
        ];

        for ev in events {
            assert!(!ev.event_type().is_empty());
            let envelope = EventEnvelope {
                sequence: 1,
                timestamp_epoch_secs: 1000,
                event: ev.clone(),
            };
            let json = serde_json::to_string(&envelope).expect("serialize envelope");
            let decoded: EventEnvelope = serde_json::from_str(&json).expect("deserialize envelope");
            assert_eq!(envelope, decoded);
        }
    }

    #[tokio::test]
    async fn test_broadcast_and_multi_subscriber_reception() {
        let bus = EventBus::new(32, 16);
        assert_eq!(bus.subscriber_count(), 0);
        assert_eq!(bus.last_sequence(), 0);

        let mut sub1 = bus.subscribe();
        let mut sub2 = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 2);

        let event = SystemEvent::WorkspaceChanged {
            active_workspace_id: "ws-test".to_string(),
        };

        let seq = bus.publish(event.clone(), 1000).await.unwrap();
        assert_eq!(seq, 1);
        assert_eq!(bus.last_sequence(), 1);

        let env1 = sub1.recv().await.unwrap();
        let env2 = sub2.recv().await.unwrap();

        assert_eq!(env1.sequence, 1);
        assert_eq!(env1.event, event);
        assert_eq!(env2.sequence, 1);
        assert_eq!(env2.event, event);
    }

    #[tokio::test]
    async fn test_replay_buffer_and_eviction() {
        let bus = EventBus::new(64, 16);

        for i in 1..=20 {
            bus.publish(
                SystemEvent::WorkspaceChanged {
                    active_workspace_id: format!("ws-{i}"),
                },
                1000 + i,
            )
            .await
            .unwrap();
        }

        assert_eq!(bus.last_sequence(), 20);
        let recent = bus.recent_events().await;
        assert_eq!(recent.len(), 16);
        assert_eq!(recent.first().unwrap().sequence, 5);
        assert_eq!(recent.last().unwrap().sequence, 20);
    }

    #[tokio::test]
    async fn test_backpressure_and_lagged_subscriber() {
        let bus = EventBus::new(16, 16);
        let mut slow_subscriber = bus.subscribe();

        for i in 1..=25 {
            bus.publish(
                SystemEvent::InferenceTokenStream {
                    session_id: "s-1".to_string(),
                    token: format!("tok-{i}"),
                    is_final: i == 25,
                },
                1000 + i,
            )
            .await
            .unwrap();
        }

        let recv_result = slow_subscriber.recv().await;
        assert!(matches!(recv_result, Err(EventBusError::Lagged { .. })));

        let next_msg = slow_subscriber.recv().await;
        assert!(next_msg.is_ok());
    }

    #[tokio::test]
    async fn test_try_recv_and_error_display() {
        let bus = EventBus::new(16, 16);
        let mut sub = bus.subscribe();

        assert!(matches!(sub.try_recv(), Err(EventBusError::Lagged { .. })));

        bus.publish(
            SystemEvent::WorkspaceChanged {
                active_workspace_id: "ws-direct".to_string(),
            },
            2000,
        )
        .await
        .unwrap();

        let env = sub.try_recv().unwrap();
        assert_eq!(env.sequence, 1);

        let err1 = EventBusError::Lagged { missed_events: 5 };
        assert!(format!("{err1}").contains("missed 5 events"));
        let err2 = EventBusError::ChannelClosed;
        assert!(format!("{err2}").contains("closed"));
        let err3 = EventBusError::SerializationError {
            message: "bad format".to_string(),
        };
        assert!(format!("{err3}").contains("bad format"));
        let err4 = EventBusError::LockPoisoned;
        assert!(format!("{err4}").contains("poisoned"));

        let s_err1 = ServerError::Unauthorized;
        assert!(format!("{s_err1}").contains("Unauthorized"));
        let s_err2 = ServerError::NotFound {
            path: "/x".to_string(),
        };
        assert!(format!("{s_err2}").contains("/x"));
        let s_err3 = ServerError::BadRequest {
            message: "bad".to_string(),
        };
        assert!(format!("{s_err3}").contains("bad"));
        let s_err4 = ServerError::InternalError {
            message: "internal".to_string(),
        };
        assert!(format!("{s_err4}").contains("internal"));
        let s_err5 = ServerError::IoError {
            message: "io".to_string(),
        };
        assert!(format!("{s_err5}").contains("io"));
    }

    #[tokio::test]
    async fn test_ipc_bridge_direct_operations() {
        let storage = StorageEngine::in_memory().unwrap();
        let bus = EventBus::default();
        let nonce = "test-session-nonce-12345".to_string();

        let bridge = IpcBridge::new(storage, bus, nonce.clone()).unwrap();
        assert_eq!(bridge.session_nonce(), nonce);
        assert!(bridge.verify_nonce(&nonce).is_ok());
        assert!(bridge.verify_nonce(&format!("Bearer {nonce}")).is_ok());
        assert!(bridge.verify_nonce("invalid-nonce").is_err());

        // Health check
        let health = bridge.get_health();
        assert_eq!(health["status"], "ok");

        // Snapshot query
        let snapshot = bridge.get_system_snapshot().await.unwrap();
        assert_eq!(snapshot.contract_version, "1.0.0");
        assert_eq!(snapshot.capabilities.len(), 7);

        // Privacy policy update
        let updated = bridge
            .update_privacy_policy(
                PrivacyPolicyUpdatePayload {
                    local_only_mode: Some(false),
                    force_cpu_only: Some(true),
                    grant_consent: None,
                    revoke_consent: None,
                },
                1000,
            )
            .await
            .unwrap();
        assert!(!updated.local_only_mode);
        assert!(updated.force_cpu_only);

        // Verify state is reflected in bridge and persisted to storage
        let policy = bridge.get_privacy_policy().await;
        assert_eq!(policy, updated);
        assert_eq!(
            bridge
                .storage()
                .get_setting("local_only_mode")
                .unwrap()
                .unwrap()
                .value,
            "false"
        );
        assert_eq!(
            bridge
                .storage()
                .get_setting("force_cpu_only")
                .unwrap()
                .unwrap()
                .value,
            "true"
        );

        // Verify event was emitted to bus
        let recent = bridge.get_recent_events().await;
        assert!(!recent.is_empty());
        assert_eq!(
            recent.last().unwrap().event.event_type(),
            "privacy_policy_updated"
        );
    }

    #[tokio::test]
    async fn test_ipc_server_localhost_binding_and_http_routes() {
        let storage = StorageEngine::in_memory().unwrap();
        let bus = EventBus::default();
        let nonce = "sec-token-abcdef123456".to_string();

        let bridge = IpcBridge::new(storage, bus, nonce.clone()).unwrap();

        // 1. Verify that binding to non-localhost (0.0.0.0) is rejected for security
        let bad_bind = IpcServer::bind(bridge.clone(), "0.0.0.0:8080").await;
        assert!(bad_bind.is_err());

        // 2. Bind strictly to localhost loopback with dynamic port
        let server = IpcServer::bind(bridge, "127.0.0.1:0").await.unwrap();
        let (addr, _handle) = server.spawn();

        // Helper to execute raw HTTP request
        async fn send_http(addr: std::net::SocketAddr, raw_req: &str) -> (String, String) {
            let mut stream = TcpStream::connect(addr).await.unwrap();
            stream.write_all(raw_req.as_bytes()).await.unwrap();
            stream.flush().await.unwrap();

            let mut resp = String::new();
            let mut buf = vec![0u8; 4096];
            loop {
                let n = stream.read(&mut buf).await.unwrap();
                if n == 0 {
                    break;
                }
                resp.push_str(&String::from_utf8_lossy(&buf[..n]));
            }
            let status = resp.lines().next().unwrap_or("").to_string();
            (status, resp)
        }

        // 3. GET /v1/health (public, no auth required)
        let (status, body) =
            send_http(addr, "GET /v1/health HTTP/1.1\r\nHost: localhost\r\n\r\n").await;
        assert!(status.contains("200 OK"));
        assert!(body.contains("\"status\":\"ok\""));

        // 4. GET /v1/system/snapshot WITHOUT auth -> 401 Unauthorized
        let (status_no_auth, _) = send_http(
            addr,
            "GET /v1/system/snapshot HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await;
        assert!(status_no_auth.contains("401 Unauthorized"));

        // 5. GET /v1/system/snapshot WITH valid auth -> 200 OK
        let req_snapshot = format!(
            "GET /v1/system/snapshot HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer {nonce}\r\n\r\n"
        );
        let (status_auth, body_auth) = send_http(addr, &req_snapshot).await;
        assert!(status_auth.contains("200 OK"));
        assert!(body_auth.contains("\"contract_version\":\"1.0.0\""));

        // 6. GET /v1/privacy/policy -> 200 OK
        let req_policy = format!(
            "GET /v1/privacy/policy HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer {nonce}\r\n\r\n"
        );
        let (status_policy, body_policy) = send_http(addr, &req_policy).await;
        assert!(status_policy.contains("200 OK"));
        assert!(body_policy.contains("\"local_only_mode\":true"));

        // 7. POST /v1/privacy/policy -> 200 OK
        let post_body = r#"{"local_only_mode":false,"force_cpu_only":true}"#;
        let req_post_policy = format!(
            "POST /v1/privacy/policy HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer {nonce}\r\nContent-Length: {}\r\n\r\n{}",
            post_body.len(),
            post_body
        );
        let (status_post, body_post) = send_http(addr, &req_post_policy).await;
        assert!(status_post.contains("200 OK"));
        assert!(body_post.contains("\"local_only_mode\":false"));

        // 8. GET /v1/events/recent -> 200 OK
        let req_events = format!(
            "GET /v1/events/recent HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer {nonce}\r\n\r\n"
        );
        let (status_events, _) = send_http(addr, &req_events).await;
        assert!(status_events.contains("200 OK"));

        // 9. Unknown route -> 404 Not Found
        let req_404 = format!(
            "GET /v1/unknown HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer {nonce}\r\n\r\n"
        );
        let (status_404, _) = send_http(addr, &req_404).await;
        assert!(status_404.contains("404 Not Found"));
    }

    #[tokio::test]
    async fn test_ipc_server_sse_event_streaming() {
        let storage = StorageEngine::in_memory().unwrap();
        let bus = EventBus::default();
        let nonce = "sse-token-987654".to_string();

        let bridge = IpcBridge::new(storage, bus.clone(), nonce.clone()).unwrap();
        let server = IpcServer::bind(bridge, "127.0.0.1:0").await.unwrap();
        let (addr, _handle) = server.spawn();

        let mut stream = TcpStream::connect(addr).await.unwrap();
        let req_sse = format!(
            "GET /v1/events/stream HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer {nonce}\r\n\r\n"
        );
        stream.write_all(req_sse.as_bytes()).await.unwrap();
        stream.flush().await.unwrap();

        // Read initial SSE HTTP header
        let mut header_buf = vec![0u8; 128];
        let n = stream.read(&mut header_buf).await.unwrap();
        let header_str = String::from_utf8_lossy(&header_buf[..n]);
        assert!(header_str.contains("200 OK"));
        assert!(header_str.contains("text/event-stream"));

        // Publish event to bus while SSE client is connected
        bus.publish(
            SystemEvent::WorkspaceChanged {
                active_workspace_id: "ws-sse-test".to_string(),
            },
            1234,
        )
        .await
        .unwrap();

        // Read SSE chunk from stream
        let mut event_buf = vec![0u8; 512];
        let n_chunk = stream.read(&mut event_buf).await.unwrap();
        let chunk_str = String::from_utf8_lossy(&event_buf[..n_chunk]);
        assert!(chunk_str.contains("data: "));
        assert!(chunk_str.contains("ws-sse-test"));
    }
}
