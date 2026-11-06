use sha2::{Digest, Sha256};

use crate::types::ChunkRecord;

pub const DEFAULT_EMBEDDING_DIM: usize = 128;

/// Computes the cosine similarity between two float vectors.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;

    for i in 0..a.len() {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }

    if norm_a <= 0.0 || norm_b <= 0.0 {
        0.0
    } else {
        (dot / (norm_a.sqrt() * norm_b.sqrt())).clamp(-1.0, 1.0)
    }
}

/// Normalizes a vector in-place to unit Euclidean length (L2 norm = 1.0).
pub fn l2_normalize(vec: &mut [f32]) {
    let norm_sq: f32 = vec.iter().map(|&x| x * x).sum();
    if norm_sq > 0.0 {
        let inv_norm = 1.0 / norm_sq.sqrt();
        for val in vec.iter_mut() {
            *val *= inv_norm;
        }
    }
}

/// Local CPU-first deterministic embedding generator.
///
/// Uses high-dimensional pseudo-random subword feature hashing and token projections
/// to generate robust dense semantic representations without requiring cloud or heavy GPU dependencies.
#[derive(Debug, Clone)]
pub struct LocalEmbeddingModel {
    dimension: usize,
}

impl Default for LocalEmbeddingModel {
    fn default() -> Self {
        Self::new(DEFAULT_EMBEDDING_DIM)
    }
}

impl LocalEmbeddingModel {
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension: dimension.max(16),
        }
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Generates a dense L2-normalized embedding vector for text content.
    pub fn embed_text(&self, text: &str) -> Vec<f32> {
        let mut vector = vec![0.0f32; self.dimension];
        let tokens = crate::bm25::Bm25Index::tokenize(text);

        if tokens.is_empty() {
            return vector;
        }

        for (pos, token) in tokens.iter().enumerate() {
            // Compute deterministic token hash
            let mut hasher = Sha256::new();
            hasher.update(token.as_bytes());
            let hash = hasher.finalize();

            // Position decay weight: early words in chunks carry slightly higher salience
            let pos_weight = 1.0 / (1.0 + (pos as f32 * 0.02));

            // Accumulate projected values across dimensions
            for (dim_idx, slot) in vector.iter_mut().enumerate() {
                let byte_idx = (dim_idx * 2) % 32;
                let sign = if (hash[byte_idx] & 0x01) == 0 {
                    1.0
                } else {
                    -1.0
                };
                let val = (hash[(byte_idx + 1) % 32] as f32 / 255.0) * sign * pos_weight;
                *slot += val;
            }

            // Also hash character 3-grams for subword semantic overlap
            if token.len() >= 3 {
                for i in 0..=token.len() - 3 {
                    let trigram = &token[i..i + 3];
                    let mut tri_hasher = Sha256::new();
                    tri_hasher.update(trigram.as_bytes());
                    let tri_hash = tri_hasher.finalize();

                    for (dim_idx, slot) in vector.iter_mut().enumerate() {
                        let byte_idx = (dim_idx * 3) % 32;
                        let sign = if (tri_hash[byte_idx] & 0x02) == 0 {
                            1.0
                        } else {
                            -1.0
                        };
                        let val = (tri_hash[(byte_idx + 1) % 32] as f32 / 255.0) * sign * 0.3;
                        *slot += val;
                    }
                }
            }
        }

        l2_normalize(&mut vector);
        vector
    }

    /// Embeds a chunk record directly.
    pub fn embed_chunk(&self, chunk: &ChunkRecord) -> Vec<f32> {
        self.embed_text(&chunk.text)
    }
}

/// In-memory dense vector index with cosine similarity search.
#[derive(Debug, Default, Clone)]
pub struct VectorIndex {
    entries: Vec<(ChunkRecord, Vec<f32>)>,
}

impl VectorIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, chunk: ChunkRecord, vector: Vec<f32>) {
        self.entries.push((chunk, vector));
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Performs exact cosine similarity search over indexed dense vectors.
    pub fn search(&self, query_vec: &[f32], top_k: usize) -> Vec<(ChunkRecord, f32)> {
        let mut scored: Vec<(ChunkRecord, f32)> = self
            .entries
            .iter()
            .map(|(chunk, vec)| {
                let score = cosine_similarity(query_vec, vec);
                (chunk.clone(), score)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DocumentType;

    #[test]
    fn test_cosine_similarity_properties() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let c = vec![0.0, 1.0, 0.0];

        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-5);
        assert!((cosine_similarity(&a, &c) - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_local_embedding_deterministic_and_normalized() {
        let model = LocalEmbeddingModel::default();
        let text1 = "MiniAI local-first multimodal platform";
        let text2 = "MiniAI local-first multimodal platform";
        let text3 = "Something completely unrelated about cooking recipes";

        let v1 = model.embed_text(text1);
        let v2 = model.embed_text(text2);
        let v3 = model.embed_text(text3);

        assert_eq!(v1.len(), DEFAULT_EMBEDDING_DIM);
        assert_eq!(v1, v2, "Identical text must generate identical embeddings");

        // Check L2 unit norm
        let norm_sq: f32 = v1.iter().map(|&x| x * x).sum();
        assert!((norm_sq - 1.0).abs() < 1e-4);

        // High similarity for identical, lower for unrelated
        let sim_same = cosine_similarity(&v1, &v2);
        let sim_diff = cosine_similarity(&v1, &v3);
        assert!((sim_same - 1.0).abs() < 1e-5);
        assert!(sim_same > sim_diff);
    }

    #[test]
    fn test_vector_index_search() {
        let model = LocalEmbeddingModel::default();
        let mut index = VectorIndex::new();

        let chunk = ChunkRecord {
            chunk_id: "c1".to_string(),
            file_path: "test.txt".to_string(),
            content_sha256: "hash1".to_string(),
            text: "Rust compiler safety and memory guarantees".to_string(),
            line_start: 1,
            line_end: 2,
            char_count: 42,
            token_estimate: 6,
            citation_tag: "test.txt#L1-L2".to_string(),
            doc_type: DocumentType::PlainText,
            created_at_epoch_secs: 1700000000,
        };

        let vec = model.embed_chunk(&chunk);
        index.add(chunk, vec);

        let query_vec = model.embed_text("Rust memory safety");
        let results = index.search(&query_vec, 1);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0.chunk_id, "c1");
        assert!(results[0].1 > 0.4);
    }
}
