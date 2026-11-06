use std::path::PathBuf;

use miniai_core::{validate_safe_relative_path, AcceleratorBackend, Modality};
use miniai_storage::models::CachedModelRecord;
use serde::{Deserialize, Serialize};

use crate::error::RegistryError;

/// Canonical manifest describing an AI model artifact available in the registry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelManifest {
    pub model_id: String,
    pub name: String,
    pub modality: Modality,
    pub version: String,
    pub description: String,
    pub source_url: String,
    pub expected_sha256: String,
    pub expected_size_bytes: u64,
    pub format: String,
    pub parameter_count: Option<String>,
    pub quantization: Option<String>,
    pub min_ram_bytes: u64,
    pub recommended_backend: AcceleratorBackend,
}

impl ModelManifest {
    /// Validates the manifest fields for security and correctness.
    pub fn validate(&self) -> Result<(), RegistryError> {
        let trimmed_id = self.model_id.trim();
        if trimmed_id.is_empty() {
            return Err(RegistryError::ValidationError {
                message: "Model ID cannot be empty".to_string(),
            });
        }

        // Prevent directory traversal in model_id
        validate_safe_relative_path(trimmed_id).map_err(|e| RegistryError::ValidationError {
            message: format!("Invalid model_id: {e}"),
        })?;

        if self.name.trim().is_empty() {
            return Err(RegistryError::ValidationError {
                message: "Model name cannot be empty".to_string(),
            });
        }

        let sha = self.expected_sha256.trim();
        if sha.len() != 64 || !sha.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(RegistryError::ValidationError {
                message: format!(
                    "Invalid SHA-256 digest '{sha}': must be exactly 64 hexadecimal characters"
                ),
            });
        }

        if self.expected_size_bytes == 0 {
            return Err(RegistryError::ValidationError {
                message: "Expected size in bytes must be greater than 0".to_string(),
            });
        }

        if self.format.trim().is_empty() {
            return Err(RegistryError::ValidationError {
                message: "Model format cannot be empty".to_string(),
            });
        }

        Ok(())
    }
}

/// Download status lifecycle state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DownloadStatus {
    Pending,
    Downloading {
        bytes_downloaded: u64,
        total_bytes: u64,
    },
    VerifyingSha256,
    Verified,
    Failed {
        error: String,
    },
    Paused,
}

/// Real-time download progress report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DownloadProgress {
    pub model_id: String,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub percentage: f32,
    pub status: DownloadStatus,
}

impl DownloadProgress {
    pub fn new(
        model_id: impl Into<String>,
        downloaded: u64,
        total: u64,
        status: DownloadStatus,
    ) -> Self {
        let percentage = if total > 0 {
            ((downloaded as f64 / total as f64) * 100.0) as f32
        } else {
            0.0
        };
        Self {
            model_id: model_id.into(),
            bytes_downloaded: downloaded,
            total_bytes: total,
            percentage,
            status,
        }
    }
}

/// Request parameters for importing an existing local model file into the registry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalModelImportRequest {
    /// Absolute or relative path on the host filesystem to the model weight file.
    pub source_path: PathBuf,
    /// Canonical model ID (e.g. "llama-3.2-1b-instruct-q4"). Must satisfy safe relative path constraints.
    pub model_id: String,
    /// Human-readable display name.
    pub name: String,
    /// Modality category for the model.
    pub modality: Modality,
    /// Version tag (e.g. "1.0.0").
    pub version: String,
    /// Human-readable description.
    pub description: String,
    /// Weight format (e.g. "gguf", "onnx", "safetensors", "pt").
    pub format: String,
    /// Parameter count string (e.g. "1.23B", "7B").
    pub parameter_count: Option<String>,
    /// Quantization level (e.g. "Q4_K_M", "FP16").
    pub quantization: Option<String>,
    /// Minimum required RAM in bytes.
    pub min_ram_bytes: u64,
    /// Recommended hardware execution backend.
    pub recommended_backend: AcceleratorBackend,
    /// Expected SHA-256 digest (64 hex characters) if known.
    /// If provided, the importer verifies that the imported file matches this digest.
    /// If omitted (None), the importer computes the digest from the file directly.
    pub expected_sha256: Option<String>,
    /// Expected file size in bytes if known.
    pub expected_size_bytes: Option<u64>,
    /// Whether to replace an existing model with the same ID.
    pub overwrite: bool,
}

/// Lifecycle status of an offline local model import operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImportStatus {
    Starting,
    StagingQuarantine,
    VerifyingSha256,
    Promoted,
    Failed { error: String },
}

/// Real-time progress report during a local model import operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImportProgress {
    pub model_id: String,
    pub bytes_copied: u64,
    pub total_bytes: u64,
    pub percentage: f32,
    pub status: ImportStatus,
}

impl ImportProgress {
    pub fn new(model_id: impl Into<String>, copied: u64, total: u64, status: ImportStatus) -> Self {
        let percentage = if total > 0 {
            ((copied as f64 / total as f64) * 100.0) as f32
        } else {
            0.0
        };
        Self {
            model_id: model_id.into(),
            bytes_copied: copied,
            total_bytes: total,
            percentage,
            status,
        }
    }
}

/// Result of a successful local model import operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalModelImportResult {
    pub manifest: ModelManifest,
    pub cached_model: CachedModelRecord,
    pub verified_path: PathBuf,
    pub sha256: String,
    pub size_bytes: u64,
}
