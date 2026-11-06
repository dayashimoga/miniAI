use std::fs;
use std::path::Path;

use miniai_rag::{ChunkRecord, DocumentType, IngestionConfig, IngestionPipeline, RagError};
use tempfile::tempdir;

#[test]
fn test_markdown_chunking_with_headings_and_citations() {
    let tmp = tempdir().unwrap();
    let workspace = tmp.path();

    let doc_content = r#"# Architecture Overview

MiniAI is a modular local-first platform designed for CPU-first execution.
It provides full offline privacy and decoupled model adapters.

## Modality Adapters

There are 7 primary modalities supported:
- LLM
- Vision
- Image Generation
- Speech ASR
- Speech TTS
- Music
- Video

## Security Boundaries

All cloud compute is gated by explicit consent tokens.
Local-only mode blocks any outbound connections.
"#;

    let doc_path = workspace.join("overview.md");
    fs::write(&doc_path, doc_content).unwrap();

    let config = IngestionConfig {
        max_chunk_chars: 180,
        chunk_overlap_chars: 30,
        ..Default::default()
    };

    let mut pipeline = IngestionPipeline::new(workspace, config);
    let chunks = pipeline
        .ingest_file(&doc_path)
        .unwrap()
        .expect("chunks expected");

    assert!(!chunks.is_empty());
    for chunk in &chunks {
        assert_eq!(chunk.file_path, "overview.md");
        assert_eq!(chunk.doc_type, DocumentType::Markdown);
        assert!(chunk.line_start >= 1);
        assert!(chunk.line_end >= chunk.line_start);
        assert_eq!(
            chunk.citation_tag,
            ChunkRecord::format_citation("overview.md", chunk.line_start, chunk.line_end)
        );
        assert_eq!(chunk.content_sha256.len(), 64);
        assert_eq!(chunk.chunk_id.len(), 64);
    }

    assert_eq!(pipeline.stats().files_indexed, 1);
    assert_eq!(pipeline.stats().unique_chunks_indexed, chunks.len());
}

#[test]
fn test_code_chunking_with_accurate_lines() {
    let tmp = tempdir().unwrap();
    let workspace = tmp.path();

    let code_content = r#"// MiniAI Sample Module
pub struct Calculator {
    base_val: i32,
}

impl Calculator {
    pub fn new(base_val: i32) -> Self {
        Self { base_val }
    }

    pub fn compute(&self, x: i32) -> i32 {
        self.base_val + x * 2
    }
}
"#;

    let src_dir = workspace.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let code_path = src_dir.join("calc.rs");
    fs::write(&code_path, code_content).unwrap();

    let mut pipeline = IngestionPipeline::new(workspace, IngestionConfig::default());
    let chunks = pipeline
        .ingest_file(&code_path)
        .unwrap()
        .expect("chunks expected");

    assert!(!chunks.is_empty());
    assert_eq!(chunks[0].file_path, "src/calc.rs");
    match &chunks[0].doc_type {
        DocumentType::Code { language } => assert_eq!(language, "rust"),
        other => panic!("Expected Code doc_type, got {:?}", other),
    }

    assert_eq!(chunks[0].line_start, 1);
    assert_eq!(
        chunks[0].citation_tag,
        format!("src/calc.rs#L1-L{}", code_content.lines().count())
    );
}

#[test]
fn test_content_addressed_chunk_deduplication() {
    let tmp = tempdir().unwrap();
    let workspace = tmp.path();

    let duplicate_text = "This exact line of text is shared across multiple documentation files.";

    let file1 = workspace.join("doc1.txt");
    let file2 = workspace.join("doc2.txt");

    fs::write(&file1, duplicate_text).unwrap();
    fs::write(&file2, duplicate_text).unwrap();

    let mut pipeline = IngestionPipeline::new(workspace, IngestionConfig::default());

    let chunks1 = pipeline
        .ingest_file(&file1)
        .unwrap()
        .expect("chunks from doc1");
    assert_eq!(chunks1.len(), 1);

    let chunks2 = pipeline
        .ingest_file(&file2)
        .unwrap()
        .expect("empty unique chunks from doc2");
    assert!(
        chunks2.is_empty(),
        "Duplicate content chunk should be skipped"
    );

    assert_eq!(pipeline.stats().files_indexed, 2);
    assert_eq!(pipeline.stats().unique_chunks_indexed, 1);
    assert_eq!(pipeline.stats().duplicate_chunks_skipped, 1);
}

