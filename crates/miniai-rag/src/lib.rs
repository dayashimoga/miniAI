#![forbid(unsafe_code)]

//! MiniAI Local Retrieval-Augmented Generation (RAG) & Document Ingestion Crate.
//!
//! Provides:
//! 1. Incremental, structure-aware document and code chunking (`Markdown`, `Code`, `PlainText`, `PdfText`).
//! 2. Exact source line citation spans (`file_path#Lstart-Lend`).
//! 3. Content-addressed SHA-256 chunk deduplication.
//! 4. Security validation against path traversal, symlink escapes, binary files, and sensitive files (`.env`, `.pem`, `.key`, `id_rsa*`).
//! 5. Incremental indexing change detection skipping unchanged files based on SHA-256 digest comparison.

pub mod chunker;
pub mod dedup;
pub mod error;
pub mod pipeline;
pub mod security;
pub mod types;

pub use chunker::Chunker;
pub use dedup::ChunkDeduplicator;
pub use error::RagError;
pub use pipeline::IngestionPipeline;
pub use types::{
    ChunkRecord, DocumentRecord, DocumentType, IngestionConfig, IngestionManifest, IngestionStats,
};
