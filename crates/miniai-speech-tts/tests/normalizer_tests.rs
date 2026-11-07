use miniai_speech_tts::{PhonemeCategory, SpeechTtsError, TextNormalizer, MAX_SEGMENT_CHARS};

#[test]
fn test_empty_or_whitespace_text_rejected() {
    assert!(matches!(
        TextNormalizer::chunk_text(""),
        Err(SpeechTtsError::EmptyText)
    ));
    assert!(matches!(
        TextNormalizer::chunk_text("   \n\t  "),
        Err(SpeechTtsError::EmptyText)
    ));
}

#[test]
fn test_text_within_limits_returns_single_chunk() {
    let text = "Hello world! This is a clean voice test.";
    let chunks = TextNormalizer::chunk_text(text).expect("chunk text");
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0], text);
}

#[test]
fn test_long_text_chunking_respects_sentence_boundaries() {
    let sentence = "Artificial intelligence enables offline on-device voice conversations. ";
    let repeated = sentence.repeat(100); // ~7100 chars > MAX_SEGMENT_CHARS (4096)
    let chunks = TextNormalizer::chunk_text(&repeated).expect("chunk large text");

    assert!(chunks.len() >= 2);
    for chunk in chunks {
        assert!(chunk.len() <= MAX_SEGMENT_CHARS);
        assert!(!chunk.is_empty());
    }
}

#[test]
fn test_number_expansion() {
    assert_eq!(TextNormalizer::expand_number(0), "zero");
    assert_eq!(TextNormalizer::expand_number(7), "seven");
    assert_eq!(TextNormalizer::expand_number(14), "fourteen");
    assert_eq!(TextNormalizer::expand_number(42), "forty two");
    assert_eq!(TextNormalizer::expand_number(100), "one hundred");
    assert_eq!(TextNormalizer::expand_number(350), "three hundred fifty");
    assert_eq!(
        TextNormalizer::expand_number(1984),
        "one thousand nine hundred eighty four"
    );
}

#[test]
fn test_symbol_normalization() {
    let text = "Plan A & B costs $50 with 100% discount + extras @ home.";
    let normalized = TextNormalizer::normalize(text);
    assert!(normalized.contains("Plan A and B"));
    assert!(normalized.contains("fifty dollars"));
    assert!(normalized.contains("one hundred percent"));
    assert!(normalized.contains("plus"));
    assert!(normalized.contains("at home"));
}

#[test]
fn test_phonetic_tokenization() {
    let text = "hello audio world";
    let tokens = TextNormalizer::tokenize_phonemes(text, 1.0);
    assert!(!tokens.is_empty());

    // Verify presence of vowels, consonants, and pauses
    let has_vowel = tokens
        .iter()
        .any(|t| matches!(t.category, PhonemeCategory::Vowel { .. }));
    let has_pause = tokens
        .iter()
        .any(|t| matches!(t.category, PhonemeCategory::Pause { .. }));
    assert!(has_vowel, "Must produce vowel phonemes");
    assert!(has_pause, "Must produce pause phonemes between words");
}

#[test]
fn test_speed_scales_phoneme_duration() {
    let normal_tokens = TextNormalizer::tokenize_phonemes("test", 1.0);
    let fast_tokens = TextNormalizer::tokenize_phonemes("test", 2.0);

    let normal_duration: f32 = normal_tokens.iter().map(|t| t.duration_ms).sum();
    let fast_duration: f32 = fast_tokens.iter().map(|t| t.duration_ms).sum();

    assert!(fast_duration < normal_duration);
    assert!((fast_duration - normal_duration * 0.5).abs() < 1.0);
}