#[test]
fn test_sensitive_files_excluded() {
    let tmp = tempdir().unwrap();
    let workspace = tmp.path();

    let env_file = workspace.join(".env");
    fs::write(&env_file, "API_SECRET=1234567890").unwrap();

    let key_file = workspace.join("server.key");
    fs::write(&key_file, "simulated-private-key-material").unwrap();

    let secret_file = workspace.join("secrets.json");
    fs::write(&secret_file, "{\"secret\": \"token\"}").unwrap();

    let mut pipeline = IngestionPipeline::new(workspace, IngestionConfig::default());

    assert!(pipeline.ingest_file(&env_file).unwrap().is_none());
    assert!(pipeline.ingest_file(&key_file).unwrap().is_none());
    assert!(pipeline.ingest_file(&secret_file).unwrap().is_none());

    assert_eq!(pipeline.stats().files_excluded, 3);
    assert_eq!(pipeline.stats().files_indexed, 0);
}

#[test]
fn test_binary_files_rejected() {
    let tmp = tempdir().unwrap();
    let workspace = tmp.path();

    let bin_file = workspace.join("archive.bin");
    let bin_data = vec![0x7f, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01, 0x00, 0x00, 0x00];
    fs::write(&bin_file, &bin_data).unwrap();

    let mut pipeline = IngestionPipeline::new(workspace, IngestionConfig::default());
    let res = pipeline.ingest_file(&bin_file).unwrap();

    assert!(res.is_none());
    assert_eq!(pipeline.stats().files_excluded, 1);
    assert_eq!(pipeline.stats().files_indexed, 0);
}

#[test]
fn test_path_traversal_rejected() {
    let tmp = tempdir().unwrap();
    let workspace = tmp.path();

    let mut pipeline = IngestionPipeline::new(workspace, IngestionConfig::default());
    let traversal_path = Path::new("../outside.txt");

    let err = pipeline.ingest_file(traversal_path).unwrap_err();
    match err {
        RagError::PathTraversal(_) => {}
        other => panic!("Expected PathTraversal error, got {:?}", other),
    }
}

#[test]
fn test_file_too_large_rejected() {
    let tmp = tempdir().unwrap();
    let workspace = tmp.path();

    let big_file = workspace.join("large.txt");
    let big_content = "A".repeat(500);
    fs::write(&big_file, big_content).unwrap();

    let config = IngestionConfig {
        max_file_size_bytes: 200, // cap at 200 bytes
        ..Default::default()
    };

    let mut pipeline = IngestionPipeline::new(workspace, config);
    let err = pipeline.ingest_file(&big_file).unwrap_err();

    match err {
        RagError::FileTooLarge {
            path,
            actual_bytes,
            max_bytes,
        } => {
            assert_eq!(path, "large.txt");
            assert_eq!(actual_bytes, 500);
            assert_eq!(max_bytes, 200);
        }
        other => panic!("Expected FileTooLarge, got {:?}", other),
    }
}

#[test]
fn test_incremental_directory_ingestion_and_manifest_export() {
    let tmp = tempdir().unwrap();
    let workspace = tmp.path();

    let doc1 = workspace.join("guide.md");
    fs::write(&doc1, "# Getting Started\n\nWelcome to MiniAI.").unwrap();

    let sub = workspace.join("src");
    fs::create_dir_all(&sub).unwrap();
    let doc2 = sub.join("main.py");
    fs::write(&doc2, "def main():\n    print('hello')\n").unwrap();

    let mut pipeline = IngestionPipeline::new(workspace, IngestionConfig::default());

    // First scan: 2 files indexed
    let stats1 = pipeline.ingest_directory(workspace).unwrap();
    assert_eq!(stats1.files_indexed, 2);
    assert_eq!(stats1.files_skipped_unchanged, 0);
    assert!(stats1.unique_chunks_indexed >= 2);

    // Second scan without modifications: all unchanged files skipped!
    let stats2 = pipeline.ingest_directory(workspace).unwrap();
    assert_eq!(stats2.files_skipped_unchanged, 2);

    // Manifest export verification
    let manifest = pipeline.export_manifest("ws-default");
    assert_eq!(manifest.workspace_id, "ws-default");
    assert_eq!(manifest.documents.len(), 2);
    assert_eq!(manifest.documents[0].file_path, "guide.md");
    assert_eq!(manifest.documents[1].file_path, "src/main.py");
    assert_eq!(manifest.chunk_count, pipeline.all_chunks().len());

    // Manifest persistence
    let manifest_path = workspace.join("index_manifest.json");
    let saved = pipeline
        .save_manifest("ws-default", &manifest_path)
        .unwrap();
    assert!(saved.exists());

    let loaded_str = fs::read_to_string(&saved).unwrap();
    assert!(loaded_str.contains("ws-default"));
    assert!(loaded_str.contains("guide.md"));
}
