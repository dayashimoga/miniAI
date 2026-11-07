use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use miniai_speech_tts::{SpeechTtsEngine, SpeechTtsError, TtsSynthesisConfig, VoiceProfile};

#[test]
fn test_unloaded_engine_returns_error() {
    let engine = SpeechTtsEngine::new("test_model");
    let config = TtsSynthesisConfig::default();
    let res = engine.synthesize("hello", &config, None, None);
    assert!(matches!(res, Err(SpeechTtsError::UnloadedModel)));
}

#[test]
fn test_invalid_parameters_rejected() {
    let mut engine = SpeechTtsEngine::new("test_model");
    engine.set_weights_loaded(true);

    // Bad sample rate
    let bad_sr = TtsSynthesisConfig {
        sample_rate: 8000,
        ..Default::default()
    };
    assert!(matches!(
        engine.synthesize("hello", &bad_sr, None, None),
        Err(SpeechTtsError::InvalidParameter(_))
    ));

    // Bad speed
    let bad_speed = TtsSynthesisConfig {
        speed: 3.5,
        ..Default::default()
    };
    assert!(matches!(
        engine.synthesize("hello", &bad_speed, None, None),
        Err(SpeechTtsError::InvalidParameter(_))
    ));

    // Bad pitch
    let bad_pitch = TtsSynthesisConfig {
        pitch_semitones: 24.0,
        ..Default::default()
    };
    assert!(matches!(
        engine.synthesize("hello", &bad_pitch, None, None),
        Err(SpeechTtsError::InvalidParameter(_))
    ));
}

#[test]
fn test_deterministic_reproducibility() {
    let mut engine = SpeechTtsEngine::new("test_model");
    engine.set_weights_loaded(true);

    let config = TtsSynthesisConfig {
        voice: VoiceProfile::Alloy,
        sample_rate: 22050,
        speed: 1.0,
        pitch_semitones: 0.0,
        volume_gain: 1.0,
        seed: Some(12345),
    };

    let text = "Reproducible acoustic formant speech synthesis.";
    let res1 = engine
        .synthesize(text, &config, None, None)
        .expect("synth 1");
    let res2 = engine
        .synthesize(text, &config, None, None)
        .expect("synth 2");

    assert_eq!(res1.sha256, res2.sha256);
    assert_eq!(res1.pcm_samples, res2.pcm_samples);
    assert_eq!(res1.wav_bytes, res2.wav_bytes);
    assert_eq!(res1.phoneme_count, res2.phoneme_count);
}

#[test]
fn test_all_voice_presets_synthesize_valid_audio() {
    let mut engine = SpeechTtsEngine::new("test_model");
    engine.set_weights_loaded(true);

    for voice in VoiceProfile::presets() {
        let config = TtsSynthesisConfig {
            voice,
            sample_rate: 22050,
            seed: Some(999),
            ..Default::default()
        };

        let res = engine
            .synthesize("Testing voice profile.", &config, None, None)
            .expect("synthesize voice preset");

        assert!(res.duration_secs > 0.0);
        assert!(!res.wav_bytes.is_empty());
        assert!(!res.sha256.is_empty());
    }
}

#[test]
fn test_distinct_voices_produce_different_waveforms() {
    let mut engine = SpeechTtsEngine::new("test_model");
    engine.set_weights_loaded(true);

    let config_alloy = TtsSynthesisConfig {
        voice: VoiceProfile::Alloy,
        sample_rate: 22050,
        seed: Some(42),
        ..Default::default()
    };

    let config_onyx = TtsSynthesisConfig {
        voice: VoiceProfile::Onyx,
        sample_rate: 22050,
        seed: Some(42),
        ..Default::default()
    };

    let res_alloy = engine
        .synthesize("Compare voices.", &config_alloy, None, None)
        .expect("synth alloy");
    let res_onyx = engine
        .synthesize("Compare voices.", &config_onyx, None, None)
        .expect("synth onyx");

    assert_ne!(res_alloy.sha256, res_onyx.sha256);
}

#[test]
fn test_cooperative_cancellation() {
    let mut engine = SpeechTtsEngine::new("test_model");
    engine.set_weights_loaded(true);

    let cancel_token = Arc::new(AtomicBool::new(true)); // Already cancelled
    let config = TtsSynthesisConfig::default();

    let res = engine.synthesize(
        "This generation must cancel immediately.",
        &config,
        Some(&cancel_token),
        None,
    );

    assert!(matches!(res, Err(SpeechTtsError::Cancelled)));
}

#[test]
fn test_progress_telemetry_callback_fires() {
    let mut engine = SpeechTtsEngine::new("test_model");
    engine.set_weights_loaded(true);

    let progress_count = Arc::new(AtomicUsize::new(0));
    let progress_clone = Arc::clone(&progress_count);
    let config = TtsSynthesisConfig::default();

    let res = engine.synthesize(
        "Sentence one. Sentence two.",
        &config,
        None,
        Some(&|_| {
            progress_clone.fetch_add(1, Ordering::SeqCst);
        }),
    );

    assert!(res.is_ok());
    assert!(progress_count.load(Ordering::SeqCst) >= 1);
}
