use miniai_core::{validate_safe_relative_path, AcceleratorBackend, Modality};
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
