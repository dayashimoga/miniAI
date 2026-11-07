use miniai_speech_asr::resampler::{AudioResampler, STANDARD_SAMPLE_RATE};

#[test]
fn test_downmix_stereo_to_mono() {
    // Stereo: Left = 0.6, Right = 0.2 -> average = 0.4
    let stereo = vec![0.6, 0.2, 0.8, 0.4, -0.5, 0.5];
    let mono = AudioResampler::downmix_to_mono(&stereo, 2);

    assert_eq!(mono.len(), 3);
    assert!((mono[0] - 0.4).abs() < 1e-5);
    assert!((mono[1] - 0.6).abs() < 1e-5);
    assert!((mono[2] - 0.0).abs() < 1e-5);
}

#[test]
fn test_downmix_quad_channels_to_mono() {
    let quad = vec![0.1, 0.2, 0.3, 0.4];
    let mono = AudioResampler::downmix_to_mono(&quad, 4);

    assert_eq!(mono.len(), 1);
    assert!((mono[0] - 0.25).abs() < 1e-5);
}

#[test]
fn test_resample_48k_to_16k_exact_ratio() {
    // 48000 Hz to 16000 Hz is exactly 3:1 decimation
    let mut input_48k = Vec::new();
    for i in 0..4800 {
        let t = i as f32 / 48000.0;
        input_48k.push((t * 440.0 * 2.0 * std::f32::consts::PI).sin());
    }

    let resampled = AudioResampler::resample_to_16k(&input_48k, 48000);
    assert_eq!(resampled.len(), 1600);
    for s in &resampled {
        assert!(*s >= -1.0 && *s <= 1.0);
    }
}

#[test]
fn test_resample_44_1k_to_16k_fractional_ratio() {
    let input_44k = vec![0.5; 44100]; // 1 second of audio
    let resampled = AudioResampler::resample_to_16k(&input_44k, 44100);

    // Expect ~16000 samples for 1 second
    assert_eq!(resampled.len(), 16000);
    for s in &resampled {
        assert!((s - 0.5).abs() < 1e-4);
    }
}

#[test]
fn test_resample_pass_through_16k() {
    let input_16k = vec![0.1, 0.2, 0.3, 0.4, 0.5];
    let resampled = AudioResampler::resample_to_16k(&input_16k, STANDARD_SAMPLE_RATE);
    assert_eq!(resampled, input_16k);
}

#[test]
fn test_rms_normalization() {
    // Low-volume signal: max amp 0.02
    let mut low_vol = vec![0.02; 1000];
    AudioResampler::normalize_rms(&mut low_vol, 0.15);

    let sum_sq: f32 = low_vol.iter().map(|&s| s * s).sum();
    let rms = (sum_sq / low_vol.len() as f32).sqrt();
    assert!((rms - 0.15).abs() < 0.01);
}
