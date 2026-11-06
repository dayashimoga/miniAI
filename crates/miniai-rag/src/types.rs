use serde::{Deserialize, Serialize};

/// Supported document categories for structure-aware parsing and chunking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    Markdown,
    Code { language: String },
    PlainText,
    PdfText,
}

impl DocumentType {
    /// Detects document type based on file extension.
    pub fn from_file_path(path: &str) -> Self {
        let lower = path.to_ascii_lowercase();
        if lower.ends_with(".md") || lower.ends_with(".markdown") {
            DocumentType::Markdown
        } else if lower.ends_with(".rs") {
            DocumentType::Code {
                language: "rust".to_string(),
            }
        } else if lower.ends_with(".py") {
            DocumentType::Code {
                language: "python".to_string(),
            }
        } else if lower.ends_with(".dart") {
            DocumentType::Code {
                language: "dart".to_string(),
            }
        } else if lower.ends_with(".js") || lower.ends_with(".mjs") || lower.ends_with(".cjs") {
            DocumentType::Code {
                language: "javascript".to_string(),
            }
        } else if lower.ends_with(".ts") || lower.ends_with(".tsx") {
            DocumentType::Code {
                language: "typescript".to_string(),
            }
        } else if lower.ends_with(".json") {
            DocumentType::Code {
                language: "json".to_string(),
            }
        } else if lower.ends_with(".yaml") || lower.ends_with(".yml") {
            DocumentType::Code {
                language: "yaml".to_string(),
            }
        } else if lower.ends_with(".toml") {
            DocumentType::Code {
                language: "toml".to_string(),
            }
        } else if lower.ends_with(".c") || lower.ends_with(".h") {
            DocumentType::Code {
                language: "c".to_string(),
            }
        } else if lower.ends_with(".cpp") || lower.ends_with(".hpp") || lower.ends_with(".cc") {
            DocumentType::Code {
                language: "cpp".to_string(),
            }
        } else if lower.ends_with(".pdf.txt") || lower.ends_with(".pdf") {
            DocumentType::PdfText
        } else {
            DocumentType::PlainText
        }
    }
}

/// Metadata record for an ingested file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentRecord {
    pub file_path: String,
    pub file_sha256: String,
    pub size_bytes: usize,
    pub line_count: usize,
    pub doc_type: DocumentType,
    pub last_modified_epoch_secs: u64,
}

/// A content-addressed chunk of text with exact source line citations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkRecord {
    /// Deterministic unique chunk ID derived from `sha256(file_path + ":" + line_start + ":" + line_end + ":" + content_sha256)`.
    pub chunk_id: String,
    /// Relative workspace file path.
    pub file_path: String,
    /// SHA-256 hash of the chunk's text alone (used for content deduplication).
    pub content_sha256: String,
    /// The chunk text content.
    pub text: String,
    /// 1-indexed starting line number (inclusive).
    pub line_start: usize,
    /// 1-indexed ending line number (inclusive).
    pub line_end: usize,
    /// Character count in the chunk.
    pub char_count: usize,
    /// Heuristic token estimate (~1.3 tokens per word or chars / 4).
    pub token_estimate: usize,
    /// Formatted citation tag: `file_path#Lline_start-Lline_end`.
    pub citation_tag: String,
    /// Inferred or specified document type.
    pub doc_type: DocumentType,
    /// Ingestion timestamp.
    pub created_at_epoch_secs: u64,
}

impl ChunkRecord {
    /// Constructs a citation string in the format `file_path#L{start}-L{end}` or `file_path#L{start}`.
    pub fn format_citation(file_path: &str, line_start: usize, line_end: usize) -> String {
        if line_start == line_end {
            format!("{file_path}#L{line_start}")
        } else {
            format!("{file_path}#L{line_start}-L{line_end}")
        }
    }
}

/// Configuration parameters for document and code ingestion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionConfig {
    /// Target chunk size in characters (e.g. 1200 chars ~ 300 words).
    pub max_chunk_chars: usize,
    /// Sliding window overlap in characters (e.g. 200 chars).
    pub chunk_overlap_chars: usize,
    /// Maximum allowed single file size in bytes before rejecting (default 10MB).
    pub max_file_size_bytes: usize,
    /// User-specified exclusion patterns (glob-style or substrings).
    pub custom_exclude_patterns: Vec<String>,
}

impl Default for IngestionConfig {
    fn default() -> Self {
        Self {
            max_chunk_chars: 1200,
            chunk_overlap_chars: 200,
            max_file_size_bytes: 10 * 1024 * 1024,
            custom_exclude_patterns: Vec::new(),
        }
    }
}

/// Ingestion statistics produced after indexing pass.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct IngestionStats {
    pub files_scanned: usize,
    pub files_indexed: usize,
    pub files_skipped_unchanged: usize,
    pub files_excluded: usize,
    pub chunks_generated: usize,
    pub unique_chunks_indexed: usize,
    pub duplicate_chunks_skipped: usize,
    pub total_chars_indexed: usize,
    pub duration_ms: f64,
}

/// Verifiable cryptographic manifest of an indexed knowledge base.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionManifest {
    pub manifest_id: String,
    pub workspace_id: String,
    pub documents: Vec<DocumentRecord>,
    pub chunk_count: usize,
    pub unique_chunk_count: usize,
    pub stats: IngestionStats,
    pub created_at_epoch_secs: u64,
}
