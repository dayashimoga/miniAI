use sha2::{Digest, Sha256};

use crate::buffer_pool::SharedFrameBufferPool;
use crate::demuxer::AudioDemuxer;
use crate::error::VideoError;
use crate::types::{
    CameraMotion, DemuxedAudio, SynthesizedVideo, VideoCodec, VideoContainerFormat, VideoFrame,
    VideoMetadata, VideoSynthesisConfig,
};

/// Generates synthesized video animations from text prompts or input keyframe images.
pub struct VideoSynthesisEngine {
    pool: SharedFrameBufferPool,
}

impl VideoSynthesisEngine {
    pub fn new(pool: SharedFrameBufferPool) -> Self {
        Self { pool }
    }

    /// Synthesizes a video sequence according to the specified configuration.
    pub fn synthesize(
        &self,
        config: &VideoSynthesisConfig,
    ) -> Result<SynthesizedVideo, VideoError> {
        let total_frames = ((config.duration_secs * config.fps).round() as usize).max(2);
        let frame_duration_ms = if config.fps > 0.0 {
            (1000.0 / config.fps).round() as u64
        } else {
            66
        };

        let width = config.width;
        let height = config.height;
        let frame_bytes = (width as usize) * (height as usize) * 3;

        let seed = config.seed.unwrap_or_else(|| {
            config.prompt.bytes().fold(123456789u64, |acc, b| {
                acc.wrapping_mul(31).wrapping_add(b as u64)
            })
        });

        let mut keyframes: Vec<VideoFrame> = Vec::new();
        let mut raw_frames_payload = Vec::with_capacity(total_frames * frame_bytes);

        for frame_idx in 0..total_frames {
            let progress = (frame_idx as f32) / ((total_frames - 1) as f32);
            let ts_ms = (frame_idx as u64) * frame_duration_ms;

            let mut buffer = self.pool.allocate_buffer(width, height)?;

            // Render synthesized frame based on prompt, motion, and initial keyframe
            if let Some(ref start_img) = config.start_frame {
                if let Some(ref end_img) = config.end_frame {
                    // Two-keyframe morph interpolation
                    Self::render_morph(&mut buffer, start_img, end_img, progress, width, height);
                } else {
                    // Image-to-video motion animation
                    Self::render_motion(
                        &mut buffer,
                        start_img,
                        config.motion,
                        progress,
                        width,
                        height,
                    );
                }
            } else {
                // Text-to-video generative synthesis
                Self::render_generative_texture(
                    &mut buffer,
                    seed,
                    config.motion,
                    progress,
                    width,
                    height,
                );
            }

            // Append to raw frame payload for container encoding
            raw_frames_payload.extend_from_slice(&buffer);

            // Keep keyframes (first, middle, last, or every N frames)
            let is_keyframe =
                frame_idx == 0 || frame_idx == total_frames / 2 || frame_idx == total_frames - 1;

            if is_keyframe {
                let luminance = Self::compute_luminance(&buffer);
                let mut hasher = Sha256::new();
                hasher.update(&buffer);
                let sha256 = format!("{:x}", hasher.finalize());

                keyframes.push(VideoFrame {
                    frame_index: frame_idx,
                    timestamp_ms: ts_ms,
                    width,
                    height,
                    rgb_data: buffer,
                    is_keyframe: true,
                    scene_id: 1,
                    luminance_mean: luminance,
                    sha256,
                });
            } else {
                self.pool.release_buffer(buffer);
            }
        }

        // Generate matching audio track if requested
        let (audio, has_audio, audio_sample_rate, audio_channels) = if config.generate_audio {
            let sample_rate = 16000u32;
            let channels = 1u16;
            let total_samples = (config.duration_secs * sample_rate as f64).round() as usize;
            let mut pcm = Vec::with_capacity(total_samples);

            // Ambient background hum / synth sweep
            let base_freq = 110.0 + (seed % 110) as f32;
            for i in 0..total_samples {
                let t = (i as f32) / (sample_rate as f32);
                let freq = base_freq + (t / config.duration_secs as f32) * 55.0;
                let s = 0.3 * (2.0 * std::f32::consts::PI * freq * t).sin();
                pcm.push(s);
            }
            let wav_bytes = AudioDemuxer::encode_wav_16bit(&pcm, sample_rate, channels);
            (
                Some(DemuxedAudio {
                    sample_rate,
                    channels,
                    duration_secs: config.duration_secs,
                    pcm_samples: pcm,
                    wav_bytes,
                }),
                true,
                sample_rate,
                channels,
            )
        } else {
            (None, false, 0, 0)
        };

        // Package into standard RAWV video container
        let video_bytes = Self::encode_rawv_container(
            width,
            height,
            config.fps.round() as u32,
            total_frames as u32,
            has_audio,
            audio_sample_rate,
            audio_channels,
            &raw_frames_payload,
            audio.as_ref().map(|a| a.wav_bytes.as_slice()),
        );

        let mut hasher = Sha256::new();
        hasher.update(&video_bytes);
        let sha256_hash = format!("{:x}", hasher.finalize());

        let metadata = VideoMetadata {
            container_format: VideoContainerFormat::RawFrames,
            codec: VideoCodec::RawRgb,
            width,
            height,
            duration_secs: config.duration_secs,
            fps: config.fps,
            total_frames,
            has_audio,
            audio_channels,
            audio_sample_rate,
            file_size_bytes: video_bytes.len(),
            sha256_hash,
        };

        Ok(SynthesizedVideo {
            metadata,
            keyframes,
            video_bytes,
            audio,
        })
    }

