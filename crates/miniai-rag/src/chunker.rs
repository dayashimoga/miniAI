use sha2::{Digest, Sha256};

use crate::types::{ChunkRecord, DocumentType, IngestionConfig};

/// Structure-aware text and code chunker.
pub struct Chunker {
    config: IngestionConfig,
}

impl Chunker {
    pub fn new(config: IngestionConfig) -> Self {
        Self { config }
    }

    /// Chunks document text into structured, content-addressed `ChunkRecord` items with
    /// exact line number citations.
    pub fn chunk_document(
        &self,
        file_path: &str,
        content: &str,
        doc_type: &DocumentType,
        created_at_epoch_secs: u64,
    ) -> Vec<ChunkRecord> {
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return Vec::new();
        }

        match doc_type {
            DocumentType::Markdown => {
                self.chunk_markdown(file_path, &lines, doc_type, created_at_epoch_secs)
            }
            DocumentType::Code { .. } => {
                self.chunk_code(file_path, &lines, doc_type, created_at_epoch_secs)
            }
            DocumentType::PlainText | DocumentType::PdfText => {
                self.chunk_prose(file_path, &lines, doc_type, created_at_epoch_secs)
            }
        }
    }

    /// Chunks Markdown by respecting section headings, code fence blocks, and blank line boundaries.
    fn chunk_markdown(
        &self,
        file_path: &str,
        lines: &[&str],
        doc_type: &DocumentType,
        created_at_epoch_secs: u64,
    ) -> Vec<ChunkRecord> {
        let mut chunks = Vec::new();
        let mut current_lines: Vec<&str> = Vec::new();
        let mut chunk_start_line = 1;
        let mut in_code_fence = false;

        for (idx, &line) in lines.iter().enumerate() {
            let current_line_no = idx + 1;
            let trimmed = line.trim_start();

            if trimmed.starts_with("```") {
                in_code_fence = !in_code_fence;
            }

            let is_heading = !in_code_fence && (trimmed.starts_with('#'));
            let current_len: usize = current_lines.iter().map(|l| l.len() + 1).sum();

            // If we encounter a heading and already have enough content, close the chunk
            if is_heading
                && current_len >= self.config.max_chunk_chars / 2
                && !current_lines.is_empty()
            {
                let chunk_end_line = current_line_no - 1;
                chunks.push(self.build_chunk_record(
                    file_path,
                    &current_lines,
                    chunk_start_line,
                    chunk_end_line,
                    doc_type,
                    created_at_epoch_secs,
                ));

                // Overlap lines: keep last few lines if not exceeding overlap chars
                let overlap = self.compute_overlap_lines(&current_lines);
                chunk_start_line = current_line_no.saturating_sub(overlap.len());
                current_lines = overlap;
            }

            current_lines.push(line);

            // If length exceeds target, close chunk at current position
            let total_len: usize = current_lines.iter().map(|l| l.len() + 1).sum();
            if total_len >= self.config.max_chunk_chars {
                chunks.push(self.build_chunk_record(
                    file_path,
                    &current_lines,
                    chunk_start_line,
                    current_line_no,
                    doc_type,
                    created_at_epoch_secs,
                ));

                let overlap = self.compute_overlap_lines(&current_lines);
                chunk_start_line = (current_line_no + 1).saturating_sub(overlap.len());
                current_lines = overlap;
            }
        }

        if !current_lines.is_empty() {
            let chunk_end_line = lines.len();
            chunks.push(self.build_chunk_record(
                file_path,
                &current_lines,
                chunk_start_line,
                chunk_end_line,
                doc_type,
                created_at_epoch_secs,
            ));
        }

        chunks
    }

    /// Chunks source code while respecting function/class boundaries and bracket scopes.
    fn chunk_code(
        &self,
        file_path: &str,
        lines: &[&str],
        doc_type: &DocumentType,
        created_at_epoch_secs: u64,
    ) -> Vec<ChunkRecord> {
        let mut chunks = Vec::new();
        let mut current_lines: Vec<&str> = Vec::new();
        let mut chunk_start_line = 1;

        for (idx, &line) in lines.iter().enumerate() {
            let current_line_no = idx + 1;
            let trimmed = line.trim();

            let is_scope_break = trimmed == "}"
                || trimmed == "};"
                || (trimmed.is_empty() && current_lines.len() > 10);
            let current_len: usize = current_lines.iter().map(|l| l.len() + 1).sum();

            current_lines.push(line);

            if current_len >= self.config.max_chunk_chars
                || (is_scope_break && current_len >= (self.config.max_chunk_chars * 3) / 4)
            {
                chunks.push(self.build_chunk_record(
                    file_path,
                    &current_lines,
                    chunk_start_line,
                    current_line_no,
                    doc_type,
                    created_at_epoch_secs,
                ));

                let overlap = self.compute_overlap_lines(&current_lines);
                chunk_start_line = (current_line_no + 1).saturating_sub(overlap.len());
                current_lines = overlap;
            }
        }

        if !current_lines.is_empty() {
            chunks.push(self.build_chunk_record(
                file_path,
                &current_lines,
                chunk_start_line,
                lines.len(),
                doc_type,
                created_at_epoch_secs,
            ));
        }

        chunks
    }

    /// Chunks prose text (plain text or extracted PDF text) on paragraphs and sliding windows.
    fn chunk_prose(
        &self,
        file_path: &str,
        lines: &[&str],
        doc_type: &DocumentType,
        created_at_epoch_secs: u64,
    ) -> Vec<ChunkRecord> {
        let mut chunks = Vec::new();
        let mut current_lines: Vec<&str> = Vec::new();
        let mut chunk_start_line = 1;

        for (idx, &line) in lines.iter().enumerate() {
            let current_line_no = idx + 1;
            let is_paragraph_boundary = line.trim().is_empty();
            let current_len: usize = current_lines.iter().map(|l| l.len() + 1).sum();

            if is_paragraph_boundary
                && current_len >= self.config.max_chunk_chars / 2
                && !current_lines.is_empty()
            {
                chunks.push(self.build_chunk_record(
                    file_path,
                    &current_lines,
                    chunk_start_line,
                    current_line_no - 1,
                    doc_type,
                    created_at_epoch_secs,
                ));

                let overlap = self.compute_overlap_lines(&current_lines);
                chunk_start_line = current_line_no.saturating_sub(overlap.len());
                current_lines = overlap;
            }

            current_lines.push(line);

            let total_len: usize = current_lines.iter().map(|l| l.len() + 1).sum();
            if total_len >= self.config.max_chunk_chars {
                chunks.push(self.build_chunk_record(
                    file_path,
                    &current_lines,
                    chunk_start_line,
                    current_line_no,
                    doc_type,
                    created_at_epoch_secs,
                ));

                let overlap = self.compute_overlap_lines(&current_lines);
                chunk_start_line = (current_line_no + 1).saturating_sub(overlap.len());
                current_lines = overlap;
            }
        }

        if !current_lines.is_empty() {
            chunks.push(self.build_chunk_record(
                file_path,
                &current_lines,
                chunk_start_line,
                lines.len(),
                doc_type,
                created_at_epoch_secs,
            ));
        }

        chunks
    }

    /// Computes overlap lines to carry over to the subsequent chunk.
    fn compute_overlap_lines<'a>(&self, lines: &[&'a str]) -> Vec<&'a str> {
        let mut overlap = Vec::new();
        let mut accumulated_chars = 0;

        for &line in lines.iter().rev() {
            accumulated_chars += line.len() + 1;
            overlap.push(line);
            if accumulated_chars >= self.config.chunk_overlap_chars {
                break;
            }
        }

        overlap.reverse();
        // Don't carry over all lines as overlap if chunk was short
        if overlap.len() >= lines.len() {
            overlap.drain(..lines.len() / 2);
        }
        overlap
    }

    /// Assembles a complete `ChunkRecord` with SHA-256 hashes, citation tag, and token estimates.
    fn build_chunk_record(
        &self,
        file_path: &str,
        lines: &[&str],
        line_start: usize,
        line_end: usize,
        doc_type: &DocumentType,
        created_at_epoch_secs: u64,
    ) -> ChunkRecord {
        let text = lines.join("\n");
        let char_count = text.len();

        // SHA-256 of text content alone (for deduplication)
        let mut content_hasher = Sha256::new();
        content_hasher.update(text.as_bytes());
        let content_sha256 = format!("{:x}", content_hasher.finalize());

        // Deterministic unique chunk_id = sha256(file_path:start:end:content_sha256)
        let mut id_hasher = Sha256::new();
        id_hasher
            .update(format!("{file_path}:{line_start}:{line_end}:{content_sha256}").as_bytes());
        let chunk_id = format!("{:x}", id_hasher.finalize());

        let word_count = text.split_whitespace().count();
        let token_estimate = (word_count as f64 * 1.3).ceil() as usize;
        let citation_tag = ChunkRecord::format_citation(file_path, line_start, line_end);

        ChunkRecord {
            chunk_id,
            file_path: file_path.to_string(),
            content_sha256,
            text,
            line_start,
            line_end,
            char_count,
            token_estimate,
            citation_tag,
            doc_type: doc_type.clone(),
            created_at_epoch_secs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_markdown_headings_and_lines() {
        let chunker = Chunker::new(IngestionConfig {
            max_chunk_chars: 100,
            chunk_overlap_chars: 20,
            ..Default::default()
        });

        let md = "# Title 1\n\nParagraph 1 is here with some content.\n\n# Title 2\n\nParagraph 2 is here.";
        let chunks = chunker.chunk_document("doc.md", md, &DocumentType::Markdown, 1700000000);

        assert!(!chunks.is_empty());
        assert_eq!(chunks[0].line_start, 1);
        assert!(chunks[0].citation_tag.starts_with("doc.md#L1"));
        assert!(chunks[0].token_estimate > 0);
    }

    #[test]
    fn test_chunk_code_lines() {
        let chunker = Chunker::new(IngestionConfig {
            max_chunk_chars: 120,
            chunk_overlap_chars: 20,
            ..Default::default()
        });

        let code = "fn main() {\n    println!(\"Hello, world!\");\n}\n\nfn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n";
        let chunks = chunker.chunk_document(
            "src/main.rs",
            code,
            &DocumentType::Code {
                language: "rust".to_string(),
            },
            1700000000,
        );

        assert!(!chunks.is_empty());
        for chunk in &chunks {
            assert!(chunk.line_start <= chunk.line_end);
            assert_eq!(chunk.file_path, "src/main.rs");
            assert!(chunk.citation_tag.contains("src/main.rs#L"));
        }
    }

    #[test]
    fn test_empty_content_returns_empty_chunks() {
        let chunker = Chunker::new(IngestionConfig::default());
        let chunks = chunker.chunk_document("empty.txt", "", &DocumentType::PlainText, 1700000000);
        assert!(chunks.is_empty());
    }
}
