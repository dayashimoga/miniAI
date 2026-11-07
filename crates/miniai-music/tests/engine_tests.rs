use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use miniai_music::{MusicError, MusicGenConfig, MusicGenEngine, WavEncoder};

#[test]
fn test_unloaded_music_engine_returns_error() {
    let engine = MusicGenEngine::new("test_music");
    let config = MusicGenConfig::default();
    let res = engine.synthesize(&config, None, None);
    assert!(matches!(res, Err(MusicError::UnloadedModel)));
}

#[test]
fn test_deterministic_reproducibility() {
    let mut engine = MusicGenEngine::new("test_music");
    engine.set_weights_loaded(true);

    let config = MusicGenConfig {
        prompt: "Lofi hip hop beats".to_string(),
        duration_secs: 2.0,
        tempo_bpm: 120,
        sample_rate: 22050,
        channels: 2,
        scale_key: "A Minor".to_string(),
        seed: Some(777),
    };

    let res1 = engine.synthesize(&config, None, None).expect("synth 1");
    let res2 = engine.synthesize(&config, None, None).expect("synth 2");

    assert_eq!(res1.sha256, res2.sha256);
    assert_eq!(res1.wav_bytes, res2.wav_bytes);
    assert_eq!(res1.num_samples_per_channel, res2.num_samples_per_channel);
}

#[test]
fn test_stereo_and_mono_synthesis() {
    let mut engine = MusicGenEngine::new("test_music");
    engine.set_weights_loaded(true);

    let stereo_cfg = MusicGenConfig {
        prompt: "Stereo ambience".to_string(),
        duration_secs: 1.5,
        sample_rate: 22050,
        channels: 2,
        ..Default::default()
    };
    let stereo_out = engine
        .synthesize(&stereo_cfg, None, None)
        .expect("synth stereo");
    let (sr, ch, bits, data_len) =
        WavEncoder::parse_header(&stereo_out.wav_bytes).expect("parse stereo");
    assert_eq!(sr, 22050);
    assert_eq!(ch, 2);
    assert_eq!(bits, 16);
    assert_eq!(data_len, stereo_out.num_samples_per_channel * 2 * 2);

    let mono_cfg = MusicGenConfig {
        prompt: "Mono lead".to_string(),
        duration_secs: 1.5,
        sample_rate: 22050,
        channels: 1,
        ..Default::default()
    };
    let mono_out = engine
        .synthesize(&mono_cfg, None, None)
        .expect("synth mono");
    let (sr_m, ch_m, _, data_len_m) =
        WavEncoder::parse_header(&mono_out.wav_bytes).expect("parse mono");
    assert_eq!(sr_m, 22050);
    assert_eq!(ch_m, 1);
    assert_eq!(data_len_m, mono_out.num_samples_per_channel * 2);
}

#[test]
fn test_cooperative_cancellation() {
    let mut engine = MusicGenEngine::new("test_music");
    engine.set_weights_loaded(true);

    let cancel_token = Arc::new(AtomicBool::new(true)); // Pre-cancelled
    let config = MusicGenConfig {
        duration_secs: 5.0,
        ..Default::default()
    };

    let res = engine.synthesize(&config, Some(&cancel_token), None);
    assert!(matches!(res, Err(MusicError::Cancelled)));
}

#[test]
fn test_progress_telemetry_callback_fires() {
    let mut engine = MusicGenEngine::new("test_music");
    engine.set_weights_loaded(true);

    let progress_count = Arc::new(AtomicUsize::new(0));
    let progress_clone = Arc::clone(&progress_count);

    let config = MusicGenConfig {
        duration_secs: 3.0,
        tempo_bpm: 120,
        sample_rate: 22050,
        ..Default::default()
    };

    let res = engine.synthesize(
        &config,
        None,
        Some(&|_| {
            progress_clone.fetch_add(1, Ordering::SeqCst);
        }),
    );

    assert!(res.is_ok());
    assert!(progress_count.load(Ordering::SeqCst) >= 1);
}
