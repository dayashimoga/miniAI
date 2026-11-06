use std::collections::{HashMap, HashSet};

use crate::types::ChunkRecord;

/// Content-addressed deduplicator for chunk records.
#[derive(Debug, Default, Clone)]
pub struct ChunkDeduplicator {
    seen_content_hashes: HashSet<String>,
    content_to_canonical_chunk: HashMap<String, String>,
    duplicate_count: usize,
}

impl ChunkDeduplicator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluates a chunk. Returns `true` if the chunk is unique (new content),
    /// or `false` if an identical chunk content has already been indexed.
    pub fn is_unique(&mut self, chunk: &ChunkRecord) -> bool {
        if self.seen_content_hashes.contains(&chunk.content_sha256) {
            self.duplicate_count += 1;
            false
        } else {
            self.seen_content_hashes
                .insert(chunk.content_sha256.clone());
            self.content_to_canonical_chunk
                .insert(chunk.content_sha256.clone(), chunk.chunk_id.clone());
            true
        }
    }

    /// Filters a list of chunks, retaining only the unique chunks while counting duplicates.
    pub fn filter_unique(&mut self, chunks: Vec<ChunkRecord>) -> Vec<ChunkRecord> {
        let mut unique = Vec::new();
        for chunk in chunks {
            if self.is_unique(&chunk) {
                unique.push(chunk);
            }
        }
        unique
    }

    /// Returns the total number of unique chunk contents observed.
    pub fn unique_count(&self) -> usize {
        self.seen_content_hashes.len()
    }

    /// Returns the total number of duplicate chunks rejected.
    pub fn duplicate_count(&self) -> usize {
        self.duplicate_count
    }

    /// Returns the canonical chunk ID for a given content hash if known.
    pub fn get_canonical_chunk_id(&self, content_hash: &str) -> Option<&str> {
        self.content_to_canonical_chunk
            .get(content_hash)
            .map(|s| s.as_str())
    }

    /// Resets all deduplication state.
    pub fn clear(&mut self) {
        self.seen_content_hashes.clear();
        self.content_to_canonical_chunk.clear();
        self.duplicate_count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DocumentType;

    fn make_test_chunk(id: &str, content_hash: &str) -> ChunkRecord {
        ChunkRecord {
            chunk_id: id.to_string(),
            file_path: "test.txt".to_string(),
            content_sha256: content_hash.to_string(),
            text: "sample text".to_string(),
            line_start: 1,
            line_end: 2,
            char_count: 11,
            token_estimate: 3,
            citation_tag: "test.txt#L1-L2".to_string(),
            doc_type: DocumentType::PlainText,
            created_at_epoch_secs: 1700000000,
        }
    }

    #[test]
    fn test_deduplicator_filters_duplicates() {
        let mut dedup = ChunkDeduplicator::new();

        let chunk1 = make_test_chunk("c1", "hash_aaa");
        let chunk2 = make_test_chunk("c2", "hash_bbb");
        let chunk3 = make_test_chunk("c3", "hash_aaa"); // duplicate

        assert!(dedup.is_unique(&chunk1));
        assert!(dedup.is_unique(&chunk2));
        assert!(!dedup.is_unique(&chunk3));

        assert_eq!(dedup.unique_count(), 2);
        assert_eq!(dedup.duplicate_count(), 1);
        assert_eq!(dedup.get_canonical_chunk_id("hash_aaa"), Some("c1"));
    }
}
