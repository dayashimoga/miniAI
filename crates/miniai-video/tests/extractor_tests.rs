mod test_helpers;

use test_helpers::{create_test_mp4, create_test_rawv};

use miniai_video::buffer_pool::SharedFrameBufferPool;
use miniai_video::extractor::{AtomicWorkspaceManager, FrameExtractor};
use miniai_video::sanitizer::VideoSanitizer;
use miniai_video::types::{BufferPoolConfig, KeyframeExtractionMode, SanitizerConfig, VideoFrame};

#[test]
fn test_uniform_interval_sampling() {
    let bytes = create_test_rawv(32, 32, 10, 30, false);
    let meta = VideoSanitizer::validate_bytes(&bytes, &SanitizerConfig::default()).expect("meta");
    let pool = SharedFrameBufferPool::new(BufferPoolConfig::default());
    let extractor = FrameExtractor::new(pool);

    let keyframes = extractor
        .extract_keyframes(
            &bytes,
            &meta,
            &KeyframeExtractionMode::UniformInterval { step_frames: 5 },
        )
        .expect("extract keyframes");

    // 30 frames step by 5 -> indices 0, 5, 10, 15, 20, 25 (6 keyframes)
    assert_eq!(keyframes.len(), 6);
    assert_eq!(keyframes[0].frame_index, 0);
    assert_eq!(keyframes[1].frame_index, 5);
    assert_eq!(keyframes[5].frame_index, 25);
}

#[test]
fn test_uniform_duration_sampling() {
    let bytes = create_test_mp4(640, 360, 4.0, false);
    let meta = VideoSanitizer::validate_bytes(&bytes, &SanitizerConfig::default()).expect("meta");
    let pool = SharedFrameBufferPool::new(BufferPoolConfig::default());
    let extractor = FrameExtractor::new(pool);

    let keyframes = extractor
        .extract_keyframes(
            &bytes,
            &meta,
            &KeyframeExtractionMode::UniformDuration { interval_ms: 1000 },
        )
        .expect("duration sampling");

    assert!(!keyframes.is_empty());
    for kf in &keyframes {
        assert!(kf.is_keyframe);
        assert_eq!(kf.width, 640);
        assert_eq!(kf.height, 360);
        assert_eq!(kf.rgb_data.len(), 640 * 360 * 3);
        assert!(!kf.sha256.is_empty());
    }
}

#[test]
fn test_scene_boundary_change_sampling() {
    let bytes = create_test_mp4(640, 360, 3.0, false);
    let meta = VideoSanitizer::validate_bytes(&bytes, &SanitizerConfig::default()).expect("meta");
    let pool = SharedFrameBufferPool::new(BufferPoolConfig::default());
    let extractor = FrameExtractor::new(pool);

    let keyframes = extractor
        .extract_keyframes(
            &bytes,
            &meta,
            &KeyframeExtractionMode::SceneBoundaryChange { threshold: 0.15 },
        )
        .expect("scene boundary sampling");

    assert!(!keyframes.is_empty());
    assert!(keyframes[0].is_keyframe);
    assert_eq!(keyframes[0].frame_index, 0);
}

#[test]
fn test_adaptive_dynamic_sampling() {
    let bytes = create_test_mp4(1280, 720, 6.0, false);
    let meta = VideoSanitizer::validate_bytes(&bytes, &SanitizerConfig::default()).expect("meta");
    let pool = SharedFrameBufferPool::new(BufferPoolConfig::default());
    let extractor = FrameExtractor::new(pool);

    let keyframes = extractor
        .extract_keyframes(
            &bytes,
            &meta,
            &KeyframeExtractionMode::AdaptiveDynamic {
                max_keyframes: 8,
                threshold: 0.20,
            },
        )
        .expect("adaptive sampling");

    assert!(!keyframes.is_empty());
    assert!(keyframes.len() <= 8);
}

#[test]
fn test_frame_delta_computation() {
    let f1 = VideoFrame {
        frame_index: 0,
        timestamp_ms: 0,
        width: 10,
        height: 10,
        rgb_data: vec![100u8; 300],
        is_keyframe: true,
        scene_id: 1,
        luminance_mean: 0.5,
        sha256: "h1".to_string(),
    };

    let f2 = VideoFrame {
        frame_index: 1,
        timestamp_ms: 33,
        width: 10,
        height: 10,
        rgb_data: vec![100u8; 300], // Identical
        is_keyframe: false,
        scene_id: 1,
        luminance_mean: 0.5,
        sha256: "h2".to_string(),
    };

    let f3 = VideoFrame {
        frame_index: 2,
        timestamp_ms: 66,
        width: 10,
        height: 10,
        rgb_data: vec![200u8; 300], // Different
        is_keyframe: false,
        scene_id: 1,
        luminance_mean: 0.8,
        sha256: "h3".to_string(),
    };

    let delta_identical = FrameExtractor::compute_frame_delta(&f1, &f2);
    let delta_diff = FrameExtractor::compute_frame_delta(&f1, &f3);

    assert_eq!(delta_identical, 0.0);
    assert!(delta_diff > 0.35);
}

#[test]
fn test_atomic_workspace_manager_auto_cleanup() {
    let temp_root = tempfile::tempdir().expect("root temp");
    let ws_path;
    {
        let ws = AtomicWorkspaceManager::new(Some(temp_root.path())).expect("create ws");
        ws_path = ws.path().to_path_buf();
        assert!(ws_path.exists());
        // Write a test dummy file inside workspace
        std::fs::write(ws_path.join("frame_0.png"), b"dummy").expect("write inside");
    } // Drop happens here

    // Verify workspace directory was cleaned up automatically
    assert!(!ws_path.exists(), "Workspace dir should be deleted on Drop");
}

#[test]
fn test_atomic_workspace_manager_preserve() {
    let temp_root = tempfile::tempdir().expect("root temp");
    let ws_path;
    {
        let mut ws = AtomicWorkspaceManager::new(Some(temp_root.path())).expect("create ws");
        ws_path = ws.path().to_path_buf();
        ws.set_preserve(true);
    } // Drop happens here

    assert!(ws_path.exists(), "Preserved workspace should remain");
    let _ = std::fs::remove_dir_all(&ws_path);
}
