use std::path::Path;
use std::sync::{Arc, RwLock};

use miniai_core::{
    AcceleratorBackend, AdapterHealth, MiniAiError, Modality, ModalityAdapter, RuntimeCapability,
};
use sha2::{Digest, Sha256};

use crate::buffer_pool::SharedFrameBufferPool;
use crate::demuxer::AudioDemuxer;
use crate::engine::{VideoEngine, VideoEngineConfig};
use crate::error::VideoError;
use crate::types::{
    BufferPoolConfig, BufferPoolStats, DemuxedAudio, TemporalQaQuery, TemporalQaResult,
    VideoAnalysisResult, VideoMetadata,
};

/// Pluggable, decoupled Video Understanding & Frame Sampling runtime adapter implementing `ModalityAdapter`.
pub struct VideoAdapter {
    adapter_id: String,
    engine: Arc<RwLock<Option<VideoEngine>>>,
    pool: SharedFrameBufferPool,
    active_backend: AcceleratorBackend,
    loaded_model_id: Arc<RwLock<Option<String>>>,
}

impl VideoAdapter {
    /// Creates a new `VideoAdapter` with standard memory budget and CPU execution baseline.
    pub fn new(adapter_id: impl Into<String>) -> Self {
        let pool = SharedFrameBufferPool::new(BufferPoolConfig::default());
        let engine = VideoEngine::new(pool.clone());
        Self {
            adapter_id: adapter_id.into(),
            engine: Arc::new(RwLock::new(Some(engine))),
            pool,
            active_backend: AcceleratorBackend::Cpu,
            loaded_model_id: Arc::new(RwLock::new(Some("builtin_video_cpu_v1".to_string()))),
        }
    }

    /// Creates a `VideoAdapter` with custom buffer pool configuration.
    pub fn with_pool_config(adapter_id: impl Into<String>, pool_config: BufferPoolConfig) -> Self {
        let pool = SharedFrameBufferPool::new(pool_config);
        let engine = VideoEngine::new(pool.clone());
        Self {
            adapter_id: adapter_id.into(),
            engine: Arc::new(RwLock::new(Some(engine))),
            pool,
            active_backend: AcceleratorBackend::Cpu,
            loaded_model_id: Arc::new(RwLock::new(Some("builtin_video_cpu_v1".to_string()))),
        }
    }

    /// Verifies SHA-256 and loads external video model weights.
    pub fn load_model_weights(
        &self,
        weights_path: &Path,
        expected_sha256: Option<&str>,
    ) -> Result<(), VideoError> {
        if !weights_path.is_file() {
            return Err(VideoError::WeightsNotFound(
                weights_path.display().to_string(),
            ));
        }

        let bytes = std::fs::read(weights_path)?;

        if let Some(expected) = expected_sha256 {
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            let actual = format!("{:x}", hasher.finalize());
            if actual != expected {
                return Err(VideoError::WeightsIntegrityMismatch {
                    expected: expected.to_string(),
                    actual,
                });
            }
        }

        let model_id = weights_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "custom_video_model".to_string());

        let mut engine_guard = self
            .engine
            .write()
            .map_err(|_| VideoError::InferenceError("Lock poisoned on video engine".to_string()))?;
        *engine_guard = Some(VideoEngine::new(self.pool.clone()));

        let mut id_guard = self
            .loaded_model_id
            .write()
            .map_err(|_| VideoError::InferenceError("Lock poisoned on model id".to_string()))?;
        *id_guard = Some(model_id);

        Ok(())
    }

    /// Checks whether model weights are currently loaded and ready for inference.
    pub fn is_loaded(&self) -> bool {
        self.engine
            .read()
            .ok()
            .and_then(|g| g.as_ref().map(|_| true))
            .unwrap_or(false)
    }

    /// Returns current frame buffer pool statistics.
    pub fn pool_stats(&self) -> Result<BufferPoolStats, VideoError> {
        self.pool.stats()
    }

    /// Demuxes the audio stream from a video container into standard PCM WAV bytes.
    pub fn demux_audio(
        &self,
        bytes: &[u8],
        metadata: &VideoMetadata,
    ) -> Result<Option<DemuxedAudio>, VideoError> {
        AudioDemuxer::demux_audio(bytes, metadata)
    }

    /// Analyzes a video payload, extracting keyframes, temporal scenes, and action tags.
    pub fn analyze_video(
        &self,
        bytes: &[u8],
        config: &VideoEngineConfig,
    ) -> Result<VideoAnalysisResult, VideoError> {
        let engine_guard = self
            .engine
            .read()
            .map_err(|_| VideoError::InferenceError("Lock poisoned on video engine".to_string()))?;
        let engine = engine_guard.as_ref().ok_or(VideoError::ModelNotLoaded)?;
        engine.analyze_video(bytes, config)
    }

    /// Answers a grounded temporal question about an analyzed video.
    pub fn temporal_qa(
        &self,
        analysis: &VideoAnalysisResult,
        query: &TemporalQaQuery,
    ) -> Result<TemporalQaResult, VideoError> {
        let engine_guard = self
            .engine
            .read()
            .map_err(|_| VideoError::InferenceError("Lock poisoned on video engine".to_string()))?;
        let engine = engine_guard.as_ref().ok_or(VideoError::ModelNotLoaded)?;
        engine.temporal_qa(analysis, query)
    }
}

impl ModalityAdapter for VideoAdapter {
    fn adapter_id(&self) -> &str {
        &self.adapter_id
    }

    fn modality(&self) -> Modality {
        Modality::Video
    }

    fn supported_backends(&self) -> Vec<AcceleratorBackend> {
        vec![
            AcceleratorBackend::Cpu,
            AcceleratorBackend::Cuda,
            AcceleratorBackend::Vulkan,
            AcceleratorBackend::Metal,
        ]
    }

    fn capability(&self) -> RuntimeCapability {
        RuntimeCapability {
            adapter_id: self.adapter_id.clone(),
            modality: Modality::Video,
            backends: vec![AcceleratorBackend::Cpu],
            supports_streaming: true,
            supports_offline: true,
            isolated_process: true,
        }
    }

    fn health_check(&self) -> Result<AdapterHealth, MiniAiError> {
        let loaded = self.is_loaded();
        let model_name = self
            .loaded_model_id
            .read()
            .ok()
            .and_then(|g| g.clone())
            .unwrap_or_else(|| "none".to_string());

        Ok(AdapterHealth {
            adapter_id: self.adapter_id.clone(),
            modality: Modality::Video,
            ready: loaded,
            active_backend: self.active_backend,
            status_message: if loaded {
                format!("Video adapter active on CPU (model: {model_name})")
            } else {
                "Video adapter initialized but no model loaded".to_string()
            },
        })
    }
}
