use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use miniai_core::validate_safe_relative_path;
use miniai_storage::models::{CachedModelRecord, ModelManifestRecord};
use miniai_storage::StorageEngine;
use sha2::{Digest, Sha256};

use crate::error::RegistryError;
use crate::models::{
    ImportProgress, ImportStatus, LocalModelImportRequest, LocalModelImportResult, ModelManifest,
};

/// Offline local model importer with quarantine staging, SHA-256 digest validation,
/// and atomic promotion into the verified cache.
pub struct ModelImporter;

impl ModelImporter {
    /// Imports a model from the host filesystem into the registry and verified cache.
    ///
    /// The import operation:
    /// 1. Validates model parameters and checks for collisions.
    /// 2. Copies the source file into a quarantined staging location while computing SHA-256 in a streaming pass.
    /// 3. Validates that the computed digest matches `expected_sha256` (if provided).
    ///    On mismatch, the quarantined file is deleted immediately and the operation fails.
    /// 4. Atomically promotes the verified file into the model cache directory.
    /// 5. Records the model manifest and cached model record in SQLite storage.
    pub fn import<F>(
        request: &LocalModelImportRequest,
        storage: &StorageEngine,
        staging_dir: &Path,
        models_dir: &Path,
        mut on_progress: F,
    ) -> Result<LocalModelImportResult, RegistryError>
    where
        F: FnMut(ImportProgress),
    {
        // 1. Validate request parameters
        let trimmed_id = request.model_id.trim();
        if trimmed_id.is_empty() {
            return Err(RegistryError::ValidationError {
                message: "Model ID cannot be empty".to_string(),
            });
        }
        validate_safe_relative_path(trimmed_id).map_err(|e| RegistryError::ValidationError {
            message: format!("Invalid model_id: {e}"),
        })?;

        if request.name.trim().is_empty() {
            return Err(RegistryError::ValidationError {
                message: "Model name cannot be empty".to_string(),
            });
        }

        let clean_format = request.format.trim().to_lowercase();
        if clean_format.is_empty() {
            return Err(RegistryError::ValidationError {
                message: "Model format cannot be empty".to_string(),
            });
        }
        validate_safe_relative_path(&clean_format).map_err(|e| RegistryError::ValidationError {
            message: format!("Invalid format: {e}"),
        })?;

        if let Some(ref sha) = request.expected_sha256 {
            let sha = sha.trim();
            if sha.len() != 64 || !sha.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(RegistryError::ValidationError {
                    message: format!(
                        "Invalid expected SHA-256 digest '{sha}': must be exactly 64 hexadecimal characters"
                    ),
                });
            }
        }

        // 2. Validate source path on host filesystem
        if !request.source_path.exists() {
            return Err(RegistryError::ValidationError {
                message: format!(
                    "Source model file does not exist: {}",
                    request.source_path.display()
                ),
            });
        }
        if !request.source_path.is_file() {
            return Err(RegistryError::ValidationError {
                message: format!(
                    "Source path is not a regular file: {}",
                    request.source_path.display()
                ),
            });
        }

        let metadata = std::fs::metadata(&request.source_path)?;
        let total_bytes = metadata.len();
        if total_bytes == 0 {
            return Err(RegistryError::ValidationError {
                message: "Source model file is empty (0 bytes)".to_string(),
            });
        }

        if let Some(expected_size) = request.expected_size_bytes {
            if total_bytes != expected_size {
                return Err(RegistryError::SizeMismatch {
                    model_id: request.model_id.clone(),
                    expected: expected_size,
                    actual: total_bytes,
                });
            }
        }

        // 3. Collision check
        if !request.overwrite
            && (storage.get_model_manifest(&request.model_id)?.is_some()
                || storage.get_cached_model(&request.model_id)?.is_some())
        {
            return Err(RegistryError::ModelAlreadyExists {
                model_id: request.model_id.clone(),
            });
        }

        // Ensure directories exist
        std::fs::create_dir_all(staging_dir)?;
        std::fs::create_dir_all(models_dir)?;

        on_progress(ImportProgress::new(
            &request.model_id,
            0,
            total_bytes,
            ImportStatus::Starting,
        ));

        // 4. Safe Quarantine Staging
        let quarantine_file = staging_dir.join(format!("{}.import.quarantine", trimmed_id));
        if quarantine_file.exists() {
            let _ = std::fs::remove_file(&quarantine_file);
        }

        on_progress(ImportProgress::new(
            &request.model_id,
            0,
            total_bytes,
            ImportStatus::StagingQuarantine,
        ));

        let copy_result = (|| -> Result<(String, u64), RegistryError> {
            let mut source_file = File::open(&request.source_path)?;
            let mut target_quarantine = File::create(&quarantine_file)?;

            let mut hasher = Sha256::new();
            let mut buffer = [0u8; 65536];
            let mut bytes_copied: u64 = 0;

            loop {
                let bytes_read = source_file.read(&mut buffer)?;
                if bytes_read == 0 {
                    break;
                }
                target_quarantine.write_all(&buffer[..bytes_read])?;
                hasher.update(&buffer[..bytes_read]);
                bytes_copied += bytes_read as u64;

                on_progress(ImportProgress::new(
                    &request.model_id,
                    bytes_copied,
                    total_bytes,
                    ImportStatus::StagingQuarantine,
                ));
            }

            target_quarantine.flush()?;
            drop(target_quarantine);

            let digest_result = hasher.finalize();
            let computed_sha256 = format!("{digest_result:x}");
            Ok((computed_sha256, bytes_copied))
        })();

