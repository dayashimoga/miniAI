use miniai_core::{AcceleratorBackend, Modality, ModalityAdapter};
use miniai_speech_asr::adapter::SpeechAsrAdapter;
use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;

fn create_test_wav(
    sample_rate: u32,
    channels: u16,
    bits_per_sample: u16,
    samples_per_channel: usize,
    frequency_hz: f32,
) -> Vec<u8> {
    let mut data_bytes = Vec::new();
    let bytes_per_sample = (bits_per_sample / 8) as usize;
    for i in 0..samples_per_channel {
        let t = i as f32 / sample_rate as f32;
        let val = if frequency_hz > 0.0 {
            (t * frequency_hz * 2.0 * std::f32::consts::PI).sin() * 0.5
        } else {
            0.0
        };
        for _ in 0..channels {
            let sample_i16 = (val * 32767.0).round() as i16;
            data_bytes.extend_from_slice(&sample_i16.to_le_bytes());
        }
    }

    let mut wav = Vec::new();
    wav.extend_from_slice(b"RIFF");
    let file_size = 36 + data_bytes.len() as u32;
    wav.extend_from_slice(&file_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    // fmt chunk
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM format
    wav.extend_from_slice(&channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    let byte_rate = sample_rate * channels as u32 * bytes_per_sample as u32;
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    let block_align = channels * bytes_per_sample as u16;
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());

    // data chunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&(data_bytes.len() as u32).to_le_bytes());
    wav.extend_from_slice(&data_bytes);

    wav
}

#[test]
fn test_asr_adapter_modality_contract() {
    let adapter = SpeechAsrAdapter::new("whisper-cpu-adapter");

    assert_eq!(adapter.adapter_id(), "whisper-cpu-adapter");
    assert_eq!(adapter.modality(), Modality::SpeechAsr);
    assert_eq!(adapter.supported_backends(), vec![AcceleratorBackend::Cpu]);

    let cap = adapter.capability();
    assert_eq!(cap.adapter_id, "whisper-cpu-adapter");
    assert_eq!(cap.modality, Modality::SpeechAsr);
    assert!(cap.supports_offline);
    assert!(cap.supports_streaming);
    assert!(cap.isolated_process);

    let health = adapter.health_check().expect("Health check should succeed");
    assert!(health.ready);
    assert_eq!(health.active_backend, AcceleratorBackend::Cpu);
}

#[test]
fn test_asr_adapter_weights_loading_and_sha256() {
    let adapter = SpeechAsrAdapter::new("whisper-custom-adapter");

    // Write a dummy weights file
    let file = NamedTempFile::new().unwrap();
    let content = b"MINIAI_SPEECH_ASR_WEIGHTS_V1_DETERMINISTIC_CHECKPOINT_DATA";
    std::fs::write(file.path(), content).unwrap();

    let mut hasher = Sha256::new();
    hasher.update(content);
    let valid_sha = format!("{:x}", hasher.finalize());

    // Fails on corrupted hash
    let err = adapter
        .load_model_weights(file.path(), Some("00000000000000000000000000000000"))
        .unwrap_err();
    assert!(err.to_string().contains("SHA-256 mismatch"));

    // Succeeds on valid hash
    adapter
        .load_model_weights(file.path(), Some(&valid_sha))
        .expect("Valid SHA-256 load should succeed");
    assert!(adapter.is_loaded());

    // Unload
    adapter.unload_model();
    assert!(!adapter.is_loaded());
}

#[test]
fn test_asr_adapter_end_to_end_transcription() {
    let adapter = SpeechAsrAdapter::new("whisper-e2e-adapter");

    // Generate WAV: 16 kHz, 1 ch, 16 bps, 1 second of speech tone
    let wav_bytes = create_test_wav(16000, 1, 16, 16000, 440.0);

    let result = adapter
        .transcribe_bytes(&wav_bytes)
        .expect("End-to-end transcription should succeed");

    assert!(!result.full_text.is_empty());
    assert_eq!(result.language, "en");
    assert_eq!(result.duration_secs, 1.0);
    assert!(!result.segments.is_empty());
    assert!(!result.audio_sha256.is_empty());

    // Also test transcribe_file
    let file = NamedTempFile::new().unwrap();
    std::fs::write(file.path(), &wav_bytes).unwrap();

    let file_result = adapter
        .transcribe_file(file.path())
        .expect("transcribe_file should succeed");
    assert_eq!(file_result.full_text, result.full_text);
}
