use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::bm25::Bm25Index;
use crate::embedding::{LocalEmbeddingModel, VectorIndex};
use crate::pipeline::IngestionPipeline;
use crate::types::ChunkRecord;

/// Search configuration parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalConfig {
    /// RRF smoothing constant (standard default is 60).
    pub rrf_k: f64,
    /// Weight assigned to dense semantic rank (0.0 to 1.0).
    pub dense_weight: f64,
    /// Weight assigned to BM25 lexical rank (0.0 to 1.0).
    pub lexical_weight: f64,
    /// Maximum candidate chunks fetched from each ranker before fusion.
    pub candidate_pool_size: usize,
}

impl Default for RetrievalConfig {
    fn default() -> Self {
        Self {
            rrf_k: 60.0,
            dense_weight: 0.5,
            lexical_weight: 0.5,
            candidate_pool_size: 50,
        }
    }
}

/// Grounded search result with exact source line citations and highlighted excerpt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResult {
    pub chunk: ChunkRecord,
    pub rrf_score: f64,
    pub dense_score: f64,
    pub lexical_score: f64,
    pub citation_tag: String,
    pub excerpt: String,
}

/// Unified local hybrid search engine combining BM25 lexical retrieval and CPU dense embeddings.
pub struct HybridSearchEngine {
    bm25: Bm25Index,
    vector_index: VectorIndex,
    embedding_model: LocalEmbeddingModel,
    config: RetrievalConfig,
    chunk_map: HashMap<String, ChunkRecord>,
}

impl Default for HybridSearchEngine {
    fn default() -> Self {
        Self::new(RetrievalConfig::default())
    }
}

impl HybridSearchEngine {
    pub fn new(config: RetrievalConfig) -> Self {
        Self {
            bm25: Bm25Index::default(),
            vector_index: VectorIndex::new(),
            embedding_model: LocalEmbeddingModel::default(),
            config,
            chunk_map: HashMap::new(),
        }
    }

    /// Indexes a single chunk record into both lexical and vector indices.
    pub fn index_chunk(&mut self, chunk: ChunkRecord) {
        let chunk_id = chunk.chunk_id.clone();
        if self.chunk_map.contains_key(&chunk_id) {
            return;
        }

        let vector = self.embedding_model.embed_chunk(&chunk);
        self.bm25.add_chunk(chunk.clone());
        self.vector_index.add(chunk.clone(), vector);
        self.chunk_map.insert(chunk_id, chunk);
    }

    /// Bulk indexes all unique chunks produced by an `IngestionPipeline`.
    pub fn index_pipeline_chunks(&mut self, pipeline: &IngestionPipeline) -> usize {
        let mut count = 0;
        for chunk in pipeline.all_chunks() {
            self.index_chunk(chunk.clone());
            count += 1;
        }
        count
    }

    /// Returns the total number of indexed chunks.
    pub fn len(&self) -> usize {
        self.chunk_map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chunk_map.is_empty()
    }

    /// Executes a hybrid search combining dense semantic similarity and BM25 lexical ranking
    /// via Reciprocal Rank Fusion (RRF).
    pub fn search(&self, query: &str, limit: usize) -> Vec<RetrievalResult> {
        let trimmed_query = query.trim();
        if trimmed_query.is_empty() || self.chunk_map.is_empty() {
            return Vec::new();
        }

        let pool_limit = self.config.candidate_pool_size.max(limit * 2);

        // 1. Lexical BM25 Ranking
        let bm25_hits = self.bm25.score_query(trimmed_query);
        let mut lexical_scores: HashMap<String, f64> = HashMap::new();
        let mut lexical_ranks: HashMap<String, usize> = HashMap::new();

        for (rank, (chunk_id, score)) in bm25_hits.into_iter().take(pool_limit).enumerate() {
            lexical_scores.insert(chunk_id.clone(), score);
            lexical_ranks.insert(chunk_id, rank + 1); // 1-indexed rank
        }

        // 2. Dense Semantic Vector Ranking
        let query_vector = self.embedding_model.embed_text(trimmed_query);
        let dense_hits = self.vector_index.search(&query_vector, pool_limit);
        let mut dense_scores: HashMap<String, f64> = HashMap::new();
        let mut dense_ranks: HashMap<String, usize> = HashMap::new();

        for (rank, (chunk, score)) in dense_hits.into_iter().enumerate() {
            dense_scores.insert(chunk.chunk_id.clone(), score as f64);
            dense_ranks.insert(chunk.chunk_id, rank + 1);
        }

        // 3. Reciprocal Rank Fusion (RRF)
        let mut candidate_ids: Vec<String> = lexical_ranks.keys().cloned().collect();
        for id in dense_ranks.keys() {
            if !candidate_ids.contains(id) {
                candidate_ids.push(id.clone());
            }
        }

        let k = self.config.rrf_k;
        let mut fused: Vec<(String, f64)> = Vec::new();

        for id in &candidate_ids {
            let lex_rank = lexical_ranks.get(id).copied().unwrap_or(pool_limit + 10);
            let dense_rank = dense_ranks.get(id).copied().unwrap_or(pool_limit + 10);

            let rrf = (self.config.lexical_weight / (k + lex_rank as f64))
                + (self.config.dense_weight / (k + dense_rank as f64));

            fused.push((id.clone(), rrf));
        }

        fused.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        fused.truncate(limit);

        // 4. Construct grounded result structures
        fused
            .into_iter()
            .filter_map(|(id, rrf_score)| {
                let chunk = self.chunk_map.get(&id)?.clone();
                let lex_score = lexical_scores.get(&id).copied().unwrap_or(0.0);
                let dense_score = dense_scores.get(&id).copied().unwrap_or(0.0);
                let excerpt = extract_excerpt(&chunk.text, trimmed_query, 240);
                let citation_tag = chunk.citation_tag.clone();

                Some(RetrievalResult {
                    chunk,
                    rrf_score,
                    dense_score,
                    lexical_score: lex_score,
                    citation_tag,
                    excerpt,
                })
            })
            .collect()
    }
}

