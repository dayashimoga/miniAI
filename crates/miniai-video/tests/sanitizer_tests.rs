mod test_helpers;

use test_helpers::{create_test_mp4, create_test_rawv, create_test_webm};

use miniai_video::sanitizer::VideoSanitizer;
use miniai_video::types::{SanitizerConfig, VideoCodec, VideoContainerFormat};
use miniai_video::VideoError;

#[test]
fn test_valid_mp4_parsing() {
    let bytes = create_test_mp4(1280, 720, 5.0, true);
    let config = SanitizerConfig::default();
    let meta = VideoSanitizer::validate_bytes(&bytes, &config).expect("Valid MP4 parsing");

    assert_eq!(meta.container_format, VideoContainerFormat::Mp4);
    assert_eq!(meta.codec, VideoCodec::H264);
    assert_eq!(meta.width, 1280);
    assert_eq!(meta.height, 720);
    assert!((meta.duration_secs - 5.0).abs() < 0.1);
    assert!(meta.has_audio);
    assert_eq!(meta.audio_channels, 2);
    assert_eq!(meta.file_size_bytes, bytes.len());
    assert!(!meta.sha256_hash.is_empty());
}

#[test]
fn test_valid_webm_parsing() {
    let bytes = create_test_webm(854, 480, 8.0, true);
    let config = SanitizerConfig::default();
    let meta = VideoSanitizer::validate_bytes(&bytes, &config).expect("Valid WebM parsing");

    assert_eq!(meta.container_format, VideoContainerFormat::WebM);
    assert_eq!(meta.codec, VideoCodec::Vp9);
    assert_eq!(meta.width, 854);
    assert_eq!(meta.height, 480);
    assert!((meta.duration_secs - 8.0).abs() < 0.1);
    assert!(meta.has_audio);
    assert_eq!(meta.file_size_bytes, bytes.len());
}

#[test]
fn test_valid_rawv_parsing() {
    let bytes = create_test_rawv(64, 64, 25, 50, false);
    let config = SanitizerConfig::default();
    let meta = VideoSanitizer::validate_bytes(&bytes, &config).expect("Valid RAWV parsing");

    assert_eq!(meta.container_format, VideoContainerFormat::RawFrames);
    assert_eq!(meta.codec, VideoCodec::RawRgb);
    assert_eq!(meta.width, 64);
    assert_eq!(meta.height, 64);
    assert_eq!(meta.fps, 25.0);
    assert_eq!(meta.total_frames, 50);
    assert!(!meta.has_audio);
}

#[test]
fn test_rejection_payload_too_small() {
    let tiny = vec![0u8; 4];
    let config = SanitizerConfig::default();
    let err = VideoSanitizer::validate_bytes(&tiny, &config).expect_err("Should reject tiny bytes");
    assert!(matches!(err, VideoError::CorruptPayload(_)));
}

#[test]
fn test_rejection_invalid_container() {
    let bogus = vec![0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
    let config = SanitizerConfig::default();
    let err = VideoSanitizer::validate_bytes(&bogus, &config).expect_err("Should reject unknown");
    assert!(matches!(err, VideoError::InvalidContainer(_)));
}

#[test]
fn test_rejection_oversized_file() {
    let bytes = create_test_mp4(640, 360, 2.0, false);
    let config = SanitizerConfig {
        max_file_size_bytes: bytes.len() - 1, // Strict limit below payload
        ..Default::default()
    };
    let err = VideoSanitizer::validate_bytes(&bytes, &config).expect_err("Should reject oversized");
    assert!(matches!(err, VideoError::FileTooLarge { .. }));
}

#[test]
fn test_rejection_excessive_duration() {
    let bytes = create_test_mp4(640, 360, 500.0, false);
    let config = SanitizerConfig {
        max_duration_secs: 60.0,
        ..Default::default()
    };
    let err = VideoSanitizer::validate_bytes(&bytes, &config)
        .expect_err("Should reject excessive duration");
    assert!(matches!(err, VideoError::DurationExceeded { .. }));
}

#[test]
fn test_rejection_excessive_resolution() {
    let bytes = create_test_mp4(3840, 2160, 5.0, false);
    let config = SanitizerConfig {
        max_dimension: 1920,
        ..Default::default()
    };
    let err = VideoSanitizer::validate_bytes(&bytes, &config)
        .expect_err("Should reject 4K when max is 1080p");
    assert!(matches!(err, VideoError::ResolutionExceeded { .. }));
}

#[test]
fn test_validate_file_on_disk() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let file_path = temp_dir.path().join("sample.mp4");
    let bytes = create_test_mp4(1280, 720, 3.0, true);
    std::fs::write(&file_path, &bytes).expect("write file");

    let config = SanitizerConfig::default();
    let meta = VideoSanitizer::validate_file(&file_path, &config).expect("Valid file");
    assert_eq!(meta.width, 1280);
    assert_eq!(meta.height, 720);
    assert!(meta.has_audio);
}

#[test]
fn test_validate_nonexistent_file() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let file_path = temp_dir.path().join("missing.mp4");
    let config = SanitizerConfig::default();
    let err = VideoSanitizer::validate_file(&file_path, &config).expect_err("Should fail missing");
    assert!(matches!(err, VideoError::CorruptPayload(_)));
}

#[test]
fn test_sha256_determinism() {
    let bytes = create_test_mp4(640, 480, 2.0, false);
    let h1 = VideoSanitizer::compute_sha256(&bytes);
    let h2 = VideoSanitizer::compute_sha256(&bytes);
    assert_eq!(h1, h2);
    assert_eq!(h1.len(), 64);
}
