mod test_helpers;

use test_helpers::create_test_mp4;

use miniai_video::demuxer::AudioDemuxer;
use miniai_video::sanitizer::VideoSanitizer;
use miniai_video::types::SanitizerConfig;

#[test]
fn test_demux_audio_from_video_with_audio() {
    let bytes = create_test_mp4(1280, 720, 3.0, true);
    let meta = VideoSanitizer::validate_bytes(&bytes, &SanitizerConfig::default()).expect("meta");
    assert!(meta.has_audio);

    let demuxed = AudioDemuxer::demux_audio(&bytes, &meta)
        .expect("demux audio")
        .expect("Should produce audio track");

    assert_eq!(demuxed.channels, 2);
    assert_eq!(demuxed.sample_rate, 44100);
    assert!((demuxed.duration_secs - 3.0).abs() < 0.1);
    assert!(!demuxed.pcm_samples.is_empty());
    assert!(!demuxed.wav_bytes.is_empty());

    // Verify WAV header
    assert_eq!(&demuxed.wav_bytes[0..4], b"RIFF");
    assert_eq!(&demuxed.wav_bytes[8..12], b"WAVE");
    assert_eq!(&demuxed.wav_bytes[12..16], b"fmt ");

    // Verify interoperability with Speech ASR AudioSanitizer
    let (asr_meta, decoded_samples) =
        miniai_speech_asr::AudioSanitizer::sanitize_and_decode(&demuxed.wav_bytes)
            .expect("Demuxed WAV must pass Speech ASR Sanitizer");

    assert_eq!(asr_meta.sample_rate, 44100);
    assert_eq!(asr_meta.channels, 2);
    assert!(!decoded_samples.is_empty());
}

#[test]
fn test_demux_audio_from_silent_video() {
    let bytes = create_test_mp4(640, 360, 2.0, false);
    let meta = VideoSanitizer::validate_bytes(&bytes, &SanitizerConfig::default()).expect("meta");
    assert!(!meta.has_audio);

    let demuxed = AudioDemuxer::demux_audio(&bytes, &meta).expect("demux audio");
    assert!(demuxed.is_none());
}

#[test]
fn test_encode_wav_16bit() {
    let samples = vec![0.0f32, 0.5, -0.5, 1.0, -1.0];
    let wav = AudioDemuxer::encode_wav_16bit(&samples, 16000, 1);

    assert_eq!(&wav[0..4], b"RIFF");
    assert_eq!(&wav[8..12], b"WAVE");
    assert_eq!(wav.len(), 44 + (samples.len() * 2));
}
