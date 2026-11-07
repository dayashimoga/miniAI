use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;

use miniai_llm::{GenerationConfig, LlmEngine};
use miniai_speech_asr::SpeechAsrAdapter;

use crate::adapter::SpeechTtsAdapter;
use crate::error::SpeechTtsError;
use crate::types::{TtsSynthesisConfig, VoiceTurnMetrics, VoiceTurnResult};

/// High-performance offline voice conversation loop orchestrating ASR, LLM, and TTS.
pub struct VoiceConversationLoop {
    asr_adapter: Arc<SpeechAsrAdapter>,
    llm_engine: Arc<RwLock<LlmEngine>>,
    tts_adapter: Arc<SpeechTtsAdapter>,
    system_prompt: RwLock<String>,
    llm_config: RwLock<GenerationConfig>,
}

impl VoiceConversationLoop {
    /// Creates a new `VoiceConversationLoop` linking speech recognition, LLM reasoning, and acoustic synthesis.
    pub fn new(
        asr_adapter: Arc<SpeechAsrAdapter>,
        llm_engine: Arc<RwLock<LlmEngine>>,
        tts_adapter: Arc<SpeechTtsAdapter>,
    ) -> Self {
        Self {
            asr_adapter,
            llm_engine,
            tts_adapter,
            system_prompt: RwLock::new(
                "You are MiniAI, a helpful, concise offline voice assistant.".to_string(),
            ),
            llm_config: RwLock::new(GenerationConfig {
                max_new_tokens: 128,
                temperature: 0.7,
                top_p: 0.9,
                repetition_penalty: 1.1,
                seed: Some(42),
                stop_sequences: vec![
                    "\n\nUser:".to_string(),
                    "</s>".to_string(),
                    "\nUser:".to_string(),
                ],
                ..Default::default()
            }),
        }
    }

    /// Updates system prompt for voice conversations.
    pub fn set_system_prompt(&self, prompt: impl Into<String>) {
        if let Ok(mut g) = self.system_prompt.write() {
            *g = prompt.into();
        }
    }

    /// Gets active system prompt.
    pub fn system_prompt(&self) -> String {
        self.system_prompt
            .read()
            .map(|g| g.clone())
            .unwrap_or_default()
    }

    /// Updates LLM generation configuration.
    pub fn set_llm_config(&self, config: GenerationConfig) {
        if let Ok(mut g) = self.llm_config.write() {
            *g = config;
        }
    }

    /// Executes an offline voice conversation turn from user speech audio bytes.
    pub fn execute_audio_turn(
        &self,
        audio_bytes: &[u8],
        tts_config: &TtsSynthesisConfig,
        cancel_token: Option<&Arc<AtomicBool>>,
    ) -> Result<VoiceTurnResult, SpeechTtsError> {
        let overall_start = Instant::now();

        if let Some(token) = cancel_token {
            if token.load(Ordering::SeqCst) {
                return Err(SpeechTtsError::Cancelled);
            }
        }

        // Phase 1: Speech-to-Text (ASR)
        let asr_start = Instant::now();
        let asr_result = self
            .asr_adapter
            .transcribe_bytes(audio_bytes)
            .map_err(|e| SpeechTtsError::AsrError(e.to_string()))?;
        let asr_latency_ms = asr_start.elapsed().as_secs_f64() * 1000.0;

        let user_text = asr_result.full_text.trim();
        if user_text.is_empty() {
            return Err(SpeechTtsError::EmptyText);
        }

        self.execute_turn_from_user_text(
            user_text,
            asr_latency_ms,
            overall_start,
            tts_config,
            cancel_token,
        )
    }

    /// Executes an offline voice conversation turn directly from text (bypassing ASR).
    pub fn execute_text_turn(
        &self,
        user_text: &str,
        tts_config: &TtsSynthesisConfig,
        cancel_token: Option<&Arc<AtomicBool>>,
    ) -> Result<VoiceTurnResult, SpeechTtsError> {
        let overall_start = Instant::now();
        let trimmed = user_text.trim();
        if trimmed.is_empty() {
            return Err(SpeechTtsError::EmptyText);
        }

        self.execute_turn_from_user_text(trimmed, 0.0, overall_start, tts_config, cancel_token)
    }

    fn execute_turn_from_user_text(
        &self,
        user_text: &str,
        asr_latency_ms: f64,
        overall_start: Instant,
        tts_config: &TtsSynthesisConfig,
        cancel_token: Option<&Arc<AtomicBool>>,
    ) -> Result<VoiceTurnResult, SpeechTtsError> {
        if let Some(token) = cancel_token {
            if token.load(Ordering::SeqCst) {
                return Err(SpeechTtsError::Cancelled);
            }
        }

        // Phase 2: LLM Text Generation
        let sys_prompt = self.system_prompt();
        let prompt = format!("{sys_prompt}\n\nUser: {user_text}\n\nAssistant:");

        let llm_cfg = self
            .llm_config
            .read()
            .map(|g| g.clone())
            .unwrap_or_default();

        let llm_start = Instant::now();
        let llm_output = {
            let guard = self.llm_engine.read().map_err(|_| {
                SpeechTtsError::Processing("Lock poisoned on LLM engine".to_string())
            })?;
            guard
                .generate(&prompt, &llm_cfg)
                .map_err(|e| SpeechTtsError::LlmError(e.to_string()))?
        };
        let llm_total_latency_ms = llm_start.elapsed().as_secs_f64() * 1000.0;
        let llm_ttft_ms = if llm_output.ttft_ms > 0.0 {
            llm_output.ttft_ms
        } else {
            llm_total_latency_ms * 0.25
        };

        if let Some(token) = cancel_token {
            if token.load(Ordering::SeqCst) {
                return Err(SpeechTtsError::Cancelled);
            }
        }

        // Phase 3: Text-to-Speech (TTS) Acoustic Synthesis
        let tts_start = Instant::now();
        let synthesized_audio = self.tts_adapter.synthesize_with_cancellation(
            &llm_output.text,
            tts_config,
            cancel_token,
            None,
        )?;
        let tts_latency_ms = tts_start.elapsed().as_secs_f64() * 1000.0;

        let total_turnaround_ms = overall_start.elapsed().as_secs_f64() * 1000.0;

        Ok(VoiceTurnResult {
            user_transcription: user_text.to_string(),
            agent_response_text: llm_output.text,
            synthesized_audio,
            metrics: VoiceTurnMetrics {
                asr_latency_ms,
                llm_ttft_ms,
                llm_total_latency_ms,
                tts_latency_ms,
                total_turnaround_ms,
            },
        })
    }
}
