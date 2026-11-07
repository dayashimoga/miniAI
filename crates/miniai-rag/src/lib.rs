#![forbid(unsafe_code)]

//! MiniAI Local Retrieval-Augmented Generation (RAG) & Document Ingestion Crate.
//!
//! Provides:
//! 1. Incremental, structure-aware document and code chunking (`Markdown`, `Code`, `PlainText`, `PdfText`).
//! 2. Exact source line citation spans (`file_path#Lstart-Lend`).
//! 3. Content-addressed SHA-256 chunk deduplication.
//! 4. Security validation against path traversal, symlink escapes, binary files, and sensitive files.
//! 5. Incremental indexing change detection skipping unchanged files based on SHA-256 digest comparison.
//! 6. Local BM25 Okapi lexical indexer with document length normalization.
//! 7. Local CPU-first deterministic embedding model and vector index.
//! 8. Hybrid search engine using Reciprocal Rank Fusion (RRF) for grounded retrieval.
//! 9. Capability-sandboxed local tool runner with execution timeouts, output caps, and privacy gates.

pub mod bm25;
pub mod chunker;
pub mod dedup;
pub mod embedding;
pub mod error;
pub mod pipeline;
pub mod retrieval;
pub mod security;
pub mod tools;
pub mod types;

pub use bm25::Bm25Index;
pub use chunker::Chunker;
pub use dedup::ChunkDeduplicator;
pub use embedding::{cosine_similarity, LocalEmbeddingModel, VectorIndex};
pub use error::RagError;
pub use pipeline::IngestionPipeline;
pub use retrieval::{HybridSearchEngine, RetrievalConfig, RetrievalResult};
pub use tools::{
    CalculatorTool, CodeSymbolSearchTool, RagQueryTool, SandboxedTool, ToolCapability,
    ToolDescriptor, ToolExecutionContext, ToolExecutionResult, ToolRegistry, ToolSandboxRunner,
    WorkspaceFileReadTool,
};
pub use types::{
    ChunkRecord, DocumentRecord, DocumentType, IngestionConfig, IngestionManifest, IngestionStats,
};
