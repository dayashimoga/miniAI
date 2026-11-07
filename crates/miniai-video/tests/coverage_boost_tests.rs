use miniai_core::ModalityAdapter;
use miniai_video::adapter::VideoAdapter;
use miniai_video::buffer_pool::SharedFrameBufferPool;
use miniai_video::demuxer::AudioDemuxer;
use miniai_video::extractor::{AtomicWorkspaceManager, FrameExtractor};
use miniai_video::sanitizer::VideoSanitizer;
use miniai_video::types::{
    BufferPoolConfig, KeyframeExtractionMode, SanitizerConfig, VideoCodec, VideoContainerFormat,
    VideoFrame,
};

#[test]
fn test_display_and_helpers() {
    assert_eq!(format!("{}", VideoContainerFormat::Mp4), "mp4");
    assert_eq!(format!("{}", VideoContainerFormat::WebM), "webm");
    assert_eq!(format!("{}", VideoContainerFormat::RawFrames), "raw_frames");

    assert_eq!(format!("{}", VideoCodec::H264), "h264");
    assert_eq!(format!("{}", VideoCodec::H265), "h265");
    assert_eq!(format!("{}", VideoCodec::Vp8), "vp8");
    assert_eq!(format!("{}", VideoCodec::Vp9), "vp9");
    assert_eq!(format!("{}", VideoCodec::Av1), "av1");
    assert_eq!(format!("{}", VideoCodec::RawRgb), "raw_rgb");

    let default_mode = KeyframeExtractionMode::default();
    assert!(matches!(
        default_mode,
        KeyframeExtractionMode::AdaptiveDynamic { .. }
    ));

    let frame = VideoFrame {
        frame_index: 0,
        timestamp_ms: 0,
        width: 10,
        height: 10,
        rgb_data: vec![0u8; 300],
        is_keyframe: true,
        scene_id: 1,
        luminance_mean: 0.5,
        sha256: "hash".to_string(),
    };
    assert!(frame.byte_size() >= 300);
}

#[test]
fn test_sanitizer_predicates_and_codecs() {
    assert!(VideoSanitizer::is_mp4(b"....ftypisom"));
    assert!(!VideoSanitizer::is_mp4(b"short"));

    assert!(VideoSanitizer::is_webm(&[0x1A, 0x45, 0xDF, 0xA3]));
    assert!(!VideoSanitizer::is_webm(b"RIFF"));

    assert!(VideoSanitizer::is_rawv(b"RAWVheader"));
    assert!(!VideoSanitizer::is_rawv(b"no"));

    // MP4 with H265 brand
    let mut mp4_h265 = Vec::new();
    mp4_h265.extend_from_slice(&20u32.to_be_bytes());
    mp4_h265.extend_from_slice(b"ftyp");
    mp4_h265.extend_from_slice(b"hvc1");
    mp4_h265.extend_from_slice(&0u32.to_be_bytes());
    mp4_h265.extend_from_slice(b"mp41");
    let meta =
        VideoSanitizer::validate_bytes(&mp4_h265, &SanitizerConfig::default()).expect("h265");
    assert_eq!(meta.codec, VideoCodec::H265);

    // MP4 with AV1 brand
    let mut mp4_av1 = Vec::new();
    mp4_av1.extend_from_slice(&20u32.to_be_bytes());
    mp4_av1.extend_from_slice(b"ftyp");
    mp4_av1.extend_from_slice(b"av01");
    mp4_av1.extend_from_slice(&0u32.to_be_bytes());
    mp4_av1.extend_from_slice(b"mp41");
    let meta_av1 =
        VideoSanitizer::validate_bytes(&mp4_av1, &SanitizerConfig::default()).expect("av1");
    assert_eq!(meta_av1.codec, VideoCodec::Av1);

    // MP4 with VP9 brand
    let mut mp4_vp9 = Vec::new();
    mp4_vp9.extend_from_slice(&20u32.to_be_bytes());
    mp4_vp9.extend_from_slice(b"ftyp");
    mp4_vp9.extend_from_slice(b"vp09");
    mp4_vp9.extend_from_slice(&0u32.to_be_bytes());
    mp4_vp9.extend_from_slice(b"mp41");
    let meta_vp9 =
        VideoSanitizer::validate_bytes(&mp4_vp9, &SanitizerConfig::default()).expect("vp9");
    assert_eq!(meta_vp9.codec, VideoCodec::Vp9);

    // WebM with VP8 and Vorbis
    let mut webm_vp8 = Vec::new();
    webm_vp8.extend_from_slice(&[0x1A, 0x45, 0xDF, 0xA3]);
    webm_vp8.extend_from_slice(b"V_VP8");
    webm_vp8.extend_from_slice(b"A_VORB");
    let meta_webm =
        VideoSanitizer::validate_bytes(&webm_vp8, &SanitizerConfig::default()).expect("webm vp8");
    assert_eq!(meta_webm.codec, VideoCodec::Vp8);
    assert!(meta_webm.has_audio);

    // WebM with AV1
    let mut webm_av1 = Vec::new();
    webm_av1.extend_from_slice(&[0x1A, 0x45, 0xDF, 0xA3]);
    webm_av1.extend_from_slice(b"V_AV1");
    let meta_webm_av1 =
        VideoSanitizer::validate_bytes(&webm_av1, &SanitizerConfig::default()).expect("webm av1");
    assert_eq!(meta_webm_av1.codec, VideoCodec::Av1);
}

