use miniai_speech_asr::vad::VadEngine;

#[test]
fn test_vad_pure_silence_produces_zero_segments() {
    let vad = VadEngine::default();
    let silence = vec![0.0f32; 16000 * 2]; // 2 seconds of zero silence
    let segments = vad.segment_speech(&silence);
    assert!(segments.is_empty());
}

#[test]
fn test_vad_detects_speech_burst_boundaries() {
    let vad = VadEngine::default();
    // 0.5s silence + 1.0s sinusoidal tone (440Hz, amp 0.5) + 0.5s silence
    let mut audio = vec![0.0f32; 8000]; // 0.5s silence
    for i in 0..16000 {
        // 1.0s speech burst
        let t = i as f32 / 16000.0;
        audio.push((t * 440.0 * 2.0 * std::f32::consts::PI).sin() * 0.5);
    }
    audio.extend_from_slice(&vec![0.0f32; 8000]); // 0.5s silence

    let segments = vad.segment_speech(&audio);
    assert_eq!(segments.len(), 1);
    let seg = &segments[0];

    // Speech burst starts at ~0.5s and ends at ~1.5s (with slight hangover margin)
    assert!(
        (seg.start_secs - 0.5).abs() <= 0.1,
        "Start was {}",
        seg.start_secs
    );
    assert!(
        seg.end_secs >= 1.45 && seg.end_secs <= 1.8,
        "End was {}",
        seg.end_secs
    );
    assert!(seg.confidence > 0.8);
}

#[test]
fn test_vad_bridges_short_silence_with_hangover() {
    let vad = VadEngine::default();
    // 0.5s speech + 0.1s silence (gap < 200ms) + 0.5s speech
    let mut audio = Vec::new();
    for i in 0..8000 {
        let t = i as f32 / 16000.0;
        audio.push((t * 440.0 * 2.0 * std::f32::consts::PI).sin() * 0.4);
    }
    audio.extend_from_slice(&vec![0.0f32; 1600]); // 100ms short pause
    for i in 0..8000 {
        let t = i as f32 / 16000.0;
        audio.push((t * 440.0 * 2.0 * std::f32::consts::PI).sin() * 0.4);
    }

    let segments = vad.segment_speech(&audio);
    // Should be bridged into single continuous speech segment due to hangover
    assert_eq!(segments.len(), 1);
    assert!(segments[0].end_secs - segments[0].start_secs >= 1.0);
}

#[test]
fn test_vad_prunes_micro_bursts() {
    let vad = VadEngine::default();
    // 0.05s pulse (50ms < MIN_SPEECH_DURATION_SECS 100ms)
    let mut audio = vec![0.0f32; 4000];
    audio.extend(vec![0.5; 800]);
    audio.extend_from_slice(&vec![0.0f32; 4000]);

    let segments = vad.segment_speech(&audio);
    assert!(segments.is_empty());
}
