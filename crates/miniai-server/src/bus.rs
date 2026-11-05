use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use crate::error::EventBusError;
use crate::events::{EventEnvelope, SystemEvent};

/// An asynchronous, bounded, multi-producer multi-consumer broadcast event bus.
#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<EventEnvelope>,
    sequence_counter: Arc<AtomicU64>,
    replay_buffer: Arc<RwLock<VecDeque<EventEnvelope>>>,
    replay_capacity: usize,
}

impl EventBus {
    /// Creates an `EventBus` with specified broadcast backpressure capacity and replay buffer capacity.
    pub fn new(broadcast_capacity: usize, replay_capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(broadcast_capacity.max(16));
        Self {
            sender,
            sequence_counter: Arc::new(AtomicU64::new(0)),
            replay_buffer: Arc::new(RwLock::new(VecDeque::with_capacity(
                replay_capacity.max(16),
            ))),
            replay_capacity: replay_capacity.max(16),
        }
    }

    /// Publishes a `SystemEvent` with the given timestamp.
    ///
    /// Monotonically increments the sequence counter, caches the event envelope in the replay
    /// ring buffer, and broadcasts it to all active subscribers.
    pub async fn publish(
        &self,
        event: SystemEvent,
        timestamp_epoch_secs: u64,
    ) -> Result<u64, EventBusError> {
        let sequence = self.sequence_counter.fetch_add(1, Ordering::SeqCst) + 1;
        let envelope = EventEnvelope {
            sequence,
            timestamp_epoch_secs,
            event,
        };

        // Cache in replay buffer
        {
            let mut replay = self.replay_buffer.write().await;
            if replay.len() >= self.replay_capacity {
                replay.pop_front();
            }
            replay.push_back(envelope.clone());
        }

        // Broadcast to receivers. Ignore SendError if there are zero active listeners.
        let _ = self.sender.send(envelope);
        Ok(sequence)
    }

    /// Subscribes to the live broadcast event stream.
    pub fn subscribe(&self) -> EventSubscriber {
        EventSubscriber {
            receiver: self.sender.subscribe(),
        }
    }

    /// Returns a snapshot of the most recent events from the replay buffer.
    pub async fn recent_events(&self) -> Vec<EventEnvelope> {
        let replay = self.replay_buffer.read().await;
        replay.iter().cloned().collect()
    }

    /// Returns the number of currently active live subscribers.
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }

    /// Returns the current highest sequence number published.
    pub fn last_sequence(&self) -> u64 {
        self.sequence_counter.load(Ordering::SeqCst)
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(1024, 100)
    }
}

/// A subscriber stream receiver listening to broadcast system events.
pub struct EventSubscriber {
    receiver: broadcast::Receiver<EventEnvelope>,
}

impl EventSubscriber {
    /// Asynchronously awaits the next published `EventEnvelope`.
    pub async fn recv(&mut self) -> Result<EventEnvelope, EventBusError> {
        self.receiver.recv().await.map_err(|e| match e {
            broadcast::error::RecvError::Lagged(n) => EventBusError::Lagged { missed_events: n },
            broadcast::error::RecvError::Closed => EventBusError::ChannelClosed,
        })
    }

    /// Attempts to read the next event without awaiting.
    pub fn try_recv(&mut self) -> Result<EventEnvelope, EventBusError> {
        self.receiver.try_recv().map_err(|e| match e {
            broadcast::error::TryRecvError::Lagged(n) => EventBusError::Lagged { missed_events: n },
            broadcast::error::TryRecvError::Closed => EventBusError::ChannelClosed,
            broadcast::error::TryRecvError::Empty => EventBusError::Lagged { missed_events: 0 },
        })
    }
}
