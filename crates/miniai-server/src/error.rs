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

/// Errors produced across IPC bridge and localhost HTTP server operations.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum ServerError {
    #[error("Unauthorized: invalid or missing session authentication nonce")]
    Unauthorized,

    #[error("Resource not found: {path}")]
    NotFound { path: String },

    #[error("Bad request: {message}")]
    BadRequest { message: String },

    #[error("Internal server error: {message}")]
    InternalError { message: String },

    #[error("IO error: {message}")]
    IoError { message: String },
}
