use thiserror::Error;

/// Storage errors produced across database and migration operations.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum StorageError {
    #[error("Database error: {message}")]
    DatabaseError { message: String },

    #[error("Migration error: {message}")]
    MigrationError { message: String },

    #[error("Checksum mismatch on applied migration {version} ('{name}'): expected {expected}, recorded {recorded}")]
    MigrationChecksumMismatch {
        version: u32,
        name: String,
        expected: String,
        recorded: String,
    },

    #[error("Database integrity check failed: {details:?}")]
    IntegrityCheckFailed { details: Vec<String> },

    #[error("Foreign key integrity check failed: {violations:?}")]
    ForeignKeyCheckFailed { violations: Vec<String> },

    #[error("Item not found: {entity} with key '{key}'")]
    NotFound { entity: &'static str, key: String },

    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    #[error("Storage engine lock poisoned")]
    LockPoisoned,
}

impl From<rusqlite::Error> for StorageError {
    fn from(err: rusqlite::Error) -> Self {
        StorageError::DatabaseError {
            message: err.to_string(),
        }
    }
}
