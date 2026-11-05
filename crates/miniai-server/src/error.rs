use thiserror::Error;

/// Errors produced across event bus and local IPC server operations.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum EventBusError {
    #[error("Subscriber lagged behind: missed {missed_events} events due to backpressure")]
    Lagged { missed_events: u64 },

    #[error("Event bus channel closed: no active receivers or senders")]
    ChannelClosed,

    #[error("Event serialization error: {message}")]
    SerializationError { message: String },

    #[error("Event bus lock poisoned")]
    LockPoisoned,
}
