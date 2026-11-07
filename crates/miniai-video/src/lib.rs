#![forbid(unsafe_code)]

pub mod adapter;
pub mod buffer_pool;
pub mod demuxer;
pub mod engine;
pub mod error;
pub mod extractor;
pub mod sanitizer;
pub mod synthesis;
pub mod types;
pub mod workflow;

pub use adapter::VideoAdapter;
pub use buffer_pool::{FrameBufferPool, SharedFrameBufferPool};
pub use demuxer::AudioDemuxer;
pub use engine::{VideoEngine, VideoEngineConfig};
pub use error::VideoError;
pub use extractor::{AtomicWorkspaceManager, FrameExtractor};
pub use sanitizer::VideoSanitizer;
pub use synthesis::VideoSynthesisEngine;
pub use types::{
    ActionDetection, BufferPoolConfig, BufferPoolStats, CameraMotion, DemuxedAudio,
    KeyframeExtractionMode, NodeExecutionStatus, NodeTelemetry, SanitizerConfig, SceneSegment,
    SynthesizedVideo, TemporalQaQuery, TemporalQaResult, VideoAnalysisResult, VideoCodec,
    VideoContainerFormat, VideoFrame, VideoMetadata, VideoSynthesisConfig, WorkflowData,
    WorkflowEdge, WorkflowExecutionResult, WorkflowNode, WorkflowNodeType,
};
pub use workflow::{MediaWorkflowGraph, WorkflowExecutor};
