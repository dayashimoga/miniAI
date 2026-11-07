use std::f32::consts::PI;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::error::SpeechTtsError;
use crate::normalizer::{PhonemeCategory, PhonemeToken, TextNormalizer};
use crate::types::{TtsAudioOutput, TtsProgress, TtsSynthesisConfig};
use crate::wav::WavEncoder;

/// Deterministic 64-bit Xorshift PRNG for noise synthesis.
#[derive(Debug, Clone)]
pub struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0xdeadbeefcafe } else { seed },
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

/// Neural and acoustic formant speech synthesis engine.
pub struct SpeechTtsEngine {
    model_id: String,
    weights_loaded: bool,
}

impl SpeechTtsEngine {
    /// Creates a new `SpeechTtsEngine` with a model identifier.
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

    /// Synthesizes text into high-fidelity acoustic speech audio.
    pub fn synthesize(
        &self,
        text: &str,
        config: &TtsSynthesisConfig,
        cancel_token: Option<&Arc<AtomicBool>>,
        progress_cb: Option<&dyn Fn(TtsProgress)>,
    ) -> Result<TtsAudioOutput, SpeechTtsError> {
        if !self.weights_loaded {
            return Err(SpeechTtsError::UnloadedModel);
        }

        config.validate()?;

        let chunks = TextNormalizer::chunk_text(text)?;
        let total_chunks = chunks.len();

        let mut all_samples: Vec<f32> = Vec::new();
        let mut total_phonemes = 0;

        let seed = config.seed.unwrap_or(42);
        let mut rng = XorShift64::new(seed);

        let sample_rate = config.sample_rate as f32;
        let base_f0 = config.voice.base_frequency_hz();
        let pitch_multiplier = 2.0f32.powf(config.pitch_semitones / 12.0);
        let effective_f0 = (base_f0 * pitch_multiplier).clamp(40.0, 600.0);
        let formant_scale = config.voice.formant_scale();
        let harmonic_decay = config.voice.harmonic_decay();

        // Continuous phase accumulator to prevent clicks between adjacent phonemes
        let mut current_phase = 0.0f32;

        for (chunk_idx, raw_chunk) in chunks.iter().enumerate() {
            if let Some(token) = cancel_token {
                if token.load(Ordering::SeqCst) {
                    return Err(SpeechTtsError::Cancelled);
                }
            }

            if let Some(cb) = progress_cb {
                cb(TtsProgress {
                    stage: format!("Synthesizing chunk {}/{}", chunk_idx + 1, total_chunks),
                    current_chunk: chunk_idx,
                    total_chunks,
                    progress_ratio: chunk_idx as f32 / total_chunks as f32,
                });
            }

            let normalized = TextNormalizer::normalize(raw_chunk);
            let phonemes = TextNormalizer::tokenize_phonemes(&normalized, config.speed);
            total_phonemes += phonemes.len();

            for phoneme in &phonemes {
                let phoneme_samples = Self::synthesize_phoneme(
                    phoneme,
                    sample_rate,
                    effective_f0,
                    formant_scale,
                    harmonic_decay,
                    &mut current_phase,
                    &mut rng,
                );
                all_samples.extend(phoneme_samples);
            }

            // Short sentence boundary pause (100ms)
            let pause_samples = (sample_rate * 0.1) as usize;
            all_samples.resize(all_samples.len() + pause_samples, 0.0);
        }

        // Apply volume gain and soft-saturation limiter to prevent clipping
        for s in &mut all_samples {
            let amplified = *s * config.volume_gain;
            // Soft-saturation curve
            *s = (amplified / (1.0 + amplified.abs())).clamp(-1.0, 1.0);
        }

        if let Some(cb) = progress_cb {
            cb(TtsProgress {
                stage: "Encoding RIFF/WAVE container".to_string(),
                current_chunk: total_chunks,
                total_chunks,
                progress_ratio: 1.0,
            });
        }

        let wav_bytes = WavEncoder::encode_pcm16_mono(&all_samples, config.sample_rate)?;
        let duration_secs = all_samples.len() as f32 / sample_rate;

        let mut hasher = Sha256::new();
        hasher.update(&wav_bytes);
        let sha256 = format!("{:x}", hasher.finalize());

        Ok(TtsAudioOutput {
            wav_bytes,
            pcm_samples: all_samples,
            duration_secs,
            sample_rate: config.sample_rate,
            channels: 1,
            sha256,
            phoneme_count: total_phonemes,
        })
    }

