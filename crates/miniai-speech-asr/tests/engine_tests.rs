use miniai_speech_asr::engine::SpeechAsrEngine;
use miniai_speech_asr::error::SpeechAsrError;
use miniai_speech_asr::types::AudioBuffer;

#[test]
fn test_unloaded_model_returns_error() {
    let engine = SpeechAsrEngine::new("whisper-tiny-en");
    // Weights are not loaded
    let buffer = AudioBuffer::new(vec![0.1; 16000], 16000, 1);
    let err = engine.transcribe(&buffer, "sha256-test").unwrap_err();
    assert!(matches!(err, SpeechAsrError::UnloadedModel));
}

#[test]
fn test_transcribe_silence_produces_empty_transcript() {
    let mut engine = SpeechAsrEngine::new("whisper-tiny-en");
    engine.set_weights_loaded(true);

    let silence = AudioBuffer::new(vec![0.0f32; 16000 * 2], 16000, 1);
    let result = engine
        .transcribe(&silence, "sha256-silence")
        .expect("Transcribing silence should succeed");

    assert!(result.full_text.is_empty());
    assert!(result.segments.is_empty());
    assert_eq!(result.duration_secs, 2.0);
    assert_eq!(result.language, "en");
    assert_eq!(result.confidence, 1.0);
}

#[test]
fn test_transcribe_speech_signal_produces_timestamped_segments() {
    let mut engine = SpeechAsrEngine::new("whisper-tiny-en");
    engine.set_weights_loaded(true);

    let mut samples = vec![0.0f32; 8000]; // 0.5s pause
    for i in 0..24000 {
        // 1.5s speech burst
        let t = i as f32 / 16000.0;
        samples.push((t * 300.0 * 2.0 * std::f32::consts::PI).sin() * 0.4);
    }
    samples.extend_from_slice(&vec![0.0f32; 8000]); // 0.5s pause

    let buffer = AudioBuffer::new(samples, 16000, 1);
    let result = engine
        .transcribe(&buffer, "sha256-speech")
        .expect("Transcribing speech should succeed");

    assert!(!result.full_text.is_empty());
    assert_eq!(result.segments.len(), 1);
    let seg = &result.segments[0];
    assert!(seg.start_secs >= 0.4 && seg.start_secs <= 0.6);
    assert!(seg.end_secs >= 1.9);
    assert!(!seg.tokens.is_empty());
    assert!(result.confidence > 0.85);
}

#[test]
fn test_deterministic_reproducibility() {
    let mut engine = SpeechAsrEngine::new("whisper-tiny-en");
    engine.set_weights_loaded(true);

    let mut samples = Vec::new();
    for i in 0..16000 {
        let t = i as f32 / 16000.0;
        samples.push((t * 500.0 * 2.0 * std::f32::consts::PI).sin() * 0.35);
    }
    let buffer = AudioBuffer::new(samples, 16000, 1);

    let res1 = engine.transcribe(&buffer, "sha-repeat").unwrap();
    let res2 = engine.transcribe(&buffer, "sha-repeat").unwrap();

    assert_eq!(res1.full_text, res2.full_text);
    assert_eq!(res1.segments, res2.segments);
    assert_eq!(res1.confidence, res2.confidence);
}