        let (computed_sha256, _) = match copy_result {
            Ok(res) => res,
            Err(e) => {
                if quarantine_file.exists() {
                    let _ = std::fs::remove_file(&quarantine_file);
                }
                on_progress(ImportProgress::new(
                    &request.model_id,
                    0,
                    total_bytes,
                    ImportStatus::Failed {
                        error: e.to_string(),
                    },
                ));
                return Err(e);
            }
        };

        // 5. SHA-256 Digest Verification
        on_progress(ImportProgress::new(
            &request.model_id,
            total_bytes,
            total_bytes,
            ImportStatus::VerifyingSha256,
        ));

        if let Some(ref expected) = request.expected_sha256 {
            if !computed_sha256.eq_ignore_ascii_case(expected) {
                // Delete quarantined file immediately upon integrity failure
                if quarantine_file.exists() {
                    let _ = std::fs::remove_file(&quarantine_file);
                }
                on_progress(ImportProgress::new(
                    &request.model_id,
                    total_bytes,
                    total_bytes,
                    ImportStatus::Failed {
                        error: format!(
                            "SHA-256 digest mismatch: expected {expected}, actual {computed_sha256}"
                        ),
                    },
                ));
                return Err(RegistryError::Sha256Mismatch {
                    model_id: request.model_id.clone(),
                    expected: expected.clone(),
                    actual: computed_sha256,
                });
            }
        }

        // 6. Atomic Promotion to Verified Storage
        let model_folder = models_dir.join(trimmed_id);
        std::fs::create_dir_all(&model_folder)?;

        let final_filename = format!("{}.{}", trimmed_id, clean_format);
        let final_file = model_folder.join(&final_filename);

        if final_file.exists() && request.overwrite {
            let _ = std::fs::remove_file(&final_file);
        }

        // Attempt rename; fallback to copy+delete for cross-device moves
        if let Err(rename_err) = std::fs::rename(&quarantine_file, &final_file) {
            if std::fs::copy(&quarantine_file, &final_file).is_ok() {
                let _ = std::fs::remove_file(&quarantine_file);
            } else {
                let _ = std::fs::remove_file(&quarantine_file);
                return Err(RegistryError::IoError {
                    message: format!(
                        "Failed to promote quarantined model to verified cache: {rename_err}"
                    ),
                });
            }
        }

        // 7. Store Manifest & Cached Model Record in SQLite
        let now_epoch = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let source_url = format!(
            "file://{}",
            request.source_path.to_string_lossy().replace('\\', "/")
        );

        let manifest = ModelManifest {
            model_id: request.model_id.clone(),
            name: request.name.clone(),
            modality: request.modality,
            version: request.version.clone(),
            description: request.description.clone(),
            source_url: source_url.clone(),
            expected_sha256: computed_sha256.clone(),
            expected_size_bytes: total_bytes,
            format: request.format.clone(),
            parameter_count: request.parameter_count.clone(),
            quantization: request.quantization.clone(),
            min_ram_bytes: request.min_ram_bytes,
            recommended_backend: request.recommended_backend,
        };

        let manifest_record = ModelManifestRecord {
            model_id: manifest.model_id.clone(),
            name: manifest.name.clone(),
            modality: manifest.modality,
            version: manifest.version.clone(),
            description: manifest.description.clone(),
            source_url,
            expected_sha256: computed_sha256.clone(),
            expected_size_bytes: total_bytes,
            format: manifest.format.clone(),
            parameter_count: manifest.parameter_count.clone(),
            quantization: manifest.quantization.clone(),
            min_ram_bytes: manifest.min_ram_bytes,
            recommended_backend: manifest.recommended_backend,
            created_at_epoch_secs: now_epoch,
        };

        let cached_record = CachedModelRecord {
            id: request.model_id.clone(),
            modality: request.modality,
            file_path: final_file.to_string_lossy().to_string(),
            sha256: computed_sha256.clone(),
            size_bytes: total_bytes,
            preferred_backend: request.recommended_backend,
            created_at_epoch_secs: now_epoch,
        };

        if request.overwrite {
            let _ = storage.delete_model_manifest(&request.model_id);
            let _ = storage.delete_cached_model(&request.model_id);
        }

        storage.insert_model_manifest(&manifest_record)?;
        storage.insert_cached_model(&cached_record)?;

        on_progress(ImportProgress::new(
            &request.model_id,
            total_bytes,
            total_bytes,
            ImportStatus::Promoted,
        ));

        Ok(LocalModelImportResult {
            manifest,
            cached_model: cached_record,
            verified_path: final_file,
            sha256: computed_sha256,
            size_bytes: total_bytes,
        })
    }
}
