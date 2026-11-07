use std::collections::HashMap;

use crate::types::ChunkRecord;

/// BM25 Okapi lexical search index over chunk records.
#[derive(Debug, Clone)]
pub struct Bm25Index {
    k1: f64,
    b: f64,
    doc_count: usize,
    total_doc_len: usize,
    doc_lens: HashMap<String, usize>,
    inverted_index: HashMap<String, Vec<(String, usize)>>,
    chunk_store: HashMap<String, ChunkRecord>,
}

impl Default for Bm25Index {
    fn default() -> Self {
        Self::new(1.2, 0.75)
    }
}

impl Bm25Index {
    pub fn new(k1: f64, b: f64) -> Self {
        Self {
            k1,
            b,
            doc_count: 0,
            total_doc_len: 0,
            doc_lens: HashMap::new(),
            inverted_index: HashMap::new(),
            chunk_store: HashMap::new(),
        }
    }

    /// Tokenizes input text into normalized lower-case search terms.
    pub fn tokenize(text: &str) -> Vec<String> {
        text.split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|s| s.len() >= 2)
            .map(|s| s.to_ascii_lowercase())
            .collect()
    }

    /// Checks if a word is a common English stopword.
    pub fn is_stopword(word: &str) -> bool {
        matches!(
            word,
            "a" | "about"
                | "above"
                | "after"
                | "again"
                | "against"
                | "all"
                | "am"
                | "an"
                | "and"
                | "any"
                | "are"
                | "aren't"
                | "as"
                | "at"
                | "be"
                | "because"
                | "been"
                | "before"
                | "being"
                | "below"
                | "between"
                | "both"
                | "but"
                | "by"
                | "can't"
                | "cannot"
                | "could"
                | "couldn't"
                | "did"
                | "didn't"
                | "do"
                | "does"
                | "doesn't"
                | "doing"
                | "don't"
                | "down"
                | "during"
                | "each"
                | "few"
                | "for"
                | "from"
                | "further"
                | "had"
                | "hadn't"
                | "has"
                | "hasn't"
                | "have"
                | "haven't"
                | "having"
                | "he"
                | "he'd"
                | "he'll"
                | "he's"
                | "her"
                | "here"
                | "here's"
                | "hers"
                | "herself"
                | "him"
                | "himself"
                | "his"
                | "how"
                | "how's"
                | "i"
                | "i'd"
                | "i'll"
                | "i'm"
                | "i've"
                | "if"
                | "in"
                | "into"
                | "is"
                | "isn't"
                | "it"
                | "it's"
                | "its"
                | "itself"
                | "let's"
                | "me"
                | "more"
                | "most"
                | "mustn't"
                | "my"
                | "myself"
                | "no"
                | "nor"
                | "not"
                | "of"
                | "off"
                | "on"
                | "once"
                | "only"
                | "or"
                | "other"
                | "ought"
                | "our"
                | "ours"
                | "ourselves"
                | "out"
                | "over"
                | "own"
                | "same"
                | "shan't"
                | "she"
                | "she'd"
                | "she'll"
                | "she's"
                | "should"
                | "shouldn't"
                | "so"
                | "some"
                | "such"
                | "than"
                | "that"
                | "that's"
                | "the"
                | "their"
                | "theirs"
                | "them"
                | "themselves"
                | "then"
                | "there"
                | "there's"
                | "these"
                | "they"
                | "they'd"
                | "they'll"
                | "they're"
                | "they've"
                | "this"
                | "those"
                | "through"
                | "to"
                | "too"
                | "under"
                | "until"
                | "up"
                | "very"
                | "was"
                | "wasn't"
                | "we"
                | "we'd"
                | "we'll"
                | "we're"
                | "we've"
                | "were"
                | "weren't"
                | "what"
                | "what's"
                | "when"
                | "when's"
                | "where"
                | "where's"
                | "which"
                | "while"
                | "who"
                | "who's"
                | "whom"
                | "why"
                | "why's"
                | "with"
                | "won't"
                | "would"
                | "wouldn't"
                | "you"
                | "you'd"
                | "you'll"
                | "you're"
                | "you've"
                | "your"
                | "yours"
                | "yourself"
                | "yourselves"
        )
    }

    /// Tokenizes input text, omitting common English stopwords.
    pub fn tokenize_filtered(text: &str) -> Vec<String> {
        Self::tokenize(text)
            .into_iter()
            .filter(|tok| !Self::is_stopword(tok))
            .collect()
    }

    /// Adds a chunk to the lexical index.
    pub fn add_chunk(&mut self, chunk: ChunkRecord) {
        let chunk_id = chunk.chunk_id.clone();
        if self.chunk_store.contains_key(&chunk_id) {
            return;
        }

        let tokens = Self::tokenize(&chunk.text);
        let doc_len = tokens.len();

        let mut term_counts: HashMap<String, usize> = HashMap::new();
        for token in tokens {
            *term_counts.entry(token).or_insert(0) += 1;
        }

        for (term, count) in term_counts {
            self.inverted_index
                .entry(term)
                .or_default()
                .push((chunk_id.clone(), count));
        }

        self.doc_count += 1;
        self.total_doc_len += doc_len;
        self.doc_lens.insert(chunk_id.clone(), doc_len);
        self.chunk_store.insert(chunk_id, chunk);
    }

    /// Returns the number of indexed chunks.
    pub fn len(&self) -> usize {
        self.doc_count
    }

    pub fn is_empty(&self) -> bool {
        self.doc_count == 0
    }

    /// Average document length across indexed chunks.
    pub fn avg_doc_len(&self) -> f64 {
        if self.doc_count == 0 {
            0.0
        } else {
            self.total_doc_len as f64 / self.doc_count as f64
        }
    }

    /// Evaluates a query string and returns scored chunk IDs sorted descending by BM25 score.
    pub fn score_query(&self, query: &str) -> Vec<(String, f64)> {
        if self.doc_count == 0 {
            return Vec::new();
        }

        let filtered = Self::tokenize_filtered(query);
        let query_terms = if filtered.is_empty() {
            Self::tokenize(query)
        } else {
            filtered
        };
        if query_terms.is_empty() {
            return Vec::new();
        }

        let avg_dl = self.avg_doc_len().max(1.0);
        let mut scores: HashMap<String, f64> = HashMap::new();

        for term in &query_terms {
            if let Some(postings) = self.inverted_index.get(term) {
                let doc_freq = postings.len() as f64;
                // Robertson-Spärck Jones IDF with smoothing
                let idf = ((self.doc_count as f64 - doc_freq + 0.5) / (doc_freq + 0.5) + 1.0).ln();

                for (chunk_id, term_freq) in postings {
                    let doc_len = *self.doc_lens.get(chunk_id).unwrap_or(&1) as f64;
                    let tf = *term_freq as f64;
                    let tf_norm = (tf * (self.k1 + 1.0))
                        / (tf + self.k1 * (1.0 - self.b + self.b * (doc_len / avg_dl)));

                    *scores.entry(chunk_id.clone()).or_insert(0.0) += idf * tf_norm;
                }
            }
        }

        let mut ranked: Vec<(String, f64)> = scores.into_iter().collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked
    }

    /// Retrieves a chunk by ID from the index store.
    pub fn get_chunk(&self, chunk_id: &str) -> Option<&ChunkRecord> {
        self.chunk_store.get(chunk_id)
    }

    /// Clears the index.
    pub fn clear(&mut self) {
        self.doc_count = 0;
        self.total_doc_len = 0;
        self.doc_lens.clear();
        self.inverted_index.clear();
        self.chunk_store.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DocumentType;

    fn make_chunk(id: &str, text: &str) -> ChunkRecord {
        ChunkRecord {
            chunk_id: id.to_string(),
            file_path: format!("{id}.txt"),
            content_sha256: format!("hash_{id}"),
            text: text.to_string(),
            line_start: 1,
            line_end: 2,
            char_count: text.len(),
            token_estimate: 5,
            citation_tag: format!("{id}.txt#L1-L2"),
            doc_type: DocumentType::PlainText,
            created_at_epoch_secs: 1700000000,
        }
    }

    #[test]
    fn test_bm25_ranking_accuracy() {
        let mut index = Bm25Index::default();

        index.add_chunk(make_chunk(
            "c1",
            "Rust is a systems programming language focusing on safety and speed.",
        ));
        index.add_chunk(make_chunk(
            "c2",
            "Python is an interpreted scripting language popular for ML and data science.",
        ));
        index.add_chunk(make_chunk(
            "c3",
            "Flutter is a cross-platform UI framework written in Dart for mobile and desktop.",
        ));

        let results = index.score_query("Rust safety systems");
        assert!(!results.is_empty());
        assert_eq!(
            results[0].0, "c1",
            "c1 should be top-ranked for Rust safety query"
        );
        assert!(results[0].1 > 0.0);

        let ml_results = index.score_query("Python data science");
        assert!(!ml_results.is_empty());
        assert_eq!(
            ml_results[0].0, "c2",
            "c2 should be top-ranked for Python data query"
        );
    }

    #[test]
    fn test_bm25_empty_query() {
        let index = Bm25Index::default();
        let res = index.score_query("");
        assert!(res.is_empty());
    }
}
