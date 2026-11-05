#![forbid(unsafe_code)]
//! # MiniAI Server & Event Bus (`miniai-server`)
//!
//! Strongly typed asynchronous event bus, system telemetry streaming, bounded backpressure
//! queues, replay buffer, and local IPC API server for MiniAI.

pub mod bus;
pub mod error;
pub mod events;

pub use bus::{EventBus, EventSubscriber};
pub use error::EventBusError;
pub use events::{EventEnvelope, SystemEvent};

#[cfg(test)]
mod tests {
    use super::*;
    use miniai_core::{AcceleratorBackend, HardwareProfile, PrivacyPolicyState};
    use miniai_storage::DiagnosticEventRecord;

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

        // Publish 20 events into a buffer with capacity 16
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
        // Oldest retained is event 5, newest is 20
        assert_eq!(recent.first().unwrap().sequence, 5);
        assert_eq!(recent.last().unwrap().sequence, 20);
    }

    #[tokio::test]
    async fn test_backpressure_and_lagged_subscriber() {
        // Create an event bus with a tiny broadcast queue of 16 items
        let bus = EventBus::new(16, 16);
        let mut slow_subscriber = bus.subscribe();

        // Publish 25 events without reading
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

        // Slow subscriber should report Lagged error on read
        let recv_result = slow_subscriber.recv().await;
        assert!(matches!(recv_result, Err(EventBusError::Lagged { .. })));

        // Subsequent read succeeds and picks up the newest available message
        let next_msg = slow_subscriber.recv().await;
        assert!(next_msg.is_ok());
    }

    #[tokio::test]
    async fn test_try_recv_and_error_display() {
        let bus = EventBus::new(16, 16);
        let mut sub = bus.subscribe();

        // Initially empty
        assert!(matches!(sub.try_recv(), Err(EventBusError::Lagged { .. })));

        bus.publish(
            SystemEvent::WorkspaceChanged {
                active_workspace_id: "ws-direct".to_string(),
            },
            2000,
        )
        .await
        .unwrap();

        // Non-blocking try_recv succeeds
        let env = sub.try_recv().unwrap();
        assert_eq!(env.sequence, 1);

        // Error display
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
    }
}
