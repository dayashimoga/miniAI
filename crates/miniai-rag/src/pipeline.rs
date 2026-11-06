use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use sha2::{Digest, Sha256};

use crate::chunker::Chunker;
use crate::dedup::ChunkDeduplicator;
use crate::error::RagError;
use crate::security::{
    is_binary_content, is_sensitive_or_ignored, normalize_relative_path,
    validate_safe_workspace_path,
};
use crate::types::{
    ChunkRecord, DocumentRecord, DocumentType, IngestionConfig, IngestionManifest, IngestionStats,
};

/// End-to-end incremental ingestion pipeline for local documents and source code.
pub struct IngestionPipeline {
    workspace_root: PathBuf,
    config: IngestionConfig,
    chunker: Chunker,
    deduplicator: ChunkDeduplicator,
    file_records: HashMap<String, DocumentRecord>,
    chunks: Vec<ChunkRecord>,
    stats: IngestionStats,
}

impl IngestionPipeline {
    /// Creates a new ingestion pipeline scoped to `workspace_root`.
    pub fn new(workspace_root: impl Into<PathBuf>, config: IngestionConfig) -> Self {
        let raw_root = workspace_root.into();
        let root = raw_root.canonicalize().unwrap_or(raw_root);
        let chunker = Chunker::new(config.clone());
        let deduplicator = ChunkDeduplicator::new();

        Self {
            workspace_root: root,
            config,
            chunker,
            deduplicator,
            file_records: HashMap::new(),
            chunks: Vec::new(),
            stats: IngestionStats::default(),
        }
    }

    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    pub fn config(&self) -> &IngestionConfig {
        &self.config
    }

    pub fn stats(&self) -> &IngestionStats {
        &self.stats
    }

    pub fn all_chunks(&self) -> &[ChunkRecord] {
        &self.chunks
    }

    pub fn indexed_documents(&self) -> Vec<&DocumentRecord> {
        self.file_records.values().collect()
    }

    /// Ingests a single file incrementally.
    ///
    /// Returns:
    /// - `Ok(Some(chunks))` if new or modified file was indexed.
    /// - `Ok(None)` if file was excluded, binary, or skipped because content SHA-256 is unchanged.
    /// - `Err(RagError)` on path traversal or I/O failure.
    pub fn ingest_file(
        &mut self,
        candidate_path: &Path,
    ) -> Result<Option<Vec<ChunkRecord>>, RagError> {
        self.stats.files_scanned += 1;

        // Security traversal validation
        let absolute_path = validate_safe_workspace_path(candidate_path, &self.workspace_root)?;
        let relative_path = normalize_relative_path(&absolute_path, &self.workspace_root);

        // Sensitive pattern exclusion
        if is_sensitive_or_ignored(&absolute_path)
            || is_sensitive_or_ignored(Path::new(&relative_path))
        {
            self.stats.files_excluded += 1;
            return Ok(None);
        }

        // Check custom exclude patterns
        for pattern in &self.config.custom_exclude_patterns {
            if relative_path.contains(pattern) {
                self.stats.files_excluded += 1;
                return Ok(None);
            }
        }

        let metadata = fs::metadata(&absolute_path)?;
        if !metadata.is_file() {
            return Ok(None);
        }

        let size_bytes = metadata.len() as usize;
        if size_bytes > self.config.max_file_size_bytes {
            return Err(RagError::FileTooLarge {
                path: relative_path,
                actual_bytes: size_bytes,
                max_bytes: self.config.max_file_size_bytes,
            });
        }

        let raw_bytes = fs::read(&absolute_path)?;
        if is_binary_content(&raw_bytes) {
            self.stats.files_excluded += 1;
            return Ok(None);
        }

        let content =
            String::from_utf8(raw_bytes.clone()).map_err(|source| RagError::InvalidEncoding {
                path: relative_path.clone(),
                source,
            })?;

        if content.trim().is_empty() {
            self.stats.files_excluded += 1;
            return Ok(None);
        }

        // Calculate file SHA-256
        let mut hasher = Sha256::new();
        hasher.update(&raw_bytes);
        let file_sha256 = format!("{:x}", hasher.finalize());

        // Incremental cache check: If file is already indexed and digest matches, skip!
        if let Some(existing) = self.file_records.get(&relative_path) {
            if existing.file_sha256 == file_sha256 {
                self.stats.files_skipped_unchanged += 1;
                return Ok(None);
            }
            // File changed: remove old chunks first
            self.chunks.retain(|c| c.file_path != relative_path);
        }

        let doc_type = DocumentType::from_file_path(&relative_path);
        let line_count = content.lines().count();
        let now_epoch = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Chunk document
        let candidate_chunks =
            self.chunker
                .chunk_document(&relative_path, &content, &doc_type, now_epoch);

        self.stats.chunks_generated += candidate_chunks.len();

        let mut unique_file_chunks = Vec::new();
        for chunk in candidate_chunks {
            if self.deduplicator.is_unique(&chunk) {
                self.stats.unique_chunks_indexed += 1;
                self.stats.total_chars_indexed += chunk.char_count;
                unique_file_chunks.push(chunk.clone());
                self.chunks.push(chunk);
            } else {
                self.stats.duplicate_chunks_skipped += 1;
            }
        }

        let doc_record = DocumentRecord {
            file_path: relative_path.clone(),
            file_sha256,
            size_bytes,
            line_count,
            doc_type,
            last_modified_epoch_secs: now_epoch,
        };

        self.file_records.insert(relative_path, doc_record);
        self.stats.files_indexed += 1;

        Ok(Some(unique_file_chunks))
    }

