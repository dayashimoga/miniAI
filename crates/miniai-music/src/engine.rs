use std::f32::consts::PI;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::error::MusicError;
use crate::types::{MusicGenConfig, MusicProgress, MusicTrackOutput, MusicalScale};
use crate::wav::WavEncoder;

/// Deterministic 64-bit Xorshift PRNG for musical noise and variation.
#[derive(Debug, Clone)]
pub struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0xfeedfac0ffee } else { seed },
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    /// Uniform float in [-1.0, 1.0].
    pub fn next_f32(&mut self) -> f32 {
        let u = (self.next_u64() >> 40) as u32;
        (u as f32 / 16777215.0) * 2.0 - 1.0
    }
}

/// Offline polyphonic acoustic music synthesis engine.
pub struct MusicGenEngine {
    model_id: String,
    weights_loaded: bool,
}

impl MusicGenEngine {
    /// Creates a new `MusicGenEngine` with an assigned model identifier.
    pub fn new(model_id: impl Into<String>) -> Self {
        Self {
            model_id: model_id.into(),
            weights_loaded: false,
        }
    }

    pub fn model_id(&self) -> &str {
        &self.model_id
    }

    pub fn is_weights_loaded(&self) -> bool {
        self.weights_loaded
    }

    pub fn set_weights_loaded(&mut self, loaded: bool) {
        self.weights_loaded = loaded;
    }

