#![forbid(unsafe_code)]

pub mod downloader;
pub mod error;
pub mod importer;
pub mod models;
pub mod registry;

pub use downloader::ResumableDownloader;
pub use error::RegistryError;
pub use importer::ModelImporter;
pub use models::{
    DownloadProgress, DownloadStatus, ImportProgress, ImportStatus, LocalModelImportRequest,
    LocalModelImportResult, ModelManifest,
};
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

    #[tokio::test]
    async fn test_http_redirect_chain_is_followed_and_verified() {
        let temp = tempdir().unwrap();
        let storage = StorageEngine::in_memory().unwrap();
        let mut privacy_engine = PrivacyPolicyEngine::default();
        privacy_engine.set_local_only_mode(false);
        let registry = ModelRegistry::new(storage, Arc::new(privacy_engine), temp.path()).unwrap();

        let payload = b"redirected-cdn-artifact-0123456789";
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let payload_vec = payload.to_vec();
        tokio::spawn(async move {
            while let Ok((mut socket, _)) = listener.accept().await {
                let payload = payload_vec.clone();
                tokio::spawn(async move {
                    let mut buf = vec![0u8; 4096];
                    let n = socket.read(&mut buf).await.unwrap_or(0);
                    let req = String::from_utf8_lossy(&buf[..n]).to_string();
                    let first = req.lines().next().unwrap_or("").to_string();
                    let resp = if first.contains("/resolve/main/m.gguf") {
                        // Origin-relative redirect (HuggingFace -> CDN style)
                        "HTTP/1.1 302 Found\r\nLocation: /cdn/signed-m.gguf?sig=abc\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string()
                    } else {
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            payload.len()
                        )
                    };
                    let _ = socket.write_all(resp.as_bytes()).await;
                    if first.contains("/cdn/signed-m.gguf") {
                        let _ = socket.write_all(&payload).await;
                    }
                });
            }
        });

        let url = format!("http://{addr}/resolve/main/m.gguf");
        let manifest = create_test_manifest("redirected-model", &url, payload, false);
        registry.register_manifest(&manifest).unwrap();

        let path = registry
            .download_model("redirected-model", |_| {})
            .await
            .unwrap();
        assert_eq!(std::fs::read(path).unwrap(), payload);
    }

    #[tokio::test]
    async fn test_redirect_loop_is_rejected() {
        let temp = tempdir().unwrap();
        let storage = StorageEngine::in_memory().unwrap();
        let mut privacy_engine = PrivacyPolicyEngine::default();
        privacy_engine.set_local_only_mode(false);
        let registry = ModelRegistry::new(storage, Arc::new(privacy_engine), temp.path()).unwrap();

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            while let Ok((mut socket, _)) = listener.accept().await {
                tokio::spawn(async move {
                    let mut buf = vec![0u8; 4096];
                    let _ = socket.read(&mut buf).await;
                    let _ = socket
                        .write_all(
                            b"HTTP/1.1 301 Moved\r\nLocation: /loop\r\nConnection: close\r\n\r\n",
                        )
                        .await;
                });
            }
        });

        let manifest = create_test_manifest(
            "loop-model",
            &format!("http://{addr}/loop"),
            b"never-delivered",
            false,
        );
        registry.register_manifest(&manifest).unwrap();
        let result = registry.download_model("loop-model", |_| {}).await;
        assert!(matches!(result, Err(RegistryError::HttpError { .. })));
    }

    #[test]
    fn test_verified_registry_url_allowlist_rejects_spoofing() {
        assert!(ModelRegistry::is_verified_registry_url(
            "https://huggingface.co/bartowski/x/resolve/main/m.gguf"
        ));
        assert!(ModelRegistry::is_verified_registry_url(
            "https://cdn-lfs.hf.co/repos/m.gguf"
        ));
        assert!(!ModelRegistry::is_verified_registry_url(
            "http://huggingface.co/m.gguf"
        ));
        assert!(!ModelRegistry::is_verified_registry_url(
            "https://huggingface.co.evil.example/m.gguf"
        ));
        assert!(!ModelRegistry::is_verified_registry_url(
            "https://huggingface.co@evil.example/m.gguf"
        ));
        assert!(!ModelRegistry::is_verified_registry_url(
            "https://evilhuggingface.co/m.gguf"
        ));
        assert!(!ModelRegistry::is_verified_registry_url(
            "file:///tmp/m.gguf"
        ));
    }

    #[tokio::test]
    async fn test_acknowledged_download_still_blocked_for_unverified_host_or_without_ack() {
        let temp = tempdir().unwrap();
        let storage = StorageEngine::in_memory().unwrap();
        let privacy = Arc::new(PrivacyPolicyEngine::default());
        assert!(privacy.is_local_only());
        let registry = ModelRegistry::new(storage, privacy, temp.path()).unwrap();

        let hf = create_test_manifest(
            "hf-model",
            "https://huggingface.co/org/repo/resolve/main/m.gguf",
            b"x",
            false,
        );
        let evil = create_test_manifest(
            "evil-model",
            "https://weights.evil.example/m.gguf",
            b"x",
            false,
        );
        registry.register_manifest(&hf).unwrap();
        registry.register_manifest(&evil).unwrap();

        // No acknowledgement -> blocked even for a verified host
        assert!(matches!(
            registry
                .download_model_acknowledged("hf-model", false, |_| {})
                .await,
            Err(RegistryError::PrivacyModeBlocked { .. })
        ));
        // Acknowledged but host not pinned -> blocked
        assert!(matches!(
            registry
                .download_model_acknowledged("evil-model", true, |_| {})
                .await,
            Err(RegistryError::PrivacyModeBlocked { .. })
        ));
    }

    #[test]
    fn test_import_local_model_with_expected_sha256_success() {
        let temp = tempdir().unwrap();
        let storage = StorageEngine::in_memory().unwrap();
        let privacy = Arc::new(PrivacyPolicyEngine::default());
        let registry = ModelRegistry::new(storage, privacy, temp.path()).unwrap();

        // 1. Create a dummy model file on host
        let host_model_file = temp.path().join("local_weights.gguf");
        let model_bytes = b"GGUF_MAGIC_HEADER_AND_QUANTIZED_TENSORS_123456789";
        std::fs::write(&host_model_file, model_bytes).unwrap();

        let mut hasher = Sha256::new();
        hasher.update(model_bytes);
        let expected_sha256 = format!("{:x}", hasher.finalize());

        let request = LocalModelImportRequest {
            source_path: host_model_file.clone(),
            model_id: "imported-llama-01".to_string(),
            name: "Imported LLaMA 01".to_string(),
            modality: Modality::Llm,
            version: "1.0.0".to_string(),
            description: "Locally imported GGUF weights".to_string(),
            format: "gguf".to_string(),
            parameter_count: Some("1B".to_string()),
            quantization: Some("Q4_K_M".to_string()),
            min_ram_bytes: 512_000_000,
            recommended_backend: AcceleratorBackend::Cpu,
            expected_sha256: Some(expected_sha256.clone()),
            expected_size_bytes: Some(model_bytes.len() as u64),
            overwrite: false,
        };

        let mut progress_events = Vec::new();
        let result = registry
            .import_local_model(request, |p| progress_events.push(p))
            .expect("import must succeed");

        assert_eq!(result.sha256, expected_sha256);
        assert_eq!(result.size_bytes, model_bytes.len() as u64);
        assert!(result.verified_path.is_file());
        assert_eq!(std::fs::read(&result.verified_path).unwrap(), model_bytes);

        // Verify staging quarantine file was cleaned up
        let staging_quarantine = registry
            .staging_dir()
            .join("imported-llama-01.import.quarantine");
        assert!(!staging_quarantine.exists());

        // Verify SQLite manifest
        let manifest = registry
            .get_manifest("imported-llama-01")
            .unwrap()
            .expect("manifest registered");
        assert_eq!(manifest.expected_sha256, expected_sha256);
        assert_eq!(manifest.expected_size_bytes, model_bytes.len() as u64);

        // Verify SQLite cached model
        let cached = registry
            .get_cached_model("imported-llama-01")
            .unwrap()
            .expect("cached model registered");
        assert_eq!(cached.sha256, expected_sha256);

        // Verify integrity check
        assert!(registry
            .verify_cached_model_integrity("imported-llama-01")
            .unwrap());

        // Verify progress events had all states
        let statuses: Vec<_> = progress_events.into_iter().map(|p| p.status).collect();
        assert!(statuses.contains(&ImportStatus::Starting));
        assert!(statuses.contains(&ImportStatus::StagingQuarantine));
        assert!(statuses.contains(&ImportStatus::VerifyingSha256));
        assert!(statuses.contains(&ImportStatus::Promoted));
    }

    #[test]
    fn test_import_local_model_auto_computes_sha256() {
        let temp = tempdir().unwrap();
        let storage = StorageEngine::in_memory().unwrap();
        let privacy = Arc::new(PrivacyPolicyEngine::default());
        let registry = ModelRegistry::new(storage, privacy, temp.path()).unwrap();

        let host_model_file = temp.path().join("auto_hash.safetensors");
        let data = b"SAFETENSORS_HEADER_DATA_999";
        std::fs::write(&host_model_file, data).unwrap();

        let request = LocalModelImportRequest {
            source_path: host_model_file,
            model_id: "auto-hash-model".to_string(),
            name: "Auto Hash Model".to_string(),
            modality: Modality::Vision,
            version: "0.1.0".to_string(),
            description: "Auto hash test".to_string(),
            format: "safetensors".to_string(),
            parameter_count: None,
            quantization: None,
            min_ram_bytes: 256_000_000,
            recommended_backend: AcceleratorBackend::Cpu,
            expected_sha256: None, // Auto-compute hash
            expected_size_bytes: None,
            overwrite: false,
        };

        let result = registry
            .import_local_model(request, |_| {})
            .expect("import succeeds with auto hash");

        assert_eq!(result.sha256.len(), 64);
        assert!(registry
            .verify_cached_model_integrity("auto-hash-model")
            .unwrap());
    }

    #[test]
    fn test_import_local_model_tampered_weights_quarantined_and_deleted() {
        let temp = tempdir().unwrap();
        let storage = StorageEngine::in_memory().unwrap();
        let privacy = Arc::new(PrivacyPolicyEngine::default());
        let registry = ModelRegistry::new(storage, privacy, temp.path()).unwrap();

        let host_model_file = temp.path().join("tampered.gguf");
        std::fs::write(&host_model_file, b"MALICIOUS_TAMPERED_CONTENT").unwrap();

        let wrong_sha =
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string();

        let request = LocalModelImportRequest {
            source_path: host_model_file,
            model_id: "tampered-import".to_string(),
            name: "Tampered Model".to_string(),
            modality: Modality::Llm,
            version: "1.0.0".to_string(),
            description: "Tampered".to_string(),
            format: "gguf".to_string(),
            parameter_count: None,
            quantization: None,
            min_ram_bytes: 100_000,
            recommended_backend: AcceleratorBackend::Cpu,
            expected_sha256: Some(wrong_sha),
            expected_size_bytes: None,
            overwrite: false,
        };

        let err = registry
            .import_local_model(request, |_| {})
            .expect_err("tampered hash must fail");

        match err {
            RegistryError::Sha256Mismatch { model_id, .. } => {
                assert_eq!(model_id, "tampered-import");
            }
            other => panic!("Expected Sha256Mismatch, got {other:?}"),
        }

        // Verify quarantined staging file was deleted immediately
        let staging_file = registry
            .staging_dir()
            .join("tampered-import.import.quarantine");
        assert!(!staging_file.exists());

        // Verify no verified file was created
        let promoted_dir = registry.models_dir().join("tampered-import");
        assert!(!promoted_dir.exists());

        // Verify nothing in database
        assert!(registry.get_manifest("tampered-import").unwrap().is_none());
        assert!(registry
            .get_cached_model("tampered-import")
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_import_local_model_size_mismatch_rejected() {
        let temp = tempdir().unwrap();
        let storage = StorageEngine::in_memory().unwrap();
        let privacy = Arc::new(PrivacyPolicyEngine::default());
        let registry = ModelRegistry::new(storage, privacy, temp.path()).unwrap();

        let host_model_file = temp.path().join("size_test.bin");
        std::fs::write(&host_model_file, b"12345").unwrap();

        let request = LocalModelImportRequest {
            source_path: host_model_file,
            model_id: "size-mismatch-model".to_string(),
            name: "Size Test".to_string(),
            modality: Modality::SpeechAsr,
            version: "1.0.0".to_string(),
            description: "Size test".to_string(),
            format: "bin".to_string(),
            parameter_count: None,
            quantization: None,
            min_ram_bytes: 100_000,
            recommended_backend: AcceleratorBackend::Cpu,
            expected_sha256: None,
            expected_size_bytes: Some(9999), // Wrong size
            overwrite: false,
        };

        let err = registry
            .import_local_model(request, |_| {})
            .expect_err("size mismatch must fail");

        match err {
            RegistryError::SizeMismatch {
                expected, actual, ..
            } => {
                assert_eq!(expected, 9999);
                assert_eq!(actual, 5);
            }
            other => panic!("Expected SizeMismatch, got {other:?}"),
        }
    }

    #[test]
    fn test_import_local_model_collision_and_overwrite() {
        let temp = tempdir().unwrap();
        let storage = StorageEngine::in_memory().unwrap();
        let privacy = Arc::new(PrivacyPolicyEngine::default());
        let registry = ModelRegistry::new(storage, privacy, temp.path()).unwrap();

        let host_file_1 = temp.path().join("v1.gguf");
        std::fs::write(&host_file_1, b"VERSION_1_WEIGHTS").unwrap();

        let req1 = LocalModelImportRequest {
            source_path: host_file_1,
            model_id: "colliding-model".to_string(),
            name: "Colliding Model V1".to_string(),
            modality: Modality::Llm,
            version: "1.0.0".to_string(),
            description: "V1".to_string(),
            format: "gguf".to_string(),
            parameter_count: None,
            quantization: None,
            min_ram_bytes: 100_000,
            recommended_backend: AcceleratorBackend::Cpu,
            expected_sha256: None,
            expected_size_bytes: None,
            overwrite: false,
        };
        registry.import_local_model(req1, |_| {}).unwrap();

        // Attempt second import without overwrite
        let host_file_2 = temp.path().join("v2.gguf");
        std::fs::write(&host_file_2, b"VERSION_2_UPDATED_WEIGHTS").unwrap();

        let mut req2 = LocalModelImportRequest {
            source_path: host_file_2.clone(),
            model_id: "colliding-model".to_string(),
            name: "Colliding Model V2".to_string(),
            modality: Modality::Llm,
            version: "2.0.0".to_string(),
            description: "V2".to_string(),
            format: "gguf".to_string(),
            parameter_count: None,
            quantization: None,
            min_ram_bytes: 100_000,
            recommended_backend: AcceleratorBackend::Cpu,
            expected_sha256: None,
            expected_size_bytes: None,
            overwrite: false,
        };

        let err = registry
            .import_local_model(req2.clone(), |_| {})
            .expect_err("duplicate without overwrite must fail");
        match err {
            RegistryError::ModelAlreadyExists { model_id } => {
                assert_eq!(model_id, "colliding-model");
            }
            other => panic!("Expected ModelAlreadyExists, got {other:?}"),
        }

        // Now import with overwrite: true
        req2.overwrite = true;
        let res2 = registry
            .import_local_model(req2, |_| {})
            .expect("overwrite must succeed");
        assert_eq!(
            std::fs::read(&res2.verified_path).unwrap(),
            b"VERSION_2_UPDATED_WEIGHTS"
        );
        let manifest = registry.get_manifest("colliding-model").unwrap().unwrap();
        assert_eq!(manifest.version, "2.0.0");
    }

    #[test]
    fn test_import_local_model_path_traversal_and_invalid_params_rejected() {
        let temp = tempdir().unwrap();
        let storage = StorageEngine::in_memory().unwrap();
        let privacy = Arc::new(PrivacyPolicyEngine::default());
        let registry = ModelRegistry::new(storage, privacy, temp.path()).unwrap();

        let host_file = temp.path().join("safe.gguf");
        std::fs::write(&host_file, b"SAFE_DATA").unwrap();

        // 1. Path traversal in model_id
        let req_traversal = LocalModelImportRequest {
            source_path: host_file.clone(),
            model_id: "../../../evil-model".to_string(),
            name: "Evil Model".to_string(),
            modality: Modality::Llm,
            version: "1.0.0".to_string(),
            description: "Evil".to_string(),
            format: "gguf".to_string(),
            parameter_count: None,
            quantization: None,
            min_ram_bytes: 100_000,
            recommended_backend: AcceleratorBackend::Cpu,
            expected_sha256: None,
            expected_size_bytes: None,
            overwrite: false,
        };
        assert!(registry.import_local_model(req_traversal, |_| {}).is_err());

        // 2. Empty file rejected
        let empty_file = temp.path().join("empty.gguf");
        std::fs::write(&empty_file, b"").unwrap();
        let req_empty = LocalModelImportRequest {
            source_path: empty_file,
            model_id: "empty-model".to_string(),
            name: "Empty Model".to_string(),
            modality: Modality::Llm,
            version: "1.0.0".to_string(),
            description: "Empty".to_string(),
            format: "gguf".to_string(),
            parameter_count: None,
            quantization: None,
            min_ram_bytes: 100_000,
            recommended_backend: AcceleratorBackend::Cpu,
            expected_sha256: None,
            expected_size_bytes: None,
            overwrite: false,
        };
        assert!(registry.import_local_model(req_empty, |_| {}).is_err());

        // 3. Nonexistent file rejected
        let req_nonexistent = LocalModelImportRequest {
            source_path: temp.path().join("does_not_exist.bin"),
            model_id: "nonexistent-model".to_string(),
            name: "Nonexistent Model".to_string(),
            modality: Modality::Llm,
            version: "1.0.0".to_string(),
            description: "Nonexistent".to_string(),
            format: "bin".to_string(),
            parameter_count: None,
            quantization: None,
            min_ram_bytes: 100_000,
            recommended_backend: AcceleratorBackend::Cpu,
            expected_sha256: None,
            expected_size_bytes: None,
            overwrite: false,
        };
        assert!(registry
            .import_local_model(req_nonexistent, |_| {})
            .is_err());
    }

    #[test]
    fn test_import_local_model_allowed_in_local_only_privacy_mode() {
        let temp = tempdir().unwrap();
        let storage = StorageEngine::in_memory().unwrap();
        let mut privacy_engine = PrivacyPolicyEngine::default();
        privacy_engine.set_local_only_mode(true); // STRICT local-only privacy mode
        let privacy = Arc::new(privacy_engine);
        let registry = ModelRegistry::new(storage, privacy, temp.path()).unwrap();

        let host_file = temp.path().join("local_isolated.onnx");
        std::fs::write(&host_file, b"ISOLATED_LOCAL_ONLY_MODEL_DATA").unwrap();

        let req = LocalModelImportRequest {
            source_path: host_file,
            model_id: "isolated-local-onnx".to_string(),
            name: "Isolated Local ONNX".to_string(),
            modality: Modality::Vision,
            version: "1.0.0".to_string(),
            description: "Runs without network".to_string(),
            format: "onnx".to_string(),
            parameter_count: Some("25M".to_string()),
            quantization: Some("FP16".to_string()),
            min_ram_bytes: 100_000_000,
            recommended_backend: AcceleratorBackend::Cpu,
            expected_sha256: None,
            expected_size_bytes: None,
            overwrite: false,
        };

        // Local import must succeed completely even when local-only mode is active
        let result = registry
            .import_local_model(req, |_| {})
            .expect("local import must succeed in Local-Only Privacy Mode");

        assert!(result.verified_path.is_file());
        assert!(registry
            .verify_cached_model_integrity("isolated-local-onnx")
            .unwrap());
    }
}
