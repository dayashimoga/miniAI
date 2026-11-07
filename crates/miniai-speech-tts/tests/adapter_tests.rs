use std::io::Write;
use tempfile::NamedTempFile;

use miniai_core::{AcceleratorBackend, Modality, ModalityAdapter};
use miniai_speech_tts::{SpeechTtsAdapter, SpeechTtsError, TtsSynthesisConfig};
use sha2::{Digest, Sha256};

#[test]
fn test_tts_adapter_modality_contract() {
    let adapter = SpeechTtsAdapter::new("test-speech-tts-adapter");

    assert_eq!(adapter.adapter_id(), "test-speech-tts-adapter");
    assert_eq!(adapter.modality(), Modality::SpeechTts);
    assert_eq!(adapter.supported_backends(), vec![AcceleratorBackend::Cpu]);

    let cap = adapter.capability();
    assert_eq!(cap.adapter_id, "test-speech-tts-adapter");
    assert_eq!(cap.modality, Modality::SpeechTts);
    assert!(cap.supports_streaming);
    assert!(cap.supports_offline);

    let health = adapter.health_check().expect("health check");
    assert!(health.ready);
    assert_eq!(health.active_backend, AcceleratorBackend::Cpu);
}

#[test]
fn test_tts_adapter_weights_loading_and_sha256() {
    let adapter = SpeechTtsAdapter::new("test-weights-adapter");

    let mut temp = NamedTempFile::new().expect("temp file");
    let dummy_weights = b"MINIAI_DUMMY_TTS_WEIGHTS_DATA_V1";
    temp.write_all(dummy_weights).expect("write weights");
    temp.flush().expect("flush");

    let mut hasher = Sha256::new();
    hasher.update(dummy_weights);
    let expected_sha256 = format!("{:x}", hasher.finalize());

    // Valid load
    let res = adapter.load_model_weights(temp.path(), Some(&expected_sha256));
    assert!(res.is_ok());
    assert!(adapter.is_loaded());

    // Corrupt sha256 mismatch
    let corrupt_res =
        adapter.load_model_weights(temp.path(), Some("00000000000000000000000000000000"));
    assert!(matches!(
        corrupt_res,
        Err(SpeechTtsError::CorruptPayload(_))
    ));

    // Unload
    adapter.unload_model();
    assert!(!adapter.is_loaded());
}

#[test]
fn test_tts_adapter_end_to_end_synthesis() {
    let adapter = SpeechTtsAdapter::new("e2e-tts-adapter");
    let config = TtsSynthesisConfig::default();

    let output = adapter
        .synthesize_text("MiniAI provides local neural speech synthesis.", &config)
        .expect("synthesize text");

    assert!(output.duration_secs > 0.0);
    assert_eq!(output.sample_rate, 22050);
    assert_eq!(output.channels, 1);
    assert_eq!(output.wav_bytes.len(), 44 + output.pcm_samples.len() * 2);
    assert!(!output.sha256.is_empty());
}
