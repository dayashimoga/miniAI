use miniai_rag::{HybridSearchEngine, IngestionConfig, IngestionPipeline, RetrievalConfig};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_end_to_end_ingestion_and_hybrid_retrieval() {
    let tmp = tempdir().unwrap();
    let workspace = tmp.path();

    // Create realistic repository documentation and code files
    let arch_doc = r#"# MiniAI System Architecture

MiniAI is built around a local-first philosophy with CPU-first execution.
When dedicated accelerators (CUDA, Metal, Vulkan) are unavailable or lack memory,
the AdmissionGuard smoothly falls back to AcceleratorBackend::Cpu.

Hardware detection inspects SIMD vector instructions including AVX2, AVX-512, and ARM NEON.
"#;

    let privacy_doc = r#"# Privacy & Security Governance

LocalOnlyPrivacyMode is enabled by default to prevent unauthorized cloud telemetry.
Every outbound remote inference call requires a valid, scoped CloudConsentToken.
If the token expires or is explicitly revoked, cloud calls are immediately blocked.
"#;

    let code_file = r#"// Model Registry and Download Manager
pub struct ModelImporter {
    quarantine_dir: PathBuf,
}

impl ModelImporter {
    pub fn import_local_model(&self, path: &Path, expected_sha256: Option<&str>) -> Result<(), RegistryError> {
        // Quarantine staging and SHA-256 validation
        println!("Staging in quarantine...");
        Ok(())
    }
}
"#;

    fs::write(workspace.join("ARCHITECTURE.md"), arch_doc).unwrap();
    fs::write(workspace.join("SECURITY.md"), privacy_doc).unwrap();
    let src_dir = workspace.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("importer.rs"), code_file).unwrap();

    // Ingest workspace
    let mut pipeline = IngestionPipeline::new(workspace, IngestionConfig::default());
    let stats = pipeline.ingest_directory(workspace).unwrap();
    assert_eq!(stats.files_indexed, 3);
    assert!(stats.unique_chunks_indexed >= 3);

    // Feed into Hybrid Search Engine
    let mut engine = HybridSearchEngine::new(RetrievalConfig {
        rrf_k: 60.0,
        dense_weight: 0.5,
        lexical_weight: 0.5,
        candidate_pool_size: 20,
    });
    let indexed_count = engine.index_pipeline_chunks(&pipeline);
    assert_eq!(indexed_count, pipeline.all_chunks().len());

    // 1. Query matching Architecture CPU fallback
    let arch_results = engine.search("AdmissionGuard CPU fallback SIMD", 3);
    assert!(!arch_results.is_empty());
    assert_eq!(arch_results[0].chunk.file_path, "ARCHITECTURE.md");
    assert!(arch_results[0]
        .citation_tag
        .starts_with("ARCHITECTURE.md#L"));
    assert!(
        arch_results[0].excerpt.contains("AdmissionGuard")
            || arch_results[0].excerpt.contains("CPU")
    );

    // 2. Query matching Privacy and CloudConsentToken
    let privacy_results = engine.search("CloudConsentToken revoked LocalOnlyPrivacyMode", 2);
    assert!(!privacy_results.is_empty());
    assert_eq!(privacy_results[0].chunk.file_path, "SECURITY.md");
    assert!(privacy_results[0].citation_tag.starts_with("SECURITY.md#L"));
    assert!(
        privacy_results[0].excerpt.contains("CloudConsentToken")
            || privacy_results[0].excerpt.contains("Privacy")
    );

    // 3. Query matching Rust code
    let code_results = engine.search("ModelImporter quarantine SHA-256", 2);
    assert!(!code_results.is_empty());
    assert_eq!(code_results[0].chunk.file_path, "src/importer.rs");
    assert!(code_results[0]
        .citation_tag
        .starts_with("src/importer.rs#L"));

    // Verify citation tags are non-empty and well-formed
    for res in &arch_results {
        assert!(res.rrf_score > 0.0);
        assert!(!res.citation_tag.is_empty());
        assert!(!res.excerpt.is_empty());
    }
}
