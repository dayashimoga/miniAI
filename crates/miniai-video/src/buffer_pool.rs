use std::sync::{Arc, Mutex};

use crate::error::VideoError;
use crate::types::{BufferPoolConfig, BufferPoolStats, VideoFrame};

/// Memory-bounded frame buffer pool protecting host system from OOM.
#[derive(Debug)]
pub struct FrameBufferPool {
    config: BufferPoolConfig,
    allocated_bytes: usize,
    active_frames: usize,
    peak_bytes: usize,
    free_buffers: Vec<Vec<u8>>,
}

impl FrameBufferPool {
    /// Creates a new buffer pool with the given configuration.
    pub fn new(config: BufferPoolConfig) -> Self {
        Self {
            config,
            allocated_bytes: 0,
            active_frames: 0,
            peak_bytes: 0,
            free_buffers: Vec::new(),
        }
    }

    /// Allocates an RGB pixel buffer for a frame of the specified dimensions.
    pub fn allocate_buffer(&mut self, width: u32, height: u32) -> Result<Vec<u8>, VideoError> {
        let required_bytes = (width as usize)
            .checked_mul(height as usize)
            .and_then(|wh| wh.checked_mul(3))
            .ok_or(VideoError::ResolutionExceeded {
                width,
                height,
                max_dim: self.config.budget_bytes as u32,
            })?;

        if self.allocated_bytes + required_bytes > self.config.budget_bytes {
            return Err(VideoError::BufferPoolExhausted {
                requested_bytes: required_bytes,
                available_bytes: self
                    .config
                    .budget_bytes
                    .saturating_sub(self.allocated_bytes),
                budget_bytes: self.config.budget_bytes,
            });
        }

        self.allocated_bytes += required_bytes;
        self.active_frames += 1;
        if self.allocated_bytes > self.peak_bytes {
            self.peak_bytes = self.allocated_bytes;
        }

        // Try to reuse an existing buffer with sufficient capacity
        if let Some(pos) = self
            .free_buffers
            .iter()
            .position(|b| b.capacity() >= required_bytes)
        {
            let mut buf = self.free_buffers.swap_remove(pos);
            buf.clear();
            buf.resize(required_bytes, 0);
            Ok(buf)
        } else {
            Ok(vec![0u8; required_bytes])
        }
    }

    /// Releases an allocated buffer back to the pool.
    pub fn release_buffer(&mut self, mut buffer: Vec<u8>) {
        let size = buffer.len();
        self.allocated_bytes = self.allocated_bytes.saturating_sub(size);
        self.active_frames = self.active_frames.saturating_sub(1);

        if self.free_buffers.len() < self.config.max_cached_frames {
            buffer.clear();
            self.free_buffers.push(buffer);
        }
    }

    /// Releases a `VideoFrame`'s payload back to the pool.
    pub fn release_frame(&mut self, frame: VideoFrame) {
        self.release_buffer(frame.rgb_data);
    }

    /// Returns current buffer pool telemetry metrics.
    pub fn stats(&self) -> BufferPoolStats {
        BufferPoolStats {
            allocated_bytes: self.allocated_bytes,
            budget_bytes: self.config.budget_bytes,
            active_frames: self.active_frames,
            peak_bytes: self.peak_bytes,
        }
    }

    /// Clears cached free buffers to immediately release unused memory back to OS.
    pub fn shrink_to_fit(&mut self) {
        self.free_buffers.clear();
        self.free_buffers.shrink_to_fit();
    }
}

/// Thread-safe wrapper for `FrameBufferPool`.
#[derive(Debug, Clone)]
pub struct SharedFrameBufferPool {
    inner: Arc<Mutex<FrameBufferPool>>,
}

impl SharedFrameBufferPool {
    pub fn new(config: BufferPoolConfig) -> Self {
        Self {
            inner: Arc::new(Mutex::new(FrameBufferPool::new(config))),
        }
    }

    pub fn allocate_buffer(&self, width: u32, height: u32) -> Result<Vec<u8>, VideoError> {
        let mut pool = self.inner.lock().map_err(|e| {
            VideoError::FrameExtractionError(format!("Buffer pool lock poisoned: {e}"))
        })?;
        pool.allocate_buffer(width, height)
    }

    pub fn release_buffer(&self, buffer: Vec<u8>) {
        if let Ok(mut pool) = self.inner.lock() {
            pool.release_buffer(buffer);
        }
    }

    pub fn release_frame(&self, frame: VideoFrame) {
        if let Ok(mut pool) = self.inner.lock() {
            pool.release_frame(frame);
        }
    }

    pub fn stats(&self) -> Result<BufferPoolStats, VideoError> {
        let pool = self.inner.lock().map_err(|e| {
            VideoError::FrameExtractionError(format!("Buffer pool lock poisoned: {e}"))
        })?;
        Ok(pool.stats())
    }

    pub fn shrink_to_fit(&self) {
        if let Ok(mut pool) = self.inner.lock() {
            pool.shrink_to_fit();
        }
    }
}
