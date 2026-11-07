use miniai_speech_asr::AudioSanitizer;
use miniai_speech_tts::{SpeechTtsError, WavEncoder};

#[test]
fn test_wav_encoder_valid_header_and_parsing() {
    let samples = vec![0.0f32, 0.5, -0.5, 0.25, -0.25, 0.0];
    let sample_rate = 22050;
    let wav_bytes = WavEncoder::encode_pcm16_mono(&samples, sample_rate).expect("encode wav");

    assert!(wav_bytes.len() >= 44);
    assert_eq!(&wav_bytes[0..4], b"RIFF");
    assert_eq!(&wav_bytes[8..12], b"WAVE");
    assert_eq!(&wav_bytes[12..16], b"fmt ");
    assert_eq!(&wav_bytes[36..40], b"data");

    let (sr, ch, bits, data_len) =
        WavEncoder::parse_header(&wav_bytes).expect("parse encoded wav header");
    assert_eq!(sr, 22050);
    assert_eq!(ch, 1);
    assert_eq!(bits, 16);
    assert_eq!(data_len, samples.len() * 2);
}

#[test]
fn test_wav_encoder_roundtrip_with_asr_sanitizer() {
    // Generate 1 second of synthetic 440Hz sine wave at 22050 Hz
    let sample_rate = 22050;
    let mut samples = Vec::with_capacity(sample_rate as usize);
    for i in 0..sample_rate {
        let t = i as f32 / sample_rate as f32;
        samples.push((2.0 * std::f32::consts::PI * 440.0 * t).sin() * 0.8);
    }

    let wav_bytes = WavEncoder::encode_pcm16_mono(&samples, sample_rate).expect("encode wav");

    // Pass through miniai-speech-asr AudioSanitizer to guarantee cross-crate interoperability
    let (metadata, decoded_samples) =
        AudioSanitizer::sanitize_and_decode(&wav_bytes).expect("sanitizer decodes generated wav");

    assert_eq!(metadata.sample_rate, 22050);
    assert_eq!(metadata.channels, 1);
    assert_eq!(metadata.bits_per_sample, 16);
    assert!((metadata.duration_secs - 1.0).abs() < 0.01);
    assert_eq!(decoded_samples.len(), samples.len());
}

#[test]
fn test_wav_encoder_clamping_and_boundaries() {
    // Samples exceeding [-1.0, 1.0] must be clamped without overflow panics
    let extreme_samples = vec![-5.0f32, -1.0, 0.0, 1.0, 10.0];
    let wav_bytes = WavEncoder::encode_pcm16_mono(&extreme_samples, 24000).expect("encode clamped");

    let (_, _, _, data_len) = WavEncoder::parse_header(&wav_bytes).expect("parse header");
    assert_eq!(data_len, extreme_samples.len() * 2);
}

#[test]
fn test_wav_encoder_rejects_zero_sample_rate() {
    let samples = vec![0.1f32, 0.2];
    let res = WavEncoder::encode_pcm16_mono(&samples, 0);
    assert!(matches!(res, Err(SpeechTtsError::InvalidParameter(_))));
}

#[test]
fn test_wav_header_parser_rejects_truncated() {
    let truncated = vec![0u8; 20];
    let res = WavEncoder::parse_header(&truncated);
    assert!(matches!(res, Err(SpeechTtsError::EncodingError(_))));
}

