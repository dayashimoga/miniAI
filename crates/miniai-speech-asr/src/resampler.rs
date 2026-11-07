use crate::types::AudioBuffer;

pub const STANDARD_SAMPLE_RATE: u32 = 16000;
pub const TARGET_RMS_LEVEL: f32 = 0.15;

pub struct AudioResampler;

impl AudioResampler {
    /// Full audio pipeline: downmixes to mono, resamples to 16 kHz, and normalizes RMS.
    pub fn process(samples: &[f32], source_rate: u32, channels: u16) -> AudioBuffer {
        let mono = Self::downmix_to_mono(samples, channels);
        let mut resampled = Self::resample_to_16k(&mono, source_rate);
        Self::normalize_rms(&mut resampled, TARGET_RMS_LEVEL);
        AudioBuffer::new(resampled, STANDARD_SAMPLE_RATE, 1)
    }

    /// Downmixes multi-channel interleaved audio samples to single-channel mono.
    pub fn downmix_to_mono(samples: &[f32], channels: u16) -> Vec<f32> {
        if channels <= 1 {
            return samples.to_vec();
        }

        let ch = channels as usize;
        let frame_count = samples.len() / ch;
        let mut mono = Vec::with_capacity(frame_count);

        for i in 0..frame_count {
            let mut sum = 0.0f32;
            let offset = i * ch;
            for c in 0..ch {
                sum += samples[offset + c];
            }
            mono.push(sum / (channels as f32));
        }

        mono
    }

    /// Resamples mono audio stream from arbitrary source sample rate to standard 16 kHz.
    pub fn resample_to_16k(mono_samples: &[f32], source_rate: u32) -> Vec<f32> {
        if source_rate == STANDARD_SAMPLE_RATE {
            return mono_samples.to_vec();
        }

        if mono_samples.is_empty() {
            return Vec::new();
        }

        let target_len = ((mono_samples.len() as f64) * (STANDARD_SAMPLE_RATE as f64)
            / (source_rate as f64))
            .round() as usize;

        if target_len == 0 {
            return Vec::new();
        }

        let mut resampled = Vec::with_capacity(target_len);
        let step = source_rate as f64 / STANDARD_SAMPLE_RATE as f64;

        for j in 0..target_len {
            let src_pos = (j as f64) * step;
            let idx = src_pos.floor() as usize;
            let frac = (src_pos - idx as f64) as f32;

            if idx >= mono_samples.len() {
                break;
            }

            let s0 = mono_samples[idx];
            let s1 = if idx + 1 < mono_samples.len() {
                mono_samples[idx + 1]
            } else {
                s0
            };

            let interpolated = s0 * (1.0 - frac) + s1 * frac;
            resampled.push(interpolated.clamp(-1.0, 1.0));
        }

        resampled
    }

    /// Normalizes audio buffer to target RMS level while preventing clipping.
    pub fn normalize_rms(samples: &mut [f32], target_rms: f32) {
        if samples.is_empty() {
            return;
        }

        let sum_sq: f32 = samples.iter().map(|&s| s * s).sum();
        let rms = (sum_sq / samples.len() as f32).sqrt();

        if rms > 1e-6 {
            let gain = (target_rms / rms).min(10.0); // Bound maximum gain boost to 10x
            for s in samples.iter_mut() {
                *s = (*s * gain).clamp(-1.0, 1.0);
            }
        }
    }
}
