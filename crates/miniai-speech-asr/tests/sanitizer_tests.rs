use miniai_speech_asr::error::SpeechAsrError;
use miniai_speech_asr::sanitizer::{AudioSanitizer, MAX_AUDIO_DURATION_SECS, MAX_AUDIO_FILE_SIZE};
use miniai_speech_asr::types::{AudioFormat, AudioMetadata};

pub fn create_test_wav(
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
            match bits_per_sample {
                16 => {
                    let sample_i16 = (val * 32767.0).round() as i16;
                    data_bytes.extend_from_slice(&sample_i16.to_le_bytes());
                }
                8 => {
                    let sample_u8 = ((val * 127.0) + 128.0).round() as u8;
                    data_bytes.push(sample_u8);
                }
                32 => {
                    let sample_f32 = val;
                    data_bytes.extend_from_slice(&sample_f32.to_le_bytes());
                }
                _ => {}
            }
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
    let format_tag = if bits_per_sample == 32 { 3u16 } else { 1u16 };
    wav.extend_from_slice(&format_tag.to_le_bytes());
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
fn test_sanitize_valid_16bit_wav() {
    let wav_bytes = create_test_wav(16000, 1, 16, 16000, 440.0);
    let (meta, samples) = AudioSanitizer::sanitize_and_decode(&wav_bytes)
        .expect("Valid 16-bit WAV should decode successfully");

    expect_metadata_matches(&meta, AudioFormat::Wav, 16000, 1, 16, 1.0, 16000);
    assert_eq!(samples.len(), 16000);
    assert!(!meta.sha256.is_empty());
}

#[test]
fn test_sanitize_valid_stereo_32bit_float_wav() {
    let wav_bytes = create_test_wav(48000, 2, 32, 24000, 880.0);
    let (meta, samples) = AudioSanitizer::sanitize_and_decode(&wav_bytes)
        .expect("Valid 32-bit float WAV should decode");

    expect_metadata_matches(&meta, AudioFormat::Wav, 48000, 2, 32, 0.5, 48000);
    assert_eq!(samples.len(), 48000); // 24000 * 2 channels
}

#[test]
fn test_sanitize_valid_8bit_wav() {
    let wav_bytes = create_test_wav(8000, 1, 8, 8000, 200.0);
    let (meta, samples) =
        AudioSanitizer::sanitize_and_decode(&wav_bytes).expect("Valid 8-bit WAV should decode");

    expect_metadata_matches(&meta, AudioFormat::Wav, 8000, 1, 8, 1.0, 8000);
    assert_eq!(samples.len(), 8000);
}

#[test]
fn test_sanitize_valid_flac_header() {
    let mut flac = Vec::new();
    flac.extend_from_slice(b"fLaC");
    flac.push(0x00); // STREAMINFO block type
    flac.extend_from_slice(&[0x00, 0x00, 0x22]); // 34 bytes length
    let mut streaminfo = vec![0u8; 34];
    // 16000 Hz, 1 channel, 16 bps
    // 16000 = 0x03E80 -> (0x03 << 12) | (0xE8 << 4) | (0x00)
    streaminfo[10] = 0x03;
    streaminfo[11] = 0xE8;
    streaminfo[12] = 0x00; // channels = ((0 >> 1) + 1) = 1
    streaminfo[13] = 0xF0; // bps = 16 (0xF0 >> 4 = 15 + 1)
                           // Total samples: 16000
    streaminfo[17] = 0xE8;
    streaminfo[16] = 0x3E;
    flac.extend_from_slice(&streaminfo);
    flac.extend_from_slice(&vec![0u8; 3200]); // 1600 samples of 16-bit PCM

    let (meta, samples) =
        AudioSanitizer::sanitize_and_decode(&flac).expect("Valid FLAC header should decode");
    assert_eq!(meta.format, AudioFormat::Flac);
    assert_eq!(meta.sample_rate, 16000);
    assert_eq!(meta.channels, 1);
    assert!(!samples.is_empty());
}

#[test]
fn test_sanitize_valid_ogg_header() {
    let mut ogg = Vec::new();
    ogg.extend_from_slice(b"OggS");
    ogg.extend_from_slice(&[0u8; 24]); // Minimal page header
    ogg.extend_from_slice(&vec![0u8; 3200]); // Payload

    let (meta, samples) =
        AudioSanitizer::sanitize_and_decode(&ogg).expect("Valid OGG header should decode");
    assert_eq!(meta.format, AudioFormat::Ogg);
    assert_eq!(meta.sample_rate, 16000);
    assert!(!samples.is_empty());
}

#[test]
fn test_reject_truncated_headers() {
    let short_payload = b"RIFF short";
    let err = AudioSanitizer::sanitize_and_decode(short_payload).unwrap_err();
    assert!(matches!(err, SpeechAsrError::CorruptPayload(_)));
}

#[test]
fn test_reject_invalid_magic_bytes() {
    let invalid = b"NOT_AUDIO_DATA_FOR_PARSER_1234";
    let err = AudioSanitizer::sanitize_and_decode(invalid).unwrap_err();
    assert!(matches!(err, SpeechAsrError::InvalidFormat(_)));
}

#[test]
fn test_reject_file_exceeding_max_size() {
    let oversized = vec![0u8; MAX_AUDIO_FILE_SIZE + 1];
    let err = AudioSanitizer::sanitize_and_decode(&oversized).unwrap_err();
    assert!(matches!(err, SpeechAsrError::FileTooLarge { .. }));
}

#[test]
fn test_reject_oversized_duration() {
    let wav_bytes = create_test_wav(
        16000,
        1,
        16,
        ((MAX_AUDIO_DURATION_SECS + 10.0) * 16000.0) as usize,
        0.0,
    );
    let err = AudioSanitizer::sanitize_and_decode(&wav_bytes).unwrap_err();
    assert!(matches!(err, SpeechAsrError::AudioTooLong { .. }));
}

fn expect_metadata_matches(
    meta: &AudioMetadata,
    fmt: AudioFormat,
    rate: u32,
    channels: u16,
    bps: u16,
    duration: f32,
    count: usize,
) {
    assert_eq!(meta.format, fmt);
    assert_eq!(meta.sample_rate, rate);
    assert_eq!(meta.channels, channels);
    assert_eq!(meta.bits_per_sample, bps);
    assert!((meta.duration_secs - duration).abs() < 0.05);
    assert_eq!(meta.sample_count, count);
}