    fn render_morph(
        buffer: &mut [u8],
        start_img: &[u8],
        end_img: &[u8],
        progress: f32,
        width: u32,
        height: u32,
    ) {
        let len = ((width * height * 3) as usize).min(buffer.len());
        let inv = 1.0 - progress;

        for i in 0..len {
            let s = if i < start_img.len() {
                start_img[i] as f32
            } else {
                0.0
            };
            let e = if i < end_img.len() {
                end_img[i] as f32
            } else {
                0.0
            };
            buffer[i] = ((s * inv) + (e * progress)).clamp(0.0, 255.0) as u8;
        }
    }

    fn render_motion(
        buffer: &mut [u8],
        start_img: &[u8],
        motion: CameraMotion,
        progress: f32,
        width: u32,
        height: u32,
    ) {
        let w = width as usize;
        let h = height as usize;

        let (dx, dy) = match motion {
            CameraMotion::PanLeft => (-((progress * 30.0) as i32), 0),
            CameraMotion::PanRight => (((progress * 30.0) as i32), 0),
            CameraMotion::TiltUp => (0, -((progress * 20.0) as i32)),
            CameraMotion::TiltDown => (0, ((progress * 20.0) as i32)),
            CameraMotion::ZoomIn | CameraMotion::Orbit => {
                (((progress * 15.0) as i32), ((progress * 10.0) as i32))
            }
            CameraMotion::ZoomOut => (
                -(((1.0 - progress) * 15.0) as i32),
                -(((1.0 - progress) * 10.0) as i32),
            ),
            CameraMotion::Static => (0, 0),
        };

        for y in 0..h {
            let src_y = ((y as i32 + dy).rem_euclid(h as i32)) as usize;
            for x in 0..w {
                let src_x = ((x as i32 + dx).rem_euclid(w as i32)) as usize;
                let dst_idx = (y * w + x) * 3;
                let src_idx = (src_y * w + src_x) * 3;

                if dst_idx + 2 < buffer.len() && src_idx + 2 < start_img.len() {
                    buffer[dst_idx] = start_img[src_idx];
                    buffer[dst_idx + 1] = start_img[src_idx + 1];
                    buffer[dst_idx + 2] = start_img[src_idx + 2];
                }
            }
        }
    }

