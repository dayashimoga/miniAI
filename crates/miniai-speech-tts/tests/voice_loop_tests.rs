use std::io::Write;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};
use tempfile::NamedTempFile;

use miniai_core::AcceleratorBackend;
use miniai_llm::{LlmEngine, GGUF_MAGIC};
use miniai_speech_asr::SpeechAsrAdapter;
use miniai_speech_tts::{
    SpeechTtsAdapter, SpeechTtsError, TtsSynthesisConfig, VoiceConversationLoop, WavEncoder,
};

fn create_dummy_llm_model_file() -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("create temp model file");
    let magic = GGUF_MAGIC.to_le_bytes();
    file.write_all(&magic).expect("write magic");
    file.write_all(&2u64.to_le_bytes()).expect("write version");
    file.write_all(&12u64.to_le_bytes())
        .expect("write tensor count");
    file.write_all(&4u64.to_le_bytes()).expect("write kv count");
    file.write_all(&vec![0u8; 1024])
        .expect("write dummy weights");
    file.flush().expect("flush");
    file
}

fn setup_voice_loop() -> (VoiceConversationLoop, NamedTempFile) {
    let asr = Arc::new(SpeechAsrAdapter::new("loop-asr-adapter"));
    let tts = Arc::new(SpeechTtsAdapter::new("loop-tts-adapter"));

    let model_file = create_dummy_llm_model_file();
    let mut engine = LlmEngine::new();
    engine
        .load_model("voice-llm", model_file.path(), AcceleratorBackend::Cpu)
        .expect("load llm model");

    let llm = Arc::new(RwLock::new(engine));
    let voice_loop = VoiceConversationLoop::new(asr, llm, tts);

    (voice_loop, model_file)
}

#[test]
fn test_voice_loop_text_turn_end_to_end() {
    let (voice_loop, _model_file) = setup_voice_loop();
    let tts_config = TtsSynthesisConfig::default();

    let result = voice_loop
        .execute_text_turn("What is the weather today?", &tts_config, None)
        .expect("execute text turn");

    assert_eq!(result.user_transcription, "What is the weather today?");
    assert!(!result.agent_response_text.is_empty());
    assert!(!result.synthesized_audio.wav_bytes.is_empty());
    assert!(result.synthesized_audio.duration_secs > 0.0);

    assert_eq!(result.metrics.asr_latency_ms, 0.0);
    assert!(result.metrics.llm_total_latency_ms >= 0.0);
    assert!(result.metrics.tts_latency_ms > 0.0);
    assert!(result.metrics.total_turnaround_ms > 0.0);
}

#[test]
fn test_voice_loop_audio_turn_end_to_end() {
    let (voice_loop, _model_file) = setup_voice_loop();
    let tts_config = TtsSynthesisConfig::default();

    // Generate 1-second synthetic 200 Hz tone at 16000 Hz to simulate user voice input
    let sample_rate = 16000;
    let mut audio_samples = Vec::with_capacity(sample_rate);
    for i in 0..sample_rate {
        let t = i as f32 / sample_rate as f32;
        audio_samples.push((2.0 * std::f32::consts::PI * 200.0 * t).sin() * 0.7);
    }
    let audio_wav = WavEncoder::encode_pcm16_mono(&audio_samples, sample_rate as u32)
        .expect("encode input audio wav");

    let result = voice_loop
        .execute_audio_turn(&audio_wav, &tts_config, None)
        .expect("execute audio turn");

    assert!(!result.user_transcription.is_empty());
    assert!(!result.agent_response_text.is_empty());
    assert!(!result.synthesized_audio.wav_bytes.is_empty());

    assert!(result.metrics.asr_latency_ms > 0.0);
    assert!(result.metrics.tts_latency_ms > 0.0);
    assert!(result.metrics.total_turnaround_ms > 0.0);
}

#[test]
fn test_voice_loop_empty_audio_rejection() {
    let (voice_loop, _model_file) = setup_voice_loop();
    let tts_config = TtsSynthesisConfig::default();

    // Pure silence produces empty transcription and triggers EmptyText
    let sample_rate = 16000;
    let silence = vec![0.0f32; sample_rate];
    let audio_wav =
        WavEncoder::encode_pcm16_mono(&silence, sample_rate as u32).expect("encode wav");

    let res = voice_loop.execute_audio_turn(&audio_wav, &tts_config, None);
    assert!(matches!(res, Err(SpeechTtsError::EmptyText)));
}

#[test]
fn test_voice_loop_cooperative_cancellation() {
    let (voice_loop, _model_file) = setup_voice_loop();
    let tts_config = TtsSynthesisConfig::default();

    let cancel_token = Arc::new(AtomicBool::new(true)); // Pre-cancelled
    let res =
        voice_loop.execute_text_turn("Cancel this request.", &tts_config, Some(&cancel_token));

    assert!(matches!(res, Err(SpeechTtsError::Cancelled)));
}

#[test]
fn test_voice_loop_system_prompt_update() {
    let (voice_loop, _model_file) = setup_voice_loop();
    assert!(voice_loop.system_prompt().contains("MiniAI"));

    voice_loop.set_system_prompt("You are a specialized code tutor.");
    assert_eq!(
        voice_loop.system_prompt(),
        "You are a specialized code tutor."
    );
}