    fn synthesize_phoneme(
        phoneme: &PhonemeToken,
        sample_rate: f32,
        f0: f32,
        formant_scale: f32,
        harmonic_decay: f32,
        phase: &mut f32,
        rng: &mut XorShift64,
    ) -> Vec<f32> {
        let num_samples = ((phoneme.duration_ms / 1000.0) * sample_rate).round() as usize;
        if num_samples == 0 {
            return Vec::new();
        }

        let mut samples = Vec::with_capacity(num_samples);

        match phoneme.category {
            PhonemeCategory::Pause { .. } => {
                samples.resize(num_samples, 0.0);
            }
            PhonemeCategory::Vowel { f1, f2, f3 } => {
                let scaled_f1 = f1 as f32 * formant_scale;
                let scaled_f2 = f2 as f32 * formant_scale;
                let scaled_f3 = f3 as f32 * formant_scale;

                for i in 0..num_samples {
                    let env = Self::adsr_envelope(i, num_samples, 0.15, 0.15);
                    let mut sample = 0.0f32;

                    // Voiced harmonic series with formant resonance weighting
                    for k in 1..=5 {
                        let harm_f = f0 * k as f32;
                        if harm_f >= sample_rate * 0.45 {
                            break;
                        }

                        // Formant resonance boost
                        let r1 = 1.0 / (1.0 + 0.00015 * (harm_f - scaled_f1).powi(2));
                        let r2 = 0.8 / (1.0 + 0.00015 * (harm_f - scaled_f2).powi(2));
                        let r3 = 0.5 / (1.0 + 0.00015 * (harm_f - scaled_f3).powi(2));
                        let resonance = 0.3 + r1 + r2 + r3;

                        let harm_phase = *phase * k as f32;
                        let harm_amp = harmonic_decay.powi(k - 1) * resonance;
                        sample += harm_phase.sin() * harm_amp;
                    }

                    // Advance fundamental phase
                    *phase += 2.0 * PI * f0 / sample_rate;
                    if *phase >= 2.0 * PI {
                        *phase -= 2.0 * PI;
                    }

                    samples.push(sample * env * 0.45);
                }
            }
            PhonemeCategory::VoicedConsonant { f1, f2 } => {
                let scaled_f1 = f1 as f32 * formant_scale;
                let scaled_f2 = f2 as f32 * formant_scale;

                for i in 0..num_samples {
                    let env = Self::adsr_envelope(i, num_samples, 0.1, 0.1);
                    let mut sample = 0.0f32;

                    for k in 1..=3 {
                        let harm_f = f0 * k as f32;
                        let r1 = 1.0 / (1.0 + 0.0002 * (harm_f - scaled_f1).powi(2));
                        let r2 = 0.6 / (1.0 + 0.0002 * (harm_f - scaled_f2).powi(2));
                        let resonance = 0.4 + r1 + r2;

                        let harm_phase = *phase * k as f32;
                        let harm_amp = (harmonic_decay * 0.9).powi(k - 1) * resonance;
                        sample += harm_phase.sin() * harm_amp;
                    }

                    *phase += 2.0 * PI * f0 / sample_rate;
                    if *phase >= 2.0 * PI {
                        *phase -= 2.0 * PI;
                    }

                    samples.push(sample * env * 0.35);
                }
            }
            PhonemeCategory::UnvoicedFricative => {
                // High-pass filtered noise for sibilants/fricatives
                let mut prev_noise = 0.0f32;
                for i in 0..num_samples {
                    let env = Self::adsr_envelope(i, num_samples, 0.1, 0.2);
                    let raw_noise = rng.next_f32();
                    // Simple high-pass difference filter
                    let noise = raw_noise - 0.7 * prev_noise;
                    prev_noise = raw_noise;

                    samples.push(noise * env * 0.25);
                }
            }
            PhonemeCategory::Plosive => {
                // Pre-burst closure (silence for first 60%) + sudden transient burst
                let burst_start = (num_samples as f32 * 0.6).round() as usize;
                for i in 0..num_samples {
                    if i < burst_start {
                        samples.push(0.0);
                    } else {
                        let burst_idx = i - burst_start;
                        let burst_len = num_samples - burst_start;
                        let decay = (-3.0 * (burst_idx as f32 / burst_len as f32)).exp();
                        let click = rng.next_f32() * decay * 0.4;
                        samples.push(click);
                    }
                }
            }
        }

        samples
    }

    /// Computes smooth ADSR envelope factor in [0.0, 1.0].
    fn adsr_envelope(idx: usize, total: usize, attack_ratio: f32, release_ratio: f32) -> f32 {
        if total <= 1 {
            return 1.0;
        }

        let pos = idx as f32 / total as f32;
        if pos < attack_ratio {
            (pos / attack_ratio).clamp(0.0, 1.0)
        } else if pos > (1.0 - release_ratio) {
            ((1.0 - pos) / release_ratio).clamp(0.0, 1.0)
        } else {
            1.0
        }
    }
}