#[test]
fn test_shared_pool_extended_methods() {
    let pool = SharedFrameBufferPool::new(BufferPoolConfig::default());
    let buf = pool.allocate_buffer(32, 32).expect("alloc");
    let frame = VideoFrame {
        frame_index: 0,
        timestamp_ms: 0,
        width: 32,
        height: 32,
        rgb_data: buf,
        is_keyframe: true,
        scene_id: 1,
        luminance_mean: 0.5,
        sha256: "h".to_string(),
    };
    pool.release_frame(frame);
    pool.shrink_to_fit();
    assert_eq!(pool.stats().expect("stats").active_frames, 0);
}

#[test]
fn test_adapter_with_pool_config_and_status() {
    let adapter =
        VideoAdapter::with_pool_config("custom-video-adapter", BufferPoolConfig::default());
    assert_eq!(adapter.adapter_id(), "custom-video-adapter");
    assert!(adapter.is_loaded());
}

#[test]
fn test_explicit_workspace_cleanup() {
    let mut ws = AtomicWorkspaceManager::new(None).expect("create ws");
    let p = ws.path().to_path_buf();
    assert!(p.exists());
    ws.cleanup().expect("cleanup");
    assert!(!p.exists());
}

#[test]
fn test_demuxer_with_embedded_audo_chunk() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&20u32.to_be_bytes());
    bytes.extend_from_slice(b"ftypisom");
    bytes.extend_from_slice(&0u32.to_be_bytes());
    bytes.extend_from_slice(b"mp41");
    bytes.extend_from_slice(b"AUDO");
    // Add 100 PCM 16-bit samples
    for i in 0..100i16 {
        bytes.extend_from_slice(&i.to_le_bytes());
    }

    let meta = VideoSanitizer::validate_bytes(&bytes, &SanitizerConfig::default()).expect("meta");
    let mut meta_with_audio = meta.clone();
    meta_with_audio.has_audio = true;
    meta_with_audio.audio_sample_rate = 16000;
    meta_with_audio.audio_channels = 1;
    meta_with_audio.duration_secs = 0.5;

    let demuxed = AudioDemuxer::demux_audio(&bytes, &meta_with_audio)
        .expect("demux")
        .expect("has audio");
    assert!(!demuxed.pcm_samples.is_empty());
}

#[test]
fn test_empty_frame_delta_and_signature() {
    let f1 = VideoFrame {
        frame_index: 0,
        timestamp_ms: 0,
        width: 0,
        height: 0,
        rgb_data: Vec::new(),
        is_keyframe: false,
        scene_id: 0,
        luminance_mean: 0.0,
        sha256: "".to_string(),
    };
    let f2 = f1.clone();
    assert_eq!(FrameExtractor::compute_frame_delta(&f1, &f2), 0.0);
    assert_eq!(FrameExtractor::compute_signature_delta(&[], &[]), 0.0);
    assert!(FrameExtractor::create_frame_signature(&[]).is_empty());
}
