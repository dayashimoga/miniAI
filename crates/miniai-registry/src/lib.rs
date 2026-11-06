#![forbid(unsafe_code)]

pub mod downloader;
pub mod error;
pub mod models;
pub mod registry;

pub use downloader::ResumableDownloader;
pub use error::RegistryError;
pub use models::{DownloadProgress, DownloadStatus, ModelManifest};
pub use registry::ModelRegistry;

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::sync::Arc;

    use miniai_core::{AcceleratorBackend, Modality, PrivacyPolicyEngine};
    use miniai_storage::StorageEngine;
    use sha2::{Digest, Sha256};
    use tempfile::tempdir;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    use super::*;

    fn create_test_manifest(
        id: &str,
        source_url: &str,
        data: &[u8],
        tampered_sha: bool,
    ) -> ModelManifest {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let actual_sha = format!("{:x}", hasher.finalize());

        let sha = if tampered_sha {
            "0000000000000000000000000000000000000000000000000000000000000000".to_string()
        } else {
            actual_sha
        };

        ModelManifest {
            model_id: id.to_string(),
            name: format!("Model {id}"),
            modality: Modality::Llm,
            version: "1.0.0".to_string(),
            description: "Test model".to_string(),
            source_url: source_url.to_string(),
            expected_sha256: sha,
            expected_size_bytes: data.len() as u64,
            format: "gguf".to_string(),
            parameter_count: Some("0.5B".to_string()),
            quantization: Some("Q4_K_M".to_string()),
            min_ram_bytes: 1_000_000_000,
            recommended_backend: AcceleratorBackend::Cpu,
        }
    }

    #[test]
    fn test_manifest_validation() {
        let valid =
            create_test_manifest("valid-model-01", "file:///tmp/test.gguf", b"hello", false);
        assert!(valid.validate().is_ok());

        // Empty model ID
        let mut invalid = valid.clone();
        invalid.model_id = "   ".to_string();
        assert!(invalid.validate().is_err());

        // Path traversal in model ID
        let mut traversal = valid.clone();
        traversal.model_id = "../../../etc/passwd".to_string();
        assert!(traversal.validate().is_err());

        // Invalid SHA-256 (not 64 chars)
        let mut bad_sha = valid.clone();
        bad_sha.expected_sha256 = "tooshort".to_string();
        assert!(bad_sha.validate().is_err());

        // Zero size
        let mut zero_size = valid.clone();
        zero_size.expected_size_bytes = 0;
        assert!(zero_size.validate().is_err());
    }

    #[tokio::test]
    async fn test_registry_manifest_lifecycle() {
        let temp = tempdir().unwrap();
        let storage = StorageEngine::in_memory().unwrap();
        let privacy = Arc::new(PrivacyPolicyEngine::default());
        let registry = ModelRegistry::new(storage, privacy, temp.path()).unwrap();

        let manifest =
            create_test_manifest("model-lifecycle", "file:///local/m.gguf", b"12345", false);
        registry.register_manifest(&manifest).unwrap();

        let fetched = registry.get_manifest("model-lifecycle").unwrap().unwrap();
        assert_eq!(fetched.name, "Model model-lifecycle");
        assert_eq!(fetched.modality, Modality::Llm);

        let list = registry.list_manifests(Some(Modality::Llm)).unwrap();
        assert_eq!(list.len(), 1);

        let vision_list = registry.list_manifests(Some(Modality::Vision)).unwrap();
        assert!(vision_list.is_empty());

        let deleted = registry.delete_manifest("model-lifecycle").unwrap();
        assert!(deleted);
        assert!(registry.get_manifest("model-lifecycle").unwrap().is_none());
    }

    #[tokio::test]
    async fn test_resumable_local_file_download_and_sha256_promotion() {
        let temp = tempdir().unwrap();
        let storage = StorageEngine::in_memory().unwrap();
        let privacy = Arc::new(PrivacyPolicyEngine::default());
        let registry = ModelRegistry::new(storage, privacy, temp.path()).unwrap();

        // Create local source file
        let src_file_path = temp.path().join("source_weights.bin");
        let payload = b"MiniAI neural model weights data payload 2026";
        std::fs::write(&src_file_path, payload).unwrap();

        let manifest = create_test_manifest(
            "local-verified-model",
            &src_file_path.to_string_lossy(),
            payload,
            false,
        );
        registry.register_manifest(&manifest).unwrap();

        let mut progress_events = Vec::new();
        let verified_path = registry
            .download_model("local-verified-model", |p| {
                progress_events.push(p);
            })
            .await
            .unwrap();

        // Verify file promoted to models dir and removed from staging
        assert!(verified_path.is_file());
        assert_eq!(
            verified_path,
            registry.models_dir().join("local-verified-model.gguf")
        );

        let part_file = registry.staging_dir().join("local-verified-model.part");
        assert!(!part_file.exists());

        // Verify recorded in SQLite cached_models
        let cached = registry
            .get_cached_model("local-verified-model")
            .unwrap()
            .unwrap();
        assert_eq!(cached.id, "local-verified-model");
        assert_eq!(cached.size_bytes, payload.len() as u64);

        // Verify cryptographic integrity
        assert!(registry
            .verify_cached_model_integrity("local-verified-model")
            .unwrap());

        // Verify progress events were emitted
        assert!(!progress_events.is_empty());
        assert_eq!(
            progress_events.last().unwrap().status,
            DownloadStatus::Verified
        );
    }

    #[tokio::test]
    async fn test_tampered_weights_rejected_and_quarantined() {
        let temp = tempdir().unwrap();
        let storage = StorageEngine::in_memory().unwrap();
        let privacy = Arc::new(PrivacyPolicyEngine::default());
        let registry = ModelRegistry::new(storage, privacy, temp.path()).unwrap();

        let src_file_path = temp.path().join("tampered_source.bin");
        let payload = b"Corrupted model weights payload";
        std::fs::write(&src_file_path, payload).unwrap();

        // Register with mismatched SHA-256
        let manifest = create_test_manifest(
            "tampered-model",
            &src_file_path.to_string_lossy(),
            payload,
            true, // Tampered SHA
        );
        registry.register_manifest(&manifest).unwrap();

        let mut progress_events = Vec::new();
        let result = registry
            .download_model("tampered-model", |p| {
                progress_events.push(p);
            })
            .await;

        assert!(matches!(result, Err(RegistryError::Sha256Mismatch { .. })));

        // Verify staging .part file was deleted
        let part_file = registry.staging_dir().join("tampered-model.part");
        assert!(!part_file.exists());

        // Verify final model was NEVER promoted
        let final_file = registry.models_dir().join("tampered-model.gguf");
        assert!(!final_file.exists());

        // Verify cached_models does not have the record
        assert!(registry
            .get_cached_model("tampered-model")
            .unwrap()
            .is_none());

        // Verify progress failed status
        assert!(matches!(
            progress_events.last().unwrap().status,
            DownloadStatus::Failed { .. }
        ));
    }

    #[tokio::test]
    async fn test_privacy_policy_blocks_remote_http_in_local_only_mode() {
        let temp = tempdir().unwrap();
        let storage = StorageEngine::in_memory().unwrap();
        let privacy = Arc::new(PrivacyPolicyEngine::default());
        assert!(privacy.is_local_only());

        let registry = ModelRegistry::new(storage, privacy, temp.path()).unwrap();

        let manifest = create_test_manifest(
            "remote-blocked-model",
            "http://weights.miniai.cloud/model.gguf",
            b"some-data",
            false,
        );
        registry.register_manifest(&manifest).unwrap();

        let result = registry
            .download_model("remote-blocked-model", |_| {})
            .await;
        assert!(matches!(
            result,
            Err(RegistryError::PrivacyModeBlocked { .. })
        ));
    }

    #[tokio::test]
    async fn test_real_http_loopback_resumable_range_download() {
        let temp = tempdir().unwrap();
        let storage = StorageEngine::in_memory().unwrap();

        // Disable local-only mode to test HTTP download
        let mut privacy_engine = PrivacyPolicyEngine::default();
        privacy_engine.set_local_only_mode(false);
        let privacy = Arc::new(privacy_engine);

        let registry = ModelRegistry::new(storage, privacy, temp.path()).unwrap();

        // 1. Setup mock HTTP server on loopback supporting Range header
        let full_payload = b"abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let server_addr = listener.local_addr().unwrap();

        let payload_clone = full_payload.to_vec();
        tokio::spawn(async move {
            while let Ok((mut socket, _)) = listener.accept().await {
                let payload = payload_clone.clone();
                tokio::spawn(async move {
                    let mut req_buf = vec![0u8; 4096];
                    let n = socket.read(&mut req_buf).await.unwrap_or(0);
                    let req_str = String::from_utf8_lossy(&req_buf[..n]);

                    // Check Range header
                    let mut start_offset = 0usize;
                    for line in req_str.lines() {
                        if line.to_lowercase().starts_with("range: bytes=") {
                            let range_val = line["range: bytes=".len()..].trim();
                            if let Some((start, _)) = range_val.split_once('-') {
                                if let Ok(s) = start.parse::<usize>() {
                                    start_offset = s;
                                }
                            }
                        }
                    }

                    if start_offset > 0 && start_offset < payload.len() {
                        // 206 Partial Content
                        let slice = &payload[start_offset..];
                        let response_header = format!(
                            "HTTP/1.1 206 Partial Content\r\n\
                             Content-Length: {}\r\n\
                             Content-Range: bytes {}-{}/{}\r\n\
                             Connection: close\r\n\r\n",
                            slice.len(),
                            start_offset,
                            payload.len() - 1,
                            payload.len()
                        );
                        let _ = socket.write_all(response_header.as_bytes()).await;
                        let _ = socket.write_all(slice).await;
                    } else {
                        // 200 OK
                        let response_header = format!(
                            "HTTP/1.1 200 OK\r\n\
                             Content-Length: {}\r\n\
                             Connection: close\r\n\r\n",
                            payload.len()
                        );
                        let _ = socket.write_all(response_header.as_bytes()).await;
                        let _ = socket.write_all(&payload).await;
                    }
                });
            }
        });

        let download_url = format!("http://{server_addr}/model.gguf");
        let manifest =
            create_test_manifest("http-resumable-model", &download_url, full_payload, false);
        registry.register_manifest(&manifest).unwrap();

        // 2. Simulate pre-existing partial file in staging (first 10 bytes)
        let part_file = registry.staging_dir().join("http-resumable-model.part");
        {
            let mut file = std::fs::File::create(&part_file).unwrap();
            file.write_all(&full_payload[..10]).unwrap();
        }
        assert_eq!(std::fs::metadata(&part_file).unwrap().len(), 10);

        // 3. Execute download: should resume from byte 10 using Range header!
        let mut progress = Vec::new();
        let promoted_path = registry
            .download_model("http-resumable-model", |p| {
                progress.push(p);
            })
            .await
            .unwrap();

        assert!(promoted_path.is_file());
        assert_eq!(std::fs::read(&promoted_path).unwrap(), full_payload);
        assert!(!part_file.exists()); // .part file removed after promotion

        // 4. Verify integrity in cache
        assert!(registry
            .verify_cached_model_integrity("http-resumable-model")
            .unwrap());

        // 5. Delete cached model
        assert!(registry
            .delete_cached_model("http-resumable-model")
            .unwrap());
        assert!(!promoted_path.exists());
        assert!(registry
            .get_cached_model("http-resumable-model")
            .unwrap()
            .is_none());
    }
}
