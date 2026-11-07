use std::io::Write;
use tempfile::NamedTempFile;

use miniai_core::{AcceleratorBackend, Modality, ModalityAdapter};
use miniai_music::{MusicAdapter, MusicError, MusicGenConfig};
use miniai_speech_asr::AudioSanitizer;
use sha2::{Digest, Sha256};

#[test]
fn test_music_adapter_modality_contract() {
    let adapter = MusicAdapter::new("test-music-adapter");

    assert_eq!(adapter.adapter_id(), "test-music-adapter");
    assert_eq!(adapter.modality(), Modality::Music);
    assert_eq!(adapter.supported_backends(), vec![AcceleratorBackend::Cpu]);

    let cap = adapter.capability();
    assert_eq!(cap.adapter_id, "test-music-adapter");
    assert_eq!(cap.modality, Modality::Music);
    assert!(cap.supports_offline);
    assert!(!cap.supports_streaming);

    let health = adapter.health_check().expect("health check");
    assert!(health.ready);
    assert_eq!(health.active_backend, AcceleratorBackend::Cpu);
}

#[test]
fn test_music_adapter_weights_loading_and_sha256() {
    let adapter = MusicAdapter::new("test-weights-adapter");

    let mut temp = NamedTempFile::new().expect("temp file");
    let dummy_weights = b"MINIAI_MUSIC_WEIGHTS_MOCK_FREE_V1";
    temp.write_all(dummy_weights).expect("write");
    temp.flush().expect("flush");

    let mut hasher = Sha256::new();
    hasher.update(dummy_weights);
    let expected_sha256 = format!("{:x}", hasher.finalize());

    // Valid load
    let res = adapter.load_model_weights(temp.path(), Some(&expected_sha256));
    assert!(res.is_ok());
    assert!(adapter.is_loaded());

    // Corrupt load
    let corrupt_res =
        adapter.load_model_weights(temp.path(), Some("11111111111111111111111111111111"));
    assert!(matches!(corrupt_res, Err(MusicError::CorruptPayload(_))));

    // Unload
    adapter.unload_model();
    assert!(!adapter.is_loaded());
}

#[test]
fn test_music_adapter_end_to_end_synthesis_and_asr_sanitizer_validation() {
    let adapter = MusicAdapter::new("e2e-music-adapter");
    let config = MusicGenConfig {
        prompt: "Chillhop beats in G Major".to_string(),
        duration_secs: 2.0,
        tempo_bpm: 110,
        sample_rate: 22050,
        channels: 2,
        scale_key: "G Major".to_string(),
        seed: Some(54321),
    };

    let output = adapter.synthesize(&config).expect("synthesize music");

    assert_eq!(output.duration_secs, 2.0);
    assert_eq!(output.sample_rate, 22050);
    assert_eq!(output.channels, 2);
    assert_eq!(output.scale_key, "G Major");
    assert!(!output.sha256.is_empty());

    // Verify output decodes cleanly via AudioSanitizer
    let (metadata, decoded_samples) = AudioSanitizer::sanitize_and_decode(&output.wav_bytes)
        .expect("sanitizer decodes music wav");
    assert_eq!(metadata.sample_rate, 22050);
    assert_eq!(metadata.channels, 2);
    assert_eq!(decoded_samples.len(), output.num_samples_per_channel * 2);
}
