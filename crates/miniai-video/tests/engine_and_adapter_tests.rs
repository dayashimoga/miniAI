mod test_helpers;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use test_helpers::{create_test_mp4, create_test_rawv};

use miniai_core::{AcceleratorBackend, Modality, ModalityAdapter};
use miniai_video::adapter::VideoAdapter;
use miniai_video::buffer_pool::SharedFrameBufferPool;
use miniai_video::engine::{VideoEngine, VideoEngineConfig};
use miniai_video::sanitizer::VideoSanitizer;
use miniai_video::types::{BufferPoolConfig, KeyframeExtractionMode, TemporalQaQuery};
use miniai_video::VideoError;

#[test]
fn test_video_engine_analyze_video() {
    let bytes = create_test_mp4(1280, 720, 4.0, true);
    let pool = SharedFrameBufferPool::new(BufferPoolConfig::default());
    let engine = VideoEngine::new(pool);

    let config = VideoEngineConfig {
        extraction_mode: KeyframeExtractionMode::AdaptiveDynamic {
            max_keyframes: 6,
            threshold: 0.15,
        },
        ..Default::default()
    };

    let result = engine
        .analyze_video(&bytes, &config)
        .expect("analyze video");

    assert_eq!(result.metadata.width, 1280);
    assert_eq!(result.metadata.height, 720);
    assert!(!result.keyframes.is_empty());
    assert!(!result.scenes.is_empty());
    assert!(!result.detected_actions.is_empty());
    assert!(!result.overall_summary.is_empty());
    assert!(result.processing_time_ms < 60_000);

    // Verify scene boundaries
    for scene in &result.scenes {
        assert!(scene.end_ms >= scene.start_ms);
        assert!(!scene.visual_summary.is_empty());
    }
}

#[test]
fn test_video_engine_temporal_qa() {
    let bytes = create_test_mp4(1280, 720, 5.0, true);
    let pool = SharedFrameBufferPool::new(BufferPoolConfig::default());
    let engine = VideoEngine::new(pool);
    let analysis = engine
        .analyze_video(&bytes, &VideoEngineConfig::default())
        .expect("analyze");

    let query = TemporalQaQuery::new("When does the camera pan or action sequence occur?");
    let qa_result = engine.temporal_qa(&analysis, &query).expect("temporal qa");

    assert_eq!(qa_result.question, query.question);
    assert!(!qa_result.answer.is_empty());
    assert!(qa_result.confidence >= 0.5);
    assert!(qa_result.end_ms >= qa_result.start_ms);
    assert!(!qa_result.supporting_frame_indices.is_empty());
}

#[test]
fn test_video_engine_cancellation() {
    let bytes = create_test_rawv(64, 64, 10, 20, false);
    let pool = SharedFrameBufferPool::new(BufferPoolConfig::default());
    let token = Arc::new(AtomicBool::new(true)); // Pre-cancelled
    let engine = VideoEngine::new(pool).with_cancellation_token(token);

    let err = engine
        .analyze_video(&bytes, &VideoEngineConfig::default())
        .expect_err("Should be cancelled");
    assert!(matches!(err, VideoError::Cancelled));
}

#[test]
fn test_video_adapter_modality_contract() {
    let adapter = VideoAdapter::new("video-cpu-adapter");

    assert_eq!(adapter.adapter_id(), "video-cpu-adapter");
    assert_eq!(adapter.modality(), Modality::Video);
    assert!(adapter
        .supported_backends()
        .contains(&AcceleratorBackend::Cpu));

    let cap = adapter.capability();
    assert_eq!(cap.adapter_id, "video-cpu-adapter");
    assert_eq!(cap.modality, Modality::Video);
    assert!(cap.supports_offline);
    assert!(cap.supports_streaming);

    let health = adapter.health_check().expect("health check");
    assert_eq!(health.adapter_id, "video-cpu-adapter");
    assert_eq!(health.modality, Modality::Video);
    assert!(health.ready);
    assert_eq!(health.active_backend, AcceleratorBackend::Cpu);
}

#[test]
fn test_video_adapter_weights_loading_and_integrity() {
    let adapter = VideoAdapter::new("video-cpu-adapter");
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let weights_path = temp_dir.path().join("model_weights.bin");
    let content = b"miniai_video_test_model_weights_payload_12345";
    std::fs::write(&weights_path, content).expect("write weights");

    let expected_hash = VideoSanitizer::compute_sha256(content);

    // Valid hash
    adapter
        .load_model_weights(&weights_path, Some(&expected_hash))
        .expect("Load with valid SHA-256");
    assert!(adapter.is_loaded());

    // Mismatched hash
    let err = adapter
        .load_model_weights(
            &weights_path,
            Some("bad_hash_0000000000000000000000000000000000000000000000000000000000000000"),
        )
        .expect_err("Should reject mismatched SHA-256");
    assert!(matches!(err, VideoError::WeightsIntegrityMismatch { .. }));

    // Non-existent file
    let missing_path = temp_dir.path().join("missing.bin");
    let err_missing = adapter
        .load_model_weights(&missing_path, None)
        .expect_err("Should fail for missing file");
    assert!(matches!(err_missing, VideoError::WeightsNotFound(_)));
}

#[test]
fn test_video_adapter_end_to_end_analysis_and_qa() {
    let adapter = VideoAdapter::new("video-cpu-adapter");
    let bytes = create_test_mp4(640, 480, 3.0, true);

    let analysis = adapter
        .analyze_video(&bytes, &VideoEngineConfig::default())
        .expect("adapter analyze");
    assert_eq!(analysis.metadata.width, 640);
    assert_eq!(analysis.metadata.height, 480);

    let qa = adapter
        .temporal_qa(
            &analysis,
            &TemporalQaQuery::new("What happens at the beginning?"),
        )
        .expect("adapter qa");
    assert!(!qa.answer.is_empty());

    let audio = adapter
        .demux_audio(&bytes, &analysis.metadata)
        .expect("adapter demux")
        .expect("should produce audio");
    assert_eq!(audio.sample_rate, 44100);

    let pool_stats = adapter.pool_stats().expect("pool stats");
    assert_eq!(pool_stats.budget_bytes, 64 * 1024 * 1024);
}