    /// Synthesizes polyphonic music across melody, harmony, bassline, and rhythm tracks.
    pub fn synthesize(
        &self,
        config: &MusicGenConfig,
        cancel_token: Option<&Arc<AtomicBool>>,
        progress_cb: Option<&dyn Fn(MusicProgress)>,
    ) -> Result<MusicTrackOutput, MusicError> {
        if !self.weights_loaded {
            return Err(MusicError::UnloadedModel);
        }

        config.validate()?;

        let sample_rate = config.sample_rate as f32;
        let total_samples = (config.duration_secs * sample_rate).round() as usize;
        let beat_duration_secs = 60.0 / config.tempo_bpm as f32;
        let samples_per_beat = (beat_duration_secs * sample_rate).round() as usize;
        let samples_per_measure = samples_per_beat * 4; // 4/4 time signature
        let total_measures = total_samples.div_ceil(samples_per_measure);

        let scale = MusicalScale::from_name(&config.scale_key);
        let mut rng = XorShift64::new(config.seed.unwrap_or(42));

        // Stereo audio accumulators
        let mut left_channel = vec![0.0f32; total_samples];
        let mut right_channel = vec![0.0f32; total_samples];

        // Standard 4-chord progression: I - V - vi - IV
        let chord_degrees = [0, 4, 5, 3];

        for measure in 0..total_measures {
            if let Some(token) = cancel_token {
                if token.load(Ordering::SeqCst) {
                    return Err(MusicError::Cancelled);
                }
            }

            if let Some(cb) = progress_cb {
                cb(MusicProgress {
                    stage: format!("Rendering measure {}/{}", measure + 1, total_measures),
                    current_measure: measure,
                    total_measures,
                    progress_ratio: measure as f32 / total_measures as f32,
                });
            }

            let measure_start = measure * samples_per_measure;
            if measure_start >= total_samples {
                break;
            }

            let degree = chord_degrees[measure % chord_degrees.len()];
            let triad = scale.chord_triad(degree);
            let bass_f0 = triad[0] * 0.5;

            // Render 4 beats in this measure
            for beat in 0..4 {
                let beat_start = measure_start + beat * samples_per_beat;
                if beat_start >= total_samples {
                    break;
                }

                // 1. Bassline Track (beats 0 and 2)
                if beat == 0 || beat == 2 {
                    let bass_len = (samples_per_beat * 2).min(total_samples - beat_start);
                    for s in 0..bass_len {
                        let t = s as f32 / sample_rate;
                        let env = (-2.0 * (s as f32 / bass_len as f32)).exp();
                        let wave =
                            (2.0 * PI * bass_f0 * t).sin() + 0.3 * (4.0 * PI * bass_f0 * t).sin();
                        let val = wave * env * 0.35;
                        left_channel[beat_start + s] += val * 0.5;
                        right_channel[beat_start + s] += val * 0.5;
                    }
                }

                // 2. Harmony Pad Track (sustained chord triads with stereo separation)
                let chord_len = samples_per_beat.min(total_samples - beat_start);
                for s in 0..chord_len {
                    let t = (beat_start + s) as f32 / sample_rate;
                    let env = Self::soft_attack_decay(s, chord_len);
                    let note1 = (2.0 * PI * triad[0] * t).sin();
                    let note2 = (2.0 * PI * triad[1] * t).sin();
                    let note3 = (2.0 * PI * triad[2] * t).sin();

                    // Stereo panning: left gets note1+note2, right gets note2+note3
                    let left_val = (note1 * 0.6 + note2 * 0.4) * env * 0.22;
                    let right_val = (note2 * 0.4 + note3 * 0.6) * env * 0.22;

                    left_channel[beat_start + s] += left_val;
                    right_channel[beat_start + s] += right_val;
                }

                // 3. Melody Arpeggio Track (two 8th notes per beat)
                let half_beat = samples_per_beat / 2;
                for sub in 0..2 {
                    let sub_start = beat_start + sub * half_beat;
                    if sub_start >= total_samples {
                        break;
                    }
                    let sub_len = half_beat.min(total_samples - sub_start);
                    let note_idx = (beat * 2 + sub + measure * 3) % scale.frequencies().len();
                    let melody_f0 = scale.frequencies()[note_idx] * 1.0;

                    for s in 0..sub_len {
                        let t = s as f32 / sample_rate;
                        let env = (-4.0 * (s as f32 / sub_len as f32)).exp();
                        let vibrato = 1.0 + 0.015 * (2.0 * PI * 5.0 * t).sin();
                        let wave = (2.0 * PI * melody_f0 * vibrato * t).sin();
                        let val = wave * env * 0.25;

                        // Panned slightly right (0.65 right, 0.35 left)
                        left_channel[sub_start + s] += val * 0.35;
                        right_channel[sub_start + s] += val * 0.65;
                    }
                }

                // 4. Rhythm Track: Kick on 0, 2; Snare on 1, 3; Hi-hat on every 8th note
                if beat == 0 || beat == 2 {
                    // Kick drum
                    let kick_len = (samples_per_beat / 3).min(total_samples - beat_start);
                    for s in 0..kick_len {
                        let t = s as f32 / sample_rate;
                        let freq_sweep = 120.0 * (-15.0 * t).exp() + 45.0;
                        let env = (-12.0 * t).exp();
                        let kick = (2.0 * PI * freq_sweep * t).sin() * env * 0.45;
                        left_channel[beat_start + s] += kick * 0.5;
                        right_channel[beat_start + s] += kick * 0.5;
                    }
                } else {
                    // Snare drum
                    let snare_len = (samples_per_beat / 3).min(total_samples - beat_start);
                    for s in 0..snare_len {
                        let t = s as f32 / sample_rate;
                        let env = (-10.0 * t).exp();
                        let noise = rng.next_f32() * env * 0.30;
                        left_channel[beat_start + s] += noise * 0.4;
                        right_channel[beat_start + s] += noise * 0.6;
                    }
                }

                // Hi-hat tick on every 8th note
                for sub in 0..2 {
                    let sub_start = beat_start + sub * half_beat;
                    if sub_start >= total_samples {
                        break;
                    }
                    let hat_len = (sample_rate * 0.03) as usize; // 30ms tick
                    let lim = hat_len.min(total_samples - sub_start);
                    for s in 0..lim {
                        let t = s as f32 / sample_rate;
                        let env = (-80.0 * t).exp();
                        let tick = rng.next_f32() * env * 0.15;
                        left_channel[sub_start + s] += tick * 0.6;
                        right_channel[sub_start + s] += tick * 0.4;
                    }
                }
            }
        }

        // Apply soft saturation limiter and interleave audio
        let mut interleaved = Vec::with_capacity(if config.channels == 2 {
            total_samples * 2
        } else {
            total_samples
        });

        for i in 0..total_samples {
            let l = left_channel[i];
            let r = right_channel[i];
            // Rational soft-saturation curve
            let l_sat = (l / (1.0 + l.abs())).clamp(-1.0, 1.0);
            let r_sat = (r / (1.0 + r.abs())).clamp(-1.0, 1.0);

            if config.channels == 2 {
                interleaved.push(l_sat);
                interleaved.push(r_sat);
            } else {
                interleaved.push(((l_sat + r_sat) * 0.5).clamp(-1.0, 1.0));
            }
        }

        if let Some(cb) = progress_cb {
            cb(MusicProgress {
                stage: "Encoding RIFF/WAVE container".to_string(),
                current_measure: total_measures,
                total_measures,
                progress_ratio: 1.0,
            });
        }

        let wav_bytes =
            WavEncoder::encode_pcm16(&interleaved, config.channels, config.sample_rate)?;

        let mut hasher = Sha256::new();
        hasher.update(&wav_bytes);
        let sha256 = format!("{:x}", hasher.finalize());

        Ok(MusicTrackOutput {
            wav_bytes,
            duration_secs: config.duration_secs,
            sample_rate: config.sample_rate,
            channels: config.channels,
            tempo_bpm: config.tempo_bpm,
            scale_key: scale.scale_name().to_string(),
            sha256,
            num_samples_per_channel: total_samples,
        })
    }

    fn soft_attack_decay(idx: usize, total: usize) -> f32 {
        if total <= 1 {
            return 1.0;
        }
        let pos = idx as f32 / total as f32;
        if pos < 0.15 {
            pos / 0.15
        } else if pos > 0.85 {
            (1.0 - pos) / 0.15
        } else {
            1.0
        }
    }
}
