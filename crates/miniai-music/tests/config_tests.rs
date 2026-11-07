use miniai_music::{MusicError, MusicGenConfig, MusicalScale};

#[test]
fn test_default_config_validates() {
    let config = MusicGenConfig::default();
    assert!(config.validate().is_ok());
    assert_eq!(config.channels, 2);
    assert_eq!(config.sample_rate, 44100);
}

#[test]
fn test_empty_prompt_rejected() {
    let config = MusicGenConfig {
        prompt: "   ".to_string(),
        ..Default::default()
    };
    assert!(matches!(config.validate(), Err(MusicError::PromptEmpty)));
}

#[test]
fn test_duration_bounds_enforcement() {
    let too_short = MusicGenConfig {
        duration_secs: 0.5,
        ..Default::default()
    };
    assert!(matches!(
        too_short.validate(),
        Err(MusicError::DurationOutOfBounds { .. })
    ));

    let too_long = MusicGenConfig {
        duration_secs: 150.0,
        ..Default::default()
    };
    assert!(matches!(
        too_long.validate(),
        Err(MusicError::DurationOutOfBounds { .. })
    ));
}

#[test]
fn test_tempo_bounds_enforcement() {
    let too_slow = MusicGenConfig {
        tempo_bpm: 20,
        ..Default::default()
    };
    assert!(matches!(
        too_slow.validate(),
        Err(MusicError::TempoOutOfBounds { .. })
    ));

    let too_fast = MusicGenConfig {
        tempo_bpm: 300,
        ..Default::default()
    };
    assert!(matches!(
        too_fast.validate(),
        Err(MusicError::TempoOutOfBounds { .. })
    ));
}

#[test]
fn test_unsupported_sample_rate_rejected() {
    let bad_sr = MusicGenConfig {
        sample_rate: 96000,
        ..Default::default()
    };
    assert!(matches!(
        bad_sr.validate(),
        Err(MusicError::InvalidParameter(_))
    ));
}

#[test]
fn test_unsupported_channels_rejected() {
    let bad_ch = MusicGenConfig {
        channels: 6,
        ..Default::default()
    };
    assert!(matches!(
        bad_ch.validate(),
        Err(MusicError::InvalidParameter(_))
    ));
}

#[test]
fn test_musical_scale_parsing() {
    assert_eq!(MusicalScale::from_name("C Major"), MusicalScale::CMajor);
    assert_eq!(MusicalScale::from_name("a_minor"), MusicalScale::AMinor);
    assert_eq!(MusicalScale::from_name("G Major"), MusicalScale::GMajor);
    assert_eq!(MusicalScale::from_name("e min"), MusicalScale::EMinor);
    assert_eq!(MusicalScale::from_name("D Dorian"), MusicalScale::DDorian);
    assert_eq!(MusicalScale::from_name("unknown"), MusicalScale::CMajor);
}
