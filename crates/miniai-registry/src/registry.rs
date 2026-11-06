use std::path::{Path, PathBuf};
use std::sync::Arc;

use miniai_core::{Modality, PrivacyPolicyEngine};
use miniai_storage::models::{CachedModelRecord, ModelManifestRecord};
use miniai_storage::StorageEngine;

use crate::downloader::ResumableDownloader;
use crate::error::RegistryError;
use crate::models::{DownloadProgress, ModelManifest};

/// Central model catalog, resumable downloader, and integrity verification supervisor.
pub struct ModelRegistry {
    storage: StorageEngine,
    privacy: Arc<PrivacyPolicyEngine>,
    cache_root: PathBuf,
    staging_dir: PathBuf,
    models_dir: PathBuf,
}

impl ModelRegistry {
    /// Creates or opens a `ModelRegistry` at the specified cache root directory.
    pub fn new(
        storage: StorageEngine,
        privacy: Arc<PrivacyPolicyEngine>,
        cache_root: impl AsRef<Path>,
    ) -> Result<Self, RegistryError> {
        let cache_root = cache_root.as_ref().to_path_buf();
        let models_dir = cache_root.join("models");
        let staging_dir = models_dir.join(".staging");

        std::fs::create_dir_all(&staging_dir)?;
        std::fs::create_dir_all(&models_dir)?;

        Ok(Self {
            storage,
            privacy,
            cache_root,
            staging_dir,
            models_dir,
        })
    }

    pub fn cache_root(&self) -> &Path {
        &self.cache_root
    }

    pub fn staging_dir(&self) -> &Path {
        &self.staging_dir
    }

    pub fn models_dir(&self) -> &Path {
        &self.models_dir
    }

    /// Registers a new model manifest into the catalog store.
    pub fn register_manifest(&self, manifest: &ModelManifest) -> Result<(), RegistryError> {
        manifest.validate()?;

        let record = ModelManifestRecord {
            model_id: manifest.model_id.clone(),
            name: manifest.name.clone(),
            modality: manifest.modality,
            version: manifest.version.clone(),
            description: manifest.description.clone(),
            source_url: manifest.source_url.clone(),
            expected_sha256: manifest.expected_sha256.to_lowercase(),
            expected_size_bytes: manifest.expected_size_bytes,
            format: manifest.format.clone(),
            parameter_count: manifest.parameter_count.clone(),
            quantization: manifest.quantization.clone(),
            min_ram_bytes: manifest.min_ram_bytes,
            recommended_backend: manifest.recommended_backend,
            created_at_epoch_secs: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        self.storage.insert_model_manifest(&record)?;
        Ok(())
    }

    /// Retrieves a model manifest by ID.
    pub fn get_manifest(&self, model_id: &str) -> Result<Option<ModelManifest>, RegistryError> {
        let record = self.storage.get_model_manifest(model_id)?;
        Ok(record.map(Self::record_to_manifest))
    }

    /// Lists model manifests, optionally filtered by modality.
    pub fn list_manifests(
        &self,
        modality: Option<Modality>,
    ) -> Result<Vec<ModelManifest>, RegistryError> {
        let records = self.storage.list_model_manifests(modality)?;
        Ok(records.into_iter().map(Self::record_to_manifest).collect())
    }

    /// Deletes a model manifest from the catalog.
    pub fn delete_manifest(&self, model_id: &str) -> Result<bool, RegistryError> {
        Ok(self.storage.delete_model_manifest(model_id)?)
    }

    /// Downloads and verifies a model from the registry.
    ///
    /// Checks `PrivacyPolicyEngine`: if Local-Only Privacy Mode is enabled,
    /// remote HTTP downloads are strictly blocked.
    pub async fn download_model<F>(
        &self,
        model_id: &str,
        progress_callback: F,
    ) -> Result<PathBuf, RegistryError>
    where
        F: FnMut(DownloadProgress) + Send,
    {
        let manifest =
            self.get_manifest(model_id)?
                .ok_or_else(|| RegistryError::ModelNotFound {
                    model_id: model_id.to_string(),
                })?;

        // Privacy check: Block remote downloads if Local-Only Privacy Mode is active
        if self.privacy.is_local_only() && manifest.source_url.starts_with("http") {
            return Err(RegistryError::PrivacyModeBlocked {
                model_id: model_id.to_string(),
                url: manifest.source_url.clone(),
            });
        }

        let verified_path = ResumableDownloader::download(
            &manifest,
            &self.staging_dir,
            &self.models_dir,
            progress_callback,
        )
        .await?;

        // Register the verified model in SQLite cached_models
        let cached_record = CachedModelRecord {
            id: manifest.model_id.clone(),
            modality: manifest.modality,
            file_path: verified_path.to_string_lossy().to_string(),
            sha256: manifest.expected_sha256.to_lowercase(),
            size_bytes: manifest.expected_size_bytes,
            preferred_backend: manifest.recommended_backend,
            created_at_epoch_secs: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        self.storage.insert_cached_model(&cached_record)?;
        Ok(verified_path)
    }

    /// Gets a cached model record from storage.
    pub fn get_cached_model(
        &self,
        model_id: &str,
    ) -> Result<Option<CachedModelRecord>, RegistryError> {
        Ok(self.storage.get_cached_model(model_id)?)
    }

    /// Lists all cached models.
    pub fn list_cached_models(
        &self,
        modality: Option<Modality>,
    ) -> Result<Vec<CachedModelRecord>, RegistryError> {
        Ok(self.storage.list_cached_models(modality)?)
    }

    /// Verifies the on-disk cryptographic integrity of a cached model against its recorded SHA-256.
    pub fn verify_cached_model_integrity(&self, model_id: &str) -> Result<bool, RegistryError> {
        let cached = match self.get_cached_model(model_id)? {
            Some(c) => c,
            None => {
                return Err(RegistryError::ModelNotFound {
                    model_id: model_id.to_string(),
                })
            }
        };

        let file_path = Path::new(&cached.file_path);
        if !file_path.is_file() {
            return Ok(false);
        }

        let computed_sha = ResumableDownloader::compute_file_sha256(file_path)?;
        Ok(computed_sha.eq_ignore_ascii_case(&cached.sha256))
    }

    /// Deletes a cached model from both disk and the storage database.
    pub fn delete_cached_model(&self, model_id: &str) -> Result<bool, RegistryError> {
        if let Some(cached) = self.get_cached_model(model_id)? {
            let path = PathBuf::from(cached.file_path);
            if path.exists() {
                let _ = std::fs::remove_file(path);
            }
            self.storage.delete_cached_model(model_id)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn record_to_manifest(r: ModelManifestRecord) -> ModelManifest {
        ModelManifest {
            model_id: r.model_id,
            name: r.name,
            modality: r.modality,
            version: r.version,
            description: r.description,
            source_url: r.source_url,
            expected_sha256: r.expected_sha256,
            expected_size_bytes: r.expected_size_bytes,
            format: r.format,
            parameter_count: r.parameter_count,
            quantization: r.quantization,
            min_ram_bytes: r.min_ram_bytes,
            recommended_backend: r.recommended_backend,
        }
    }
}
