use serde::{Deserialize, Serialize};

/// Supported video container formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoContainerFormat {
    Mp4,
    WebM,
    RawFrames,
}

impl std::fmt::Display for VideoContainerFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mp4 => write!(f, "mp4"),
            Self::WebM => write!(f, "webm"),
            Self::RawFrames => write!(f, "raw_frames"),
        }
    }
}

/// Supported video codecs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoCodec {
    H264,
    H265,
    Vp8,
    Vp9,
    Av1,
    RawRgb,
}

impl std::fmt::Display for VideoCodec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::H264 => write!(f, "h264"),
            Self::H265 => write!(f, "h265"),
            Self::Vp8 => write!(f, "vp8"),
            Self::Vp9 => write!(f, "vp9"),
            Self::Av1 => write!(f, "av1"),
            Self::RawRgb => write!(f, "raw_rgb"),
        }
    }
}

/// Video track and container metadata extracted during sanitization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub container_format: VideoContainerFormat,
    pub codec: VideoCodec,
    pub width: u32,
    pub height: u32,
    pub duration_secs: f64,
    pub fps: f64,
    pub total_frames: usize,
    pub has_audio: bool,
    pub audio_channels: u16,
    pub audio_sample_rate: u32,
    pub file_size_bytes: usize,
    pub sha256_hash: String,
}

/// A single extracted video frame with RGB pixel payload and metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoFrame {
    pub frame_index: usize,
    pub timestamp_ms: u64,
    pub width: u32,
    pub height: u32,
    /// RGB24 raw pixel buffer (length = width * height * 3).
    pub rgb_data: Vec<u8>,
    pub is_keyframe: bool,
    pub scene_id: usize,
    pub luminance_mean: f32,
    pub sha256: String,
}

impl VideoFrame {
    pub fn byte_size(&self) -> usize {
        self.rgb_data.len() + std::mem::size_of::<Self>()
    }
}

/// Keyframe sampling strategy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyframeExtractionMode {
    /// Sample every N frames uniformly.
    UniformInterval { step_frames: usize },
    /// Sample every N milliseconds.
    UniformDuration { interval_ms: u64 },
    /// Extract keyframes based on visual scene cut / histogram delta exceeding threshold.
    SceneBoundaryChange { threshold: f32 },
    /// Adaptive sampling with a maximum keyframe count cap and scene boundary threshold.
    AdaptiveDynamic {
        max_keyframes: usize,
        threshold: f32,
    },
}

impl Default for KeyframeExtractionMode {
    fn default() -> Self {
        Self::AdaptiveDynamic {
            max_keyframes: 16,
            threshold: 0.25,
        }
    }
}

/// A coherent temporal scene segment in a video.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneSegment {
    pub scene_id: usize,
    pub start_ms: u64,
    pub end_ms: u64,
    pub keyframe_index: usize,
    pub scene_type: String,
    pub visual_summary: String,
    pub detected_actions: Vec<String>,
    pub confidence: f32,
}

/// An action or movement detected over a temporal window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionDetection {
    pub action: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub confidence: f32,
    pub movement_magnitude: f32,
}

/// Temporal Video Question Answering query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalQaQuery {
    pub question: String,
    pub max_candidate_scenes: usize,
}

impl TemporalQaQuery {
    pub fn new(question: impl Into<String>) -> Self {
        Self {
            question: question.into(),
            max_candidate_scenes: 5,
        }
    }
}

/// Grounded Temporal Video Question Answering result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalQaResult {
    pub question: String,
    pub answer: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub confidence: f32,
    pub supporting_scene_id: usize,
    pub supporting_frame_indices: Vec<usize>,
}

/// Comprehensive video analysis result produced by the engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoAnalysisResult {
    pub metadata: VideoMetadata,
    pub keyframes: Vec<VideoFrame>,
    pub scenes: Vec<SceneSegment>,
    pub overall_summary: String,
    pub detected_actions: Vec<ActionDetection>,
    pub processing_time_ms: u64,
}

/// Audio stream demuxed from a video container.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DemuxedAudio {
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_secs: f64,
    pub pcm_samples: Vec<f32>,
    pub wav_bytes: Vec<u8>,
}

/// Configuration for the memory-bounded frame buffer pool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BufferPoolConfig {
    /// Maximum allowed memory footprint for cached/in-flight frames in bytes (default: 64 MB).
    pub budget_bytes: usize,
    /// Maximum number of active frames in memory at any one time (default: 32).
    pub max_cached_frames: usize,
}

impl Default for BufferPoolConfig {
    fn default() -> Self {
        Self {
            budget_bytes: 64 * 1024 * 1024, // 64 MB
            max_cached_frames: 32,
        }
    }
}

/// Telemetry metrics for the frame buffer pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BufferPoolStats {
    pub allocated_bytes: usize,
    pub budget_bytes: usize,
    pub active_frames: usize,
    pub peak_bytes: usize,
}

/// Safety and validation limits for input video payloads.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SanitizerConfig {
    pub max_file_size_bytes: usize,
    pub max_duration_secs: f64,
    pub max_dimension: u32,
    pub min_fps: f64,
    pub max_fps: f64,
}

impl Default for SanitizerConfig {
    fn default() -> Self {
        Self {
            max_file_size_bytes: 100 * 1024 * 1024, // 100 MB
            max_duration_secs: 300.0,               // 5 minutes
            max_dimension: 1920,                    // 1080p width/height
            min_fps: 1.0,
            max_fps: 60.0,
        }
    }
}
