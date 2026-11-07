#![forbid(unsafe_code)]
//! # MiniAI Offline Neural Text-to-Speech (TTS) & Voice Conversation Loop (`miniai-speech-tts`)
//!
//! Production-grade acoustic formant synthesizer, lossless RIFF/WAVE encoder,
//! text normalizer with phonetic expansion, pluggable `SpeechTtsAdapter`,
//! and offline roundtrip voice conversation loop (`ASR -> LLM -> TTS`).

pub mod adapter;
pub mod error;
pub mod normalizer;
pub mod synthesizer;
pub mod types;
pub mod voice_loop;
pub mod wav;

pub use adapter::SpeechTtsAdapter;
pub use error::SpeechTtsError;
pub use normalizer::{PhonemeCategory, PhonemeToken, TextNormalizer, MAX_SEGMENT_CHARS};
pub use synthesizer::{SpeechTtsEngine, XorShift64};
pub use types::*;
pub use voice_loop::VoiceConversationLoop;
pub use wav::WavEncoder;