/// Extracts a contextual snippet surrounding matching terms in text.
fn extract_excerpt(text: &str, query: &str, max_len: usize) -> String {
    let lower_text = text.to_ascii_lowercase();
    let query_terms = crate::bm25::Bm25Index::tokenize(query);

    let mut first_match = None;
    for term in &query_terms {
        if let Some(pos) = lower_text.find(term) {
            first_match = Some(pos);
            break;
        }
    }

    if let Some(pos) = first_match {
        let start = pos.saturating_sub(max_len / 4);
        let end = (start + max_len).min(text.len());
        let slice = &text[start..end];
        let prefix = if start > 0 { "..." } else { "" };
        let suffix = if end < text.len() { "..." } else { "" };
        format!("{prefix}{slice}{suffix}")
    } else {
        let end = max_len.min(text.len());
        let suffix = if end < text.len() { "..." } else { "" };
        format!("{}{suffix}", &text[..end])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DocumentType;

    fn make_chunk(id: &str, file: &str, start: usize, end: usize, text: &str) -> ChunkRecord {
        ChunkRecord {
            chunk_id: id.to_string(),
            file_path: file.to_string(),
            content_sha256: format!("hash_{id}"),
            text: text.to_string(),
            line_start: start,
            line_end: end,
            char_count: text.len(),
            token_estimate: 8,
            citation_tag: ChunkRecord::format_citation(file, start, end),
            doc_type: DocumentType::PlainText,
            created_at_epoch_secs: 1700000000,
        }
    }

    #[test]
    fn test_hybrid_search_rrf_scoring() {
        let mut engine = HybridSearchEngine::default();

        let chunk1 = make_chunk(
            "c1",
            "crates/core.rs",
            10,
            25,
            "The AdmissionGuard prevents out-of-memory crashes by falling back to CPU execution when memory exceeds 85 percent threshold.",
        );
        let chunk2 = make_chunk(
            "c2",
            "docs/privacy.md",
            1,
            15,
            "LocalOnlyPrivacyMode blocks all remote network compute unless explicit CloudConsentToken is granted by the user.",
        );
        let chunk3 = make_chunk(
            "c3",
            "ml/training.py",
            50,
            80,
            "LoRA adapters apply low-rank matrix decomposition W0 + alpha/r * (B @ A) with zero-delta B=0 initialization.",
        );

        engine.index_chunk(chunk1);
        engine.index_chunk(chunk2);
        engine.index_chunk(chunk3);

        assert_eq!(engine.len(), 3);

        // Search for admission guard memory fallback
        let results = engine.search("AdmissionGuard CPU fallback memory", 2);
        assert!(!results.is_empty());
        assert_eq!(results[0].chunk.chunk_id, "c1");
        assert_eq!(results[0].citation_tag, "crates/core.rs#L10-L25");
        assert!(results[0].rrf_score > 0.0);
        assert!(results[0].excerpt.contains("AdmissionGuard"));

        // Search for LoRA low-rank
        let lora_results = engine.search("LoRA low-rank decomposition", 1);
        assert!(!lora_results.is_empty());
        assert_eq!(lora_results[0].chunk.chunk_id, "c3");
        assert_eq!(lora_results[0].citation_tag, "ml/training.py#L50-L80");
    }

    #[test]
    fn test_hybrid_search_empty_query() {
        let engine = HybridSearchEngine::default();
        let res = engine.search("", 5);
        assert!(res.is_empty());
    }
}