    fn render_generative_texture(
        buffer: &mut [u8],
        seed: u64,
        motion: CameraMotion,
        progress: f32,
        width: u32,
        height: u32,
    ) {
        let w = width as usize;
        let h = height as usize;

        let base_r = ((seed.wrapping_add((progress * 100.0) as u64)) % 256) as u8;
        let base_g = (((seed >> 8).wrapping_add((progress * 80.0) as u64)) % 256) as u8;
        let base_b = (((seed >> 16).wrapping_add((progress * 140.0) as u64)) % 256) as u8;

        let motion_offset = match motion {
            CameraMotion::PanRight => (progress * (w as f32) * 0.2) as usize,
            CameraMotion::PanLeft => ((1.0 - progress) * (w as f32) * 0.2) as usize,
            _ => (progress * 15.0) as usize,
        };

        for y in 0..h {
            let row_offset = y * w * 3;
            let y_phase = ((y * 255) / h.max(1)) as u8;
            for x in 0..w {
                let idx = row_offset + (x * 3);
                let x_phase = (((x + motion_offset) * 255) / w.max(1)) as u8;
                if idx + 2 < buffer.len() {
                    buffer[idx] = base_r.wrapping_add(y_phase / 3);
                    buffer[idx + 1] = base_g.wrapping_add(x_phase / 4);
                    buffer[idx + 2] = base_b.wrapping_add(y_phase / 2);
                }
            }
        }
    }

    #[allow(clippy::chunks_exact_to_as_chunks)]
    fn compute_luminance(rgb_data: &[u8]) -> f32 {
        if rgb_data.is_empty() {
            return 0.0;
        }
        let mut sum = 0.0f32;
        let mut count = 0usize;
        for chunk in rgb_data.chunks_exact(3) {
            sum +=
                0.299 * (chunk[0] as f32) + 0.587 * (chunk[1] as f32) + 0.114 * (chunk[2] as f32);
            count += 1;
        }
        if count > 0 {
            sum / (count as f32 * 255.0)
        } else {
            0.0
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn encode_rawv_container(
        width: u32,
        height: u32,
        fps: u32,
        total_frames: u32,
        has_audio: bool,
        audio_sample_rate: u32,
        audio_channels: u16,
        raw_frames_payload: &[u8],
        audio_wav_payload: Option<&[u8]>,
    ) -> Vec<u8> {
        let mut container = Vec::with_capacity(37 + raw_frames_payload.len() + 1024);
        // Header
        container.extend_from_slice(b"RAWV");
        container.extend_from_slice(&1u16.to_be_bytes()); // version = 1
        container.extend_from_slice(&width.to_be_bytes());
        container.extend_from_slice(&height.to_be_bytes());
        container.extend_from_slice(&fps.to_be_bytes());
        container.extend_from_slice(&1u32.to_be_bytes()); // fps_den = 1
        container.extend_from_slice(&total_frames.to_be_bytes());
        container.push(if has_audio { 1 } else { 0 });
        container.extend_from_slice(&audio_sample_rate.to_be_bytes());
        container.extend_from_slice(&audio_channels.to_be_bytes());
        let duration_ms = if fps > 0 {
            ((total_frames as f64 / fps as f64) * 1000.0).round() as u32
        } else {
            1000
        };
        container.extend_from_slice(&duration_ms.to_be_bytes());

        // Video frames
        container.extend_from_slice(raw_frames_payload);

        // Audio chunk if present
        if let Some(audio_bytes) = audio_wav_payload {
            container.extend_from_slice(b"AUDO");
            // If WAV format, copy PCM samples past 44-byte WAV header
            if audio_bytes.len() > 44 && &audio_bytes[0..4] == b"RIFF" {
                container.extend_from_slice(&audio_bytes[44..]);
            } else {
                container.extend_from_slice(audio_bytes);
            }
        }

        container
    }
}
