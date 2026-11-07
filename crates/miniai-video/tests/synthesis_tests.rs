use miniai_video::buffer_pool::SharedFrameBufferPool;
use miniai_video::sanitizer::VideoSanitizer;
use miniai_video::synthesis::VideoSynthesisEngine;
use miniai_video::types::{BufferPoolConfig, CameraMotion, SanitizerConfig, VideoSynthesisConfig};

#[test]
fn test_text_to_video_synthesis() {
    let pool = SharedFrameBufferPool::new(BufferPoolConfig::default());
    let synth = VideoSynthesisEngine::new(pool);

    let config = VideoSynthesisConfig {
        prompt: "Neon city skyline with glowing lights".to_string(),
        motion: CameraMotion::PanRight,
        width: 160,
        height: 120,
        duration_secs: 1.0,
        fps: 10.0,
        seed: Some(12345),
        start_frame: None,
        end_frame: None,
        generate_audio: true,
    };

    let result = synth.synthesize(&config).expect("Synthesize text to video");

    assert_eq!(result.metadata.width, 160);
    assert_eq!(result.metadata.height, 120);
    assert_eq!(result.metadata.total_frames, 10);
    assert!(result.metadata.has_audio);
    assert!(!result.keyframes.is_empty());
    assert!(!result.video_bytes.is_empty());
    assert!(result.audio.is_some());

    // Validate that output video container passes sanitizer
    let sanitized_meta =
        VideoSanitizer::validate_bytes(&result.video_bytes, &SanitizerConfig::default())
            .expect("Synthesized video must pass sanitizer");
    assert_eq!(sanitized_meta.width, 160);
    assert_eq!(sanitized_meta.height, 120);
}

#[test]
fn test_image_to_video_motion_synthesis() {
    let pool = SharedFrameBufferPool::new(BufferPoolConfig::default());
    let synth = VideoSynthesisEngine::new(pool);

    let width = 64u32;
    let height = 64u32;
    let start_frame = vec![120u8; (width * height * 3) as usize];

    let config = VideoSynthesisConfig {
        prompt: "Camera zooming into portrait".to_string(),
        motion: CameraMotion::ZoomIn,
        width,
        height,
        duration_secs: 1.0,
        fps: 10.0,
        seed: Some(54321),
        start_frame: Some(start_frame),
        end_frame: None,
        generate_audio: false,
    };

    let result = synth
        .synthesize(&config)
        .expect("Synthesize image to video");

    assert_eq!(result.metadata.width, 64);
    assert_eq!(result.metadata.height, 64);
    assert!(!result.metadata.has_audio);
    assert!(!result.keyframes.is_empty());
}

#[test]
fn test_two_keyframe_morph_synthesis() {
    let pool = SharedFrameBufferPool::new(BufferPoolConfig::default());
    let synth = VideoSynthesisEngine::new(pool);

    let width = 32u32;
    let height = 32u32;
    let start_frame = vec![0u8; (width * height * 3) as usize];
    let end_frame = vec![255u8; (width * height * 3) as usize];

    let config = VideoSynthesisConfig {
        prompt: "Smooth daylight to twilight transition".to_string(),
        motion: CameraMotion::Static,
        width,
        height,
        duration_secs: 1.0,
        fps: 5.0,
        seed: Some(999),
        start_frame: Some(start_frame),
        end_frame: Some(end_frame),
        generate_audio: true,
    };

    let result = synth.synthesize(&config).expect("Morph synthesis");

    assert_eq!(result.metadata.total_frames, 5);
    assert!(result.keyframes.len() >= 2);

    // Initial frame should be darker than last frame
    let first_lum = result.keyframes.first().unwrap().luminance_mean;
    let last_lum = result.keyframes.last().unwrap().luminance_mean;
    assert!(last_lum > first_lum);
}

#[test]
fn test_synthesis_determinism_with_seed() {
    let pool1 = SharedFrameBufferPool::new(BufferPoolConfig::default());
    let synth1 = VideoSynthesisEngine::new(pool1);

    let pool2 = SharedFrameBufferPool::new(BufferPoolConfig::default());
    let synth2 = VideoSynthesisEngine::new(pool2);

    let config = VideoSynthesisConfig {
        prompt: "Deterministic mountain sunrise".to_string(),
        motion: CameraMotion::PanLeft,
        width: 64,
        height: 64,
        duration_secs: 0.5,
        fps: 10.0,
        seed: Some(777),
        start_frame: None,
        end_frame: None,
        generate_audio: true,
    };

    let res1 = synth1.synthesize(&config).expect("run 1");
    let res2 = synth2.synthesize(&config).expect("run 2");

    assert_eq!(res1.metadata.sha256_hash, res2.metadata.sha256_hash);
    assert_eq!(res1.video_bytes, res2.video_bytes);
}

#[test]
fn test_synthesized_audio_interoperability_with_speech_asr() {
    let pool = SharedFrameBufferPool::new(BufferPoolConfig::default());
    let synth = VideoSynthesisEngine::new(pool);

    let config = VideoSynthesisConfig {
        prompt: "Audio video sync check".to_string(),
        motion: CameraMotion::Static,
        width: 32,
        height: 32,
        duration_secs: 1.0,
        fps: 5.0,
        seed: Some(42),
        start_frame: None,
        end_frame: None,
        generate_audio: true,
    };

    let result = synth.synthesize(&config).expect("synth");
    let audio = result.audio.expect("Must have audio");

    let (asr_meta, samples) =
        miniai_speech_asr::AudioSanitizer::sanitize_and_decode(&audio.wav_bytes)
            .expect("Audio must be valid RIFF/WAVE");

    assert_eq!(asr_meta.sample_rate, 16000);
    assert_eq!(asr_meta.channels, 1);
    assert!(!samples.is_empty());
}
