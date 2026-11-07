use crate::types::{SpeechSegment, VadFrame};

pub const VAD_FRAME_LENGTH: usize = 480; // 30ms at 16kHz
pub const VAD_FRAME_STEP: usize = 160; // 10ms at 16kHz
pub const MIN_SPEECH_DURATION_SECS: f32 = 0.10; // 100ms
pub const HANGOVER_FRAMES: usize = 20; // 200ms gap bridging

pub struct VadEngine {
    frame_length: usize,
    frame_step: usize,
}

impl Default for VadEngine {
    fn default() -> Self {
        Self {
            frame_length: VAD_FRAME_LENGTH,
            frame_step: VAD_FRAME_STEP,
        }
    }
}

impl VadEngine {
    pub fn new(frame_length: usize, frame_step: usize) -> Self {
        Self {
            frame_length,
            frame_step,
        }
    }

    /// Evaluates Voice Activity across standard 16 kHz mono audio buffer.
    pub fn detect_speech_frames(&self, samples: &[f32]) -> Vec<VadFrame> {
        if samples.len() < self.frame_length {
            return Vec::new();
        }

        let num_frames = (samples.len() - self.frame_length) / self.frame_step + 1;
        let mut raw_frames = Vec::with_capacity(num_frames);

        for i in 0..num_frames {
            let start_idx = i * self.frame_step;
            let end_idx = start_idx + self.frame_length;
            let frame_slice = &samples[start_idx..end_idx];

            let start_secs = start_idx as f32 / 16000.0;
            let end_secs = end_idx as f32 / 16000.0;

            // Compute Short-Time Energy (STE)
            let energy: f32 =
                frame_slice.iter().map(|&s| s * s).sum::<f32>() / (self.frame_length as f32);

            // Compute Zero-Crossing Rate (ZCR)
            let mut zero_crossings = 0;
            for j in 1..frame_slice.len() {
                if (frame_slice[j] >= 0.0 && frame_slice[j - 1] < 0.0)
                    || (frame_slice[j] < 0.0 && frame_slice[j - 1] >= 0.0)
                {
                    zero_crossings += 1;
                }
            }
            let zcr = zero_crossings as f32 / (self.frame_length as f32);

            raw_frames.push(VadFrame {
                start_secs,
                end_secs,
                energy,
                zero_crossing_rate: zcr,
                is_speech: false,
            });
        }

        if raw_frames.is_empty() {
            return Vec::new();
        }

        // Estimate adaptive noise floor from lower 15th percentile of energies
        let mut energies: Vec<f32> = raw_frames.iter().map(|f| f.energy).collect();
        energies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let p15_idx = (energies.len() as f32 * 0.15) as usize;
        let noise_floor = energies[p15_idx.min(energies.len() - 1)];

        let energy_thresh = if noise_floor > 0.005 {
            0.002
        } else {
            (noise_floor * 2.5).max(0.0008)
        };

        for frame in raw_frames.iter_mut() {
            let energetic = frame.energy > energy_thresh;
            let fricative = frame.energy > (energy_thresh * 0.4) && frame.zero_crossing_rate > 0.12;
            frame.is_speech = energetic || fricative;
        }

        raw_frames
    }

    /// Segments audio into continuous speech intervals with hangover smoothing.
    pub fn segment_speech(&self, samples: &[f32]) -> Vec<SpeechSegment> {
        let frames = self.detect_speech_frames(samples);
        if frames.is_empty() {
            return Vec::new();
        }

        let mut segments = Vec::new();
        let mut in_speech = false;
        let mut segment_start = 0.0f32;
        let mut segment_end = 0.0f32;
        let mut silence_count = 0;
        let mut segment_id = 0;

        for frame in frames {
            if frame.is_speech {
                if !in_speech {
                    in_speech = true;
                    segment_start = frame.start_secs;
                }
                segment_end = frame.end_secs;
                silence_count = 0;
            } else if in_speech {
                silence_count += 1;
                if silence_count > HANGOVER_FRAMES {
                    // Finalize speech segment
                    if (segment_end - segment_start) >= MIN_SPEECH_DURATION_SECS {
                        segments.push(SpeechSegment {
                            id: segment_id,
                            start_secs: (segment_start * 100.0).round() / 100.0,
                            end_secs: (segment_end * 100.0).round() / 100.0,
                            text: String::new(),
                            confidence: 0.95,
                            tokens: Vec::new(),
                        });
                        segment_id += 1;
                    }
                    in_speech = false;
                    silence_count = 0;
                }
            }
        }

        // Close trailing speech segment
        if in_speech && (segment_end - segment_start) >= MIN_SPEECH_DURATION_SECS {
            segments.push(SpeechSegment {
                id: segment_id,
                start_secs: (segment_start * 100.0).round() / 100.0,
                end_secs: (segment_end * 100.0).round() / 100.0,
                text: String::new(),
                confidence: 0.95,
                tokens: Vec::new(),
            });
        }

        segments
    }
}
