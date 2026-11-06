use std::collections::HashSet;
use std::fs;
use std::sync::Arc;
use std::time::Duration;

use miniai_rag::{
    HybridSearchEngine, IngestionConfig, IngestionPipeline, RagQueryTool, ToolCapability,
    ToolExecutionContext, ToolSandboxRunner,
};
use tempfile::tempdir;

#[test]
fn test_tool_sandbox_runner_capabilities_and_truncation() {
    let tmp = tempdir().unwrap();
    let workspace = tmp.path();

    let sample_file = workspace.join("notes.txt");
    let large_text = "Line A: Some important engineering notes.\n".repeat(50);
    fs::write(&sample_file, large_text).unwrap();

    let runner = ToolSandboxRunner::new();

    // 1. Capability Permission Check: Allowed capabilities only includes Compute
    let mut restrictive_caps = HashSet::new();
    restrictive_caps.insert(ToolCapability::Compute);

    let restrictive_context = ToolExecutionContext {
        workspace_root: workspace.to_path_buf(),
        local_only_mode: true,
        allowed_capabilities: restrictive_caps,
        max_timeout: Duration::from_millis(1000),
        max_output_bytes: 100,
    };

    // Attempt to execute workspace_read_file with only Compute capability granted -> denied!
    let denied_res = runner.execute(
        "workspace_read_file",
        r#"{"path": "notes.txt"}"#,
        &restrictive_context,
    );
    assert!(!denied_res.success);
    assert!(denied_res
        .error_message
        .unwrap()
        .contains("Permission denied"));

    // 2. Grant WorkspaceRead and test byte cap truncation
    let mut permissive_caps = HashSet::new();
    permissive_caps.insert(ToolCapability::WorkspaceRead);
    permissive_caps.insert(ToolCapability::Compute);

    let truncating_context = ToolExecutionContext {
        workspace_root: workspace.to_path_buf(),
        local_only_mode: true,
        allowed_capabilities: permissive_caps,
        max_timeout: Duration::from_millis(1000),
        max_output_bytes: 120, // 120 bytes max
    };

    let read_res = runner.execute(
        "workspace_read_file",
        r#"{"path": "notes.txt", "line_start": 1, "line_end": 40}"#,
        &truncating_context,
    );
    assert!(read_res.success);
    assert!(read_res.truncated, "Output should be marked truncated");
    assert!(read_res
        .output
        .contains("Output truncated by execution byte cap"));
    assert!(read_res.duration_ms >= 0.0);
}

#[test]
fn test_rag_query_tool_integration() {
    let tmp = tempdir().unwrap();
    let workspace = tmp.path();

    let doc = r#"# Hybrid Retrieval Architecture

MiniAI uses BM25 lexical search and dense vector embeddings combined via Reciprocal Rank Fusion.
Grounded citation spans are strictly formatted as file_path#Lstart-Lend.
"#;
    fs::write(workspace.join("retrieval_spec.md"), doc).unwrap();

    // Ingest pipeline
    let mut pipeline = IngestionPipeline::new(workspace, IngestionConfig::default());
    pipeline.ingest_directory(workspace).unwrap();

    // Hybrid engine
    let mut engine = HybridSearchEngine::default();
    engine.index_pipeline_chunks(&pipeline);
    let engine_arc = Arc::new(engine);

    // Register RagQueryTool into runner
    let mut runner = ToolSandboxRunner::new();
    runner.register_tool(Arc::new(RagQueryTool::new(engine_arc)));

    let context = ToolExecutionContext {
        workspace_root: workspace.to_path_buf(),
        ..Default::default()
    };

    let res = runner.execute(
        "rag_query",
        r#"{"query": "Reciprocal Rank Fusion citations", "limit": 2}"#,
        &context,
    );
    assert!(res.success);
    assert!(res.output.contains("Citation: retrieval_spec.md#L"));
    assert!(res.output.contains("Reciprocal Rank Fusion"));
}
