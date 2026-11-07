#![forbid(unsafe_code)]
//! # MiniAI Offline Polyphonic Music & Audio Synthesis (`miniai-music`)
//!
//! Production-grade offline music generator, multi-track acoustic engine
//! (melody, harmony pad, bassline, and rhythm), lossless RIFF/WAVE stereo
//! encoder, and pluggable `MusicAdapter` conforming to `ModalityAdapter`.

pub mod adapter;
pub mod engine;
pub mod error;
pub mod types;
pub mod wav;

pub use adapter::MusicAdapter;
pub use engine::{MusicGenEngine, XorShift64};
pub use error::MusicError;
pub use types::*;
pub use wav::WavEncoder;

