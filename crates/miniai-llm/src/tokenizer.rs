use std::collections::HashMap;

use crate::error::LlmError;

pub const PAD_TOKEN_ID: u32 = 0;
pub const BOS_TOKEN_ID: u32 = 1;
pub const EOS_TOKEN_ID: u32 = 2;
pub const UNK_TOKEN_ID: u32 = 3;

/// Subword and character-level tokenizer with special tokens and vocabulary mapping.
#[derive(Debug, Clone)]
pub struct Tokenizer {
    token_to_id: HashMap<String, u32>,
    id_to_token: HashMap<u32, String>,
    max_token_len: usize,
}

impl Tokenizer {
    /// Constructs a tokenizer with a standard base vocabulary.
    pub fn new_default() -> Self {
        let base_tokens = [
            "<pad>", "<s>", "</s>", "<unk>", "\n", " ", "\t", ".", ",", "!", "?", ":", ";", "-",
            "_", "(", ")", "[", "]", "{", "}", "\"", "'", "/", "\\", "@", "#", "$", "%", "^", "&",
            "*", "+", "=", "<", ">", "|", "~", "`", "0", "1", "2", "3", "4", "5", "6", "7", "8",
            "9",
        ];

        let words = [
            "the",
            "be",
            "to",
            "of",
            "and",
            "a",
            "in",
            "that",
            "have",
            "I",
            "it",
            "for",
            "not",
            "on",
            "with",
            "he",
            "as",
            "you",
            "do",
            "at",
            "this",
            "but",
            "his",
            "by",
            "from",
            "they",
            "we",
            "say",
            "her",
            "she",
            "or",
            "an",
            "will",
            "my",
            "one",
            "all",
            "would",
            "there",
            "their",
            "what",
            "so",
            "up",
            "out",
            "if",
            "about",
            "who",
            "get",
            "which",
            "go",
            "me",
            "when",
            "make",
            "can",
            "like",
            "time",
            "no",
            "just",
            "him",
            "know",
            "take",
            "people",
            "into",
            "year",
            "your",
            "good",
            "some",
            "could",
            "them",
            "see",
            "other",
            "than",
            "then",
            "now",
            "look",
            "only",
            "come",
            "its",
            "over",
            "think",
            "also",
            "back",
            "after",
            "use",
            "two",
            "how",
            "our",
            "work",
            "first",
            "well",
            "way",
            "even",
            "new",
            "want",
            "because",
            "any",
            "these",
            "give",
            "day",
            "most",
            "us",
            "is",
            "are",
            "was",
            "were",
            "been",
            "has",
            "had",
            "MiniAI",
            "AI",
            "model",
            "local",
            "system",
            "offline",
            "data",
            "compute",
            "user",
            "assistant",
            "prompt",
            "response",
            "token",
            "stream",
            "inference",
            "CPU",
            "GPU",
            "hardware",
            "memory",
            "cache",
            "privacy",
            "security",
            "true",
            "false",
            "null",
            "error",
            "success",
            "hello",
            "world",
            "yes",
            "please",
            "help",
        ];

        let mut all_tokens: Vec<String> = Vec::new();
        for &t in &base_tokens {
            all_tokens.push(t.to_string());
        }

        // Add single ASCII letters (lower & upper)
        for c in ('a'..='z').chain('A'..='Z') {
            let s = c.to_string();
            if !all_tokens.contains(&s) {
                all_tokens.push(s);
            }
        }

        // Add words with leading space and without
        for &w in &words {
            let spaced = format!(" {}", w);
            if !all_tokens.contains(&spaced) {
                all_tokens.push(spaced);
            }
            if !all_tokens.contains(&w.to_string()) {
                all_tokens.push(w.to_string());
            }
        }

        Self::from_vocab(all_tokens)
    }

    /// Constructs a tokenizer from an explicit list of token strings.
    pub fn from_vocab(vocab: Vec<String>) -> Self {
        let mut token_to_id = HashMap::with_capacity(vocab.len());
        let mut id_to_token = HashMap::with_capacity(vocab.len());
        let mut max_token_len = 1;

        for (id, token) in vocab.into_iter().enumerate() {
            let u_id = id as u32;
            max_token_len = max_token_len.max(token.len());
            token_to_id.insert(token.clone(), u_id);
            id_to_token.insert(u_id, token);
        }

        // Guarantee special tokens exist
        if !token_to_id.contains_key("<pad>") {
            token_to_id.insert("<pad>".to_string(), PAD_TOKEN_ID);
            id_to_token.insert(PAD_TOKEN_ID, "<pad>".to_string());
        }
        if !token_to_id.contains_key("<s>") {
            token_to_id.insert("<s>".to_string(), BOS_TOKEN_ID);
            id_to_token.insert(BOS_TOKEN_ID, "<s>".to_string());
        }
        if !token_to_id.contains_key("</s>") {
            token_to_id.insert("</s>".to_string(), EOS_TOKEN_ID);
            id_to_token.insert(EOS_TOKEN_ID, "</s>".to_string());
        }
        if !token_to_id.contains_key("<unk>") {
            token_to_id.insert("<unk>".to_string(), UNK_TOKEN_ID);
            id_to_token.insert(UNK_TOKEN_ID, "<unk>".to_string());
        }

        Self {
            token_to_id,
            id_to_token,
            max_token_len: max_token_len.max(16),
        }
    }

    pub fn vocab_size(&self) -> usize {
        self.token_to_id.len()
    }

    pub fn is_eos(&self, token_id: u32) -> bool {
        token_id == EOS_TOKEN_ID
    }

    /// Encodes input string into a sequence of token IDs using greedy longest-match.
    pub fn encode(&self, text: &str) -> Result<Vec<u32>, LlmError> {
        let mut tokens = Vec::new();
        let bytes = text.as_bytes();
        let mut idx = 0;

        while idx < bytes.len() {
            let mut matched_len = 0;
            let mut matched_id = None;

            // Try matches from longest possible slice down to 1
            let max_lookahead = self.max_token_len.min(bytes.len() - idx);
            for len in (1..=max_lookahead).rev() {
                if let Ok(slice) = std::str::from_utf8(&bytes[idx..idx + len]) {
                    if let Some(&id) = self.token_to_id.get(slice) {
                        matched_len = len;
                        matched_id = Some(id);
                        break;
                    }
                }
            }

            if let Some(id) = matched_id {
                tokens.push(id);
                idx += matched_len;
            } else {
                // If single char or byte not found, check single char string
                if let Some(c) = text[idx..].chars().next() {
                    let c_str = c.to_string();
                    let id = self
                        .token_to_id
                        .get(&c_str)
                        .copied()
                        .unwrap_or(UNK_TOKEN_ID);
                    tokens.push(id);
                    idx += c.len_utf8();
                } else {
                    idx += 1;
                }
            }
        }

        Ok(tokens)
    }

    /// Decodes a sequence of token IDs back into a UTF-8 string.
    pub fn decode(&self, tokens: &[u32]) -> String {
        let mut result = String::new();
        for &token_id in tokens {
            result.push_str(&self.decode_single(token_id));
        }
        result
    }

    /// Decodes a single token ID into its text representation, filtering special delimiters.
    pub fn decode_single(&self, token_id: u32) -> String {
        if token_id == PAD_TOKEN_ID || token_id == BOS_TOKEN_ID || token_id == EOS_TOKEN_ID {
            return String::new();
        }
        if let Some(token_str) = self.id_to_token.get(&token_id) {
            token_str.clone()
        } else {
            String::new()
        }
    }
}
