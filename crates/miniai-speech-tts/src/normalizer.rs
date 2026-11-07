use crate::error::SpeechTtsError;

/// Maximum allowed character length for a single synthesis segment.
pub const MAX_SEGMENT_CHARS: usize = 4096;

/// Phonetic unit category for acoustic synthesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhonemeCategory {
    /// Voiced vowel with distinct formant resonance peaks.
    Vowel { f1: u32, f2: u32, f3: u32 },
    /// Voiced resonant consonant (nasals, liquids, semivowels).
    VoicedConsonant { f1: u32, f2: u32 },
    /// Unvoiced fricative or sibilant synthesized via filtered noise.
    UnvoicedFricative,
    /// Transient burst plosive with pre-burst closure.
    Plosive,
    /// Silence or pause between words or sentences.
    Pause { duration_ms: u32 },
}

/// Tokenized phonetic unit ready for acoustic engine synthesis.
#[derive(Debug, Clone, PartialEq)]
pub struct PhonemeToken {
    pub symbol: char,
    pub category: PhonemeCategory,
    pub duration_ms: f32,
    pub stress: f32,
}

/// Text preprocessor, number expander, and chunker.
pub struct TextNormalizer;

impl TextNormalizer {
    /// Normalizes and splits text into chunks strictly under `MAX_SEGMENT_CHARS`.
    pub fn chunk_text(text: &str) -> Result<Vec<String>, SpeechTtsError> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(SpeechTtsError::EmptyText);
        }

        if text.len() > MAX_SEGMENT_CHARS {
            // Split into paragraphs or sentences
            let mut chunks = Vec::new();
            let mut current = String::new();

            for sentence in Self::split_into_sentences(trimmed) {
                if sentence.len() > MAX_SEGMENT_CHARS {
                    return Err(SpeechTtsError::TextTooLong {
                        length: sentence.len(),
                        max: MAX_SEGMENT_CHARS,
                    });
                }

                if current.len() + sentence.len() + 1 > MAX_SEGMENT_CHARS {
                    chunks.push(current.trim().to_string());
                    current = sentence;
                } else {
                    if !current.is_empty() {
                        current.push(' ');
                    }
                    current.push_str(&sentence);
                }
            }

            if !current.trim().is_empty() {
                chunks.push(current.trim().to_string());
            }

            Ok(chunks)
        } else {
            Ok(vec![trimmed.to_string()])
        }
    }

    /// Expands common symbols and numbers in text to spoken words.
    pub fn normalize(text: &str) -> String {
        let mut out = String::with_capacity(text.len() * 2);
        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];

            if c == '$' {
                // Peek ahead for numbers: $100 -> one hundred dollars
                let mut num_str = String::new();
                let mut j = i + 1;
                while j < chars.len() && chars[j].is_ascii_digit() {
                    num_str.push(chars[j]);
                    j += 1;
                }
                if !num_str.is_empty() {
                    if let Ok(num) = num_str.parse::<u64>() {
                        out.push_str(&Self::expand_number(num));
                        out.push_str(" dollars");
                        i = j;
                        continue;
                    }
                }
                out.push('$');
                i += 1;
            } else if c.is_ascii_digit() {
                let mut num_str = String::new();
                let mut j = i;
                while j < chars.len() && chars[j].is_ascii_digit() {
                    num_str.push(chars[j]);
                    j += 1;
                }
                if let Ok(num) = num_str.parse::<u64>() {
                    out.push_str(&Self::expand_number(num));
                } else {
                    out.push_str(&num_str);
                }
                i = j;
            } else if c == '%' {
                out.push_str(" percent");
                i += 1;
            } else if c == '&' {
                out.push_str(" and ");
                i += 1;
            } else if c == '+' {
                out.push_str(" plus ");
                i += 1;
            } else if c == '@' {
                out.push_str(" at ");
                i += 1;
            } else {
                out.push(c);
                i += 1;
            }
        }

        // Collapse multiple spaces
        let words: Vec<&str> = out.split_whitespace().collect();
        words.join(" ")
    }

    /// Converts normalized text into a sequence of phonetic tokens for formant synthesis.
    pub fn tokenize_phonemes(normalized_text: &str, speed: f32) -> Vec<PhonemeToken> {
        let speed_factor = (1.0 / speed.clamp(0.5, 2.0)).clamp(0.5, 2.0);
        let mut tokens = Vec::new();

        let lower = normalized_text.to_lowercase();
        let chars: Vec<char> = lower.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];

            if c.is_whitespace() {
                tokens.push(PhonemeToken {
                    symbol: ' ',
                    category: PhonemeCategory::Pause { duration_ms: 60 },
                    duration_ms: 60.0 * speed_factor,
                    stress: 0.0,
                });
                i += 1;
                continue;
            }

            match c {
                '.' | '!' | '?' => {
                    tokens.push(PhonemeToken {
                        symbol: c,
                        category: PhonemeCategory::Pause { duration_ms: 180 },
                        duration_ms: 180.0 * speed_factor,
                        stress: 0.0,
                    });
                }
                ',' | ';' | ':' => {
                    tokens.push(PhonemeToken {
                        symbol: c,
                        category: PhonemeCategory::Pause { duration_ms: 100 },
                        duration_ms: 100.0 * speed_factor,
                        stress: 0.0,
                    });
                }
                'a' => tokens.push(PhonemeToken {
                    symbol: 'a',
                    category: PhonemeCategory::Vowel {
                        f1: 800,
                        f2: 1200,
                        f3: 2500,
                    },
                    duration_ms: 110.0 * speed_factor,
                    stress: 1.0,
                }),
                'e' => tokens.push(PhonemeToken {
                    symbol: 'e',
                    category: PhonemeCategory::Vowel {
                        f1: 500,
                        f2: 1800,
                        f3: 2600,
                    },
                    duration_ms: 95.0 * speed_factor,
                    stress: 0.9,
                }),
                'i' => tokens.push(PhonemeToken {
                    symbol: 'i',
                    category: PhonemeCategory::Vowel {
                        f1: 300,
                        f2: 2300,
                        f3: 3000,
                    },
                    duration_ms: 90.0 * speed_factor,
                    stress: 0.9,
                }),
                'o' => tokens.push(PhonemeToken {
                    symbol: 'o',
                    category: PhonemeCategory::Vowel {
                        f1: 500,
                        f2: 900,
                        f3: 2400,
                    },
                    duration_ms: 105.0 * speed_factor,
                    stress: 1.0,
                }),
                'u' => tokens.push(PhonemeToken {
                    symbol: 'u',
                    category: PhonemeCategory::Vowel {
                        f1: 350,
                        f2: 800,
                        f3: 2250,
                    },
                    duration_ms: 100.0 * speed_factor,
                    stress: 0.85,
                }),
                'y' => tokens.push(PhonemeToken {
                    symbol: 'y',
                    category: PhonemeCategory::Vowel {
                        f1: 320,
                        f2: 2000,
                        f3: 2800,
                    },
                    duration_ms: 85.0 * speed_factor,
                    stress: 0.8,
                }),
                's' | 'f' | 'h' | 'x' | 'z' => tokens.push(PhonemeToken {
                    symbol: c,
                    category: PhonemeCategory::UnvoicedFricative,
                    duration_ms: 70.0 * speed_factor,
                    stress: 0.5,
                }),
                'p' | 't' | 'k' | 'b' | 'd' | 'g' => tokens.push(PhonemeToken {
                    symbol: c,
                    category: PhonemeCategory::Plosive,
                    duration_ms: 55.0 * speed_factor,
                    stress: 0.6,
                }),
                'm' | 'n' | 'l' | 'r' | 'w' | 'v' => tokens.push(PhonemeToken {
                    symbol: c,
                    category: PhonemeCategory::VoicedConsonant { f1: 300, f2: 1400 },
                    duration_ms: 75.0 * speed_factor,
                    stress: 0.7,
                }),
                _ => {
                    // Generic default consonant
                    tokens.push(PhonemeToken {
                        symbol: c,
                        category: PhonemeCategory::VoicedConsonant { f1: 350, f2: 1200 },
                        duration_ms: 60.0 * speed_factor,
                        stress: 0.5,
                    });
                }
            }
            i += 1;
        }

        tokens
    }

    /// Splits text into sentence fragments based on punctuation.
    fn split_into_sentences(text: &str) -> Vec<String> {
        let mut sentences = Vec::new();
        let mut cur = String::new();

        for c in text.chars() {
            cur.push(c);
            if c == '.' || c == '!' || c == '?' || c == '\n' {
                let trimmed = cur.trim();
                if !trimmed.is_empty() {
                    sentences.push(trimmed.to_string());
                }
                cur.clear();
            }
        }
        let trimmed = cur.trim();
        if !trimmed.is_empty() {
            sentences.push(trimmed.to_string());
        }
        sentences
    }

    /// Expands an integer into English words up to 999,999.
    pub fn expand_number(n: u64) -> String {
        if n == 0 {
            return "zero".to_string();
        }

        let under_20 = [
            "",
            "one",
            "two",
            "three",
            "four",
            "five",
            "six",
            "seven",
            "eight",
            "nine",
            "ten",
            "eleven",
            "twelve",
            "thirteen",
            "fourteen",
            "fifteen",
            "sixteen",
            "seventeen",
            "eighteen",
            "nineteen",
        ];

        let tens = [
            "", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
        ];

        if n < 20 {
            return under_20[n as usize].to_string();
        }

        if n < 100 {
            let t = (n / 10) as usize;
            let rem = (n % 10) as usize;
            if rem == 0 {
                return tens[t].to_string();
            } else {
                return format!("{} {}", tens[t], under_20[rem]);
            }
        }

        if n < 1000 {
            let h = (n / 100) as usize;
            let rem = n % 100;
            if rem == 0 {
                return format!("{} hundred", under_20[h]);
            } else {
                return format!("{} hundred {}", under_20[h], Self::expand_number(rem));
            }
        }

        if n < 1_000_000 {
            let thousands = n / 1000;
            let rem = n % 1000;
            if rem == 0 {
                return format!("{} thousand", Self::expand_number(thousands));
            } else {
                return format!(
                    "{} thousand {}",
                    Self::expand_number(thousands),
                    Self::expand_number(rem)
                );
            }
        }

        // Large number fallback
        n.to_string()
    }
}