    /// Recursively scans and ingests all valid files in a directory within workspace.
    pub fn ingest_directory(&mut self, dir_path: &Path) -> Result<IngestionStats, RagError> {
        let start_time = Instant::now();
        let resolved_dir = validate_safe_workspace_path(dir_path, &self.workspace_root)?;

        let mut dirs_to_visit = vec![resolved_dir];

        while let Some(current_dir) = dirs_to_visit.pop() {
            if is_sensitive_or_ignored(&current_dir) {
                continue;
            }

            let entries = match fs::read_dir(&current_dir) {
                Ok(entries) => entries,
                Err(e) => {
                    // Skip unreadable directories gracefully
                    if e.kind() == std::io::ErrorKind::PermissionDenied {
                        continue;
                    }
                    return Err(RagError::Io(e));
                }
            };

            for entry in entries {
                let entry = entry?;
                let path = entry.path();

                if is_sensitive_or_ignored(&path) {
                    continue;
                }

                if path.is_dir() {
                    dirs_to_visit.push(path);
                } else if path.is_file() {
                    let _ = self.ingest_file(&path)?;
                }
            }
        }

        self.stats.duration_ms = start_time.elapsed().as_secs_f64() * 1000.0;
        Ok(self.stats.clone())
    }

    /// Returns all indexed chunks for a given relative file path.
    pub fn get_chunks_for_file(&self, file_path: &str) -> Vec<&ChunkRecord> {
        self.chunks
            .iter()
            .filter(|c| c.file_path == file_path)
            .collect()
    }

    /// Generates a verifiable cryptographic manifest of all indexed documents and chunks.
    pub fn export_manifest(&self, workspace_id: &str) -> IngestionManifest {
        let now_epoch = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut docs: Vec<DocumentRecord> = self.file_records.values().cloned().collect();
        docs.sort_by(|a, b| a.file_path.cmp(&b.file_path));

        let manifest_id = format!("rag-manifest-{workspace_id}-{now_epoch}");

        IngestionManifest {
            manifest_id,
            workspace_id: workspace_id.to_string(),
            documents: docs,
            chunk_count: self.chunks.len(),
            unique_chunk_count: self.deduplicator.unique_count(),
            stats: self.stats.clone(),
            created_at_epoch_secs: now_epoch,
        }
    }

    /// Exports manifest JSON to disk atomically.
    pub fn save_manifest(
        &self,
        workspace_id: &str,
        output_path: &Path,
    ) -> Result<PathBuf, RagError> {
        let manifest = self.export_manifest(workspace_id);
        let json_str = serde_json::to_string_pretty(&manifest)
            .map_err(|e| RagError::Serialization(e.to_string()))?;

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let tmp_path = output_path.with_extension("tmp");
        fs::write(&tmp_path, json_str.as_bytes())?;
        fs::rename(&tmp_path, output_path)?;

        Ok(output_path.to_path_buf())
    }
}
