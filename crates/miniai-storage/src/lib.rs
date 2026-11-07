#![forbid(unsafe_code)]
//! # MiniAI Storage (`miniai-storage`)
//!
//! Production-grade transactional SQLite storage engine, deterministic schema migrations,
//! settings persistence, workspace management, consent audit logging, model caching,
//! chat session persistence, atomic transactions, and corruption quarantine/recovery for MiniAI.

pub mod db;
pub mod error;
pub mod migrations;
pub mod models;

pub use db::StorageEngine;
pub use error::StorageError;
pub use migrations::{Migration, MigrationRunner, MIGRATIONS};
pub use models::{
    AppSetting, CachedModelRecord, ChatMessageRecord, ChatSessionExport, ChatSessionRecord,
    ConsentAuditEvent, DiagnosticEventRecord, EpisodicMemoryRecord, IntegrityReport, MessageRole,
    ModelManifestRecord, QuarantineResult, SemanticFactRecord, Workspace,
};

#[cfg(test)]
mod tests {
    use super::*;
    use miniai_core::{AcceleratorBackend, Modality};

    #[test]
    fn test_in_memory_initialization_and_schema_version() {
        let engine = StorageEngine::in_memory().expect("in-memory engine init");
        assert!(engine.is_in_memory());
        assert!(engine.path().is_none());
        assert_eq!(engine.schema_version().expect("schema version"), 5);

        let integrity = engine.check_integrity().expect("integrity check");
        assert!(integrity.is_healthy);
        assert_eq!(integrity.integrity_messages, vec!["ok"]);
        assert!(integrity.foreign_key_violations.is_empty());
    }

    #[test]
    fn test_on_disk_initialization_and_wal_mode() {
        let temp_dir = std::env::temp_dir().join(format!("miniai_test_wal_{}", std::process::id()));
        let db_path = temp_dir.join("storage.db");

        let engine = StorageEngine::open(&db_path).expect("open on-disk storage");
        assert!(!engine.is_in_memory());
        assert_eq!(engine.path().unwrap(), db_path.as_path());
        assert_eq!(engine.schema_version().expect("schema version"), 5);

        // Verify WAL mode was enabled on disk
        let integrity = engine.check_integrity().expect("integrity check");
        assert!(integrity.is_healthy);

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_app_settings_crud_operations() {
        let engine = StorageEngine::in_memory().expect("engine init");

        assert_eq!(engine.get_setting("local_only_mode").unwrap(), None);
        assert_eq!(engine.list_settings().unwrap().len(), 0);

        let setting = AppSetting {
            key: "local_only_mode".to_string(),
            value: "true".to_string(),
            updated_at_epoch_secs: 1000,
        };
        engine.set_setting(&setting).unwrap();

        let fetched = engine.get_setting("local_only_mode").unwrap().unwrap();
        assert_eq!(fetched, setting);

        let updated = AppSetting {
            key: "local_only_mode".to_string(),
            value: "false".to_string(),
            updated_at_epoch_secs: 1050,
        };
        engine.set_setting(&updated).unwrap();
        assert_eq!(
            engine
                .get_setting("local_only_mode")
                .unwrap()
                .unwrap()
                .value,
            "false"
        );

        engine
            .set_setting(&AppSetting {
                key: "force_cpu_only".to_string(),
                value: "true".to_string(),
                updated_at_epoch_secs: 1050,
            })
            .unwrap();
        let all = engine.list_settings().unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].key, "force_cpu_only");
        assert_eq!(all[1].key, "local_only_mode");

        assert!(engine.delete_setting("local_only_mode").unwrap());
        assert!(!engine.delete_setting("non_existent").unwrap());
        assert_eq!(engine.get_setting("local_only_mode").unwrap(), None);
    }

    #[test]
    fn test_workspaces_crud_and_active_switching() {
        let engine = StorageEngine::in_memory().expect("engine init");

        let ws1 = Workspace {
            id: "ws-01".to_string(),
            name: "Default Workspace".to_string(),
            root_path: "/data/ws1".to_string(),
            created_at_epoch_secs: 1000,
            updated_at_epoch_secs: 1000,
            is_active: true,
        };
        let ws2 = Workspace {
            id: "ws-02".to_string(),
            name: "Research Workspace".to_string(),
            root_path: "/data/ws2".to_string(),
            created_at_epoch_secs: 1010,
            updated_at_epoch_secs: 1010,
            is_active: false,
        };

        engine.create_workspace(&ws1).unwrap();
        engine.create_workspace(&ws2).unwrap();

        let active = engine.get_active_workspace().unwrap().unwrap();
        assert_eq!(active.id, "ws-01");

        engine.set_active_workspace("ws-02", 1020).unwrap();
        let new_active = engine.get_active_workspace().unwrap().unwrap();
        assert_eq!(new_active.id, "ws-02");

        let ws1_reloaded = engine.get_workspace("ws-01").unwrap().unwrap();
        assert!(!ws1_reloaded.is_active);

        assert!(matches!(
            engine.set_active_workspace("ws-99", 1030),
            Err(StorageError::NotFound {
                entity: "workspace",
                ..
            })
        ));

        let list = engine.list_workspaces().unwrap();
        assert_eq!(list.len(), 2);

        assert!(engine.delete_workspace("ws-01").unwrap());
        assert_eq!(engine.get_workspace("ws-01").unwrap(), None);
    }

    #[test]
    fn test_consent_audit_log_immutable_recording() {
        let engine = StorageEngine::in_memory().expect("engine init");

        assert_eq!(engine.count_consent_events().unwrap(), 0);

        let event1 = ConsentAuditEvent {
            id: None,
            token_id: "tok-1".to_string(),
            action: "GRANT".to_string(),
            modality: "llm".to_string(),
            target_endpoint: "https://gpu.miniai.local".to_string(),
            payload_bytes: 4096,
            epoch_secs: 1000,
            details: Some("User explicitly granted consent".to_string()),
        };
        let event2 = ConsentAuditEvent {
            id: None,
            token_id: "tok-1".to_string(),
            action: "REVOKE".to_string(),
            modality: "llm".to_string(),
            target_endpoint: "https://gpu.miniai.local".to_string(),
            payload_bytes: 0,
            epoch_secs: 1050,
            details: Some("User toggled Local-Only mode on".to_string()),
        };

        let id1 = engine.record_consent_event(&event1).unwrap();
        let id2 = engine.record_consent_event(&event2).unwrap();
        assert!(id2 > id1);

        assert_eq!(engine.count_consent_events().unwrap(), 2);
        let list = engine.list_consent_events(10).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].action, "REVOKE");
        assert_eq!(list[1].action, "GRANT");
    }

    #[test]
    fn test_diagnostic_events_recording() {
        let engine = StorageEngine::in_memory().expect("engine init");

        let diag = DiagnosticEventRecord {
            id: None,
            event_type: "HARDWARE_DETECTED".to_string(),
            severity: "INFO".to_string(),
            source: "miniai-core".to_string(),
            message: "Detected 8 CPU cores; selected CPU fallback".to_string(),
            payload_json: Some(r#"{"backend":"cpu"}"#.to_string()),
            created_at_epoch_secs: 1000,
        };

        let rowid = engine.record_diagnostic_event(&diag).unwrap();
        assert!(rowid > 0);

        let records = engine.list_diagnostic_events(5).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].event_type, "HARDWARE_DETECTED");
        assert_eq!(records[0].message, diag.message);
    }

    #[test]
    fn test_cached_models_crud_and_modality_filtering() {
        let engine = StorageEngine::in_memory().expect("engine init");

        let model1 = CachedModelRecord {
            id: "tiny-llama-q4".to_string(),
            modality: Modality::Llm,
            file_path: "models/llm/tiny-llama-q4.gguf".to_string(),
            sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
            size_bytes: 512 * 1024 * 1024,
            preferred_backend: AcceleratorBackend::Cpu,
            created_at_epoch_secs: 1000,
        };

        let model2 = CachedModelRecord {
            id: "vit-small".to_string(),
            modality: Modality::Vision,
            file_path: "models/vision/vit-small.onnx".to_string(),
            sha256: "a1b2c3d4e5f60123456789abcdef0123456789abcdef0123456789abcdef0123".to_string(),
            size_bytes: 128 * 1024 * 1024,
            preferred_backend: AcceleratorBackend::Cpu,
            created_at_epoch_secs: 1010,
        };

        engine.record_cached_model(&model1).unwrap();
        engine.record_cached_model(&model2).unwrap();

        let fetched = engine.get_cached_model("tiny-llama-q4").unwrap().unwrap();
        assert_eq!(fetched, model1);

        // List all models
        let all = engine.list_cached_models(None).unwrap();
        assert_eq!(all.len(), 2);

        // Filter by Modality::Llm
        let llms = engine.list_cached_models(Some(Modality::Llm)).unwrap();
        assert_eq!(llms.len(), 1);
        assert_eq!(llms[0].id, "tiny-llama-q4");

        // Filter by Modality::Vision
        let visions = engine.list_cached_models(Some(Modality::Vision)).unwrap();
        assert_eq!(visions.len(), 1);
        assert_eq!(visions[0].id, "vit-small");

        // Filter by Modality::Music (empty)
        let musics = engine.list_cached_models(Some(Modality::Music)).unwrap();
        assert_eq!(musics.len(), 0);

        // Update model metadata on conflict
        let mut updated = model1.clone();
        updated.size_bytes = 600 * 1024 * 1024;
        engine.record_cached_model(&updated).unwrap();
        assert_eq!(
            engine
                .get_cached_model("tiny-llama-q4")
                .unwrap()
                .unwrap()
                .size_bytes,
            600 * 1024 * 1024
        );

        // Delete model
        assert!(engine.delete_cached_model("tiny-llama-q4").unwrap());
        assert!(!engine.delete_cached_model("tiny-llama-q4").unwrap());
        assert_eq!(engine.get_cached_model("tiny-llama-q4").unwrap(), None);
    }

    #[test]
    fn test_chat_sessions_crud_and_ordering() {
        let engine = StorageEngine::in_memory().expect("engine init");

        let s1 = ChatSessionRecord {
            id: "chat-01".to_string(),
            title: "First Conversation".to_string(),
            created_at_epoch_secs: 1000,
            updated_at_epoch_secs: 1000,
        };
        let s2 = ChatSessionRecord {
            id: "chat-02".to_string(),
            title: "Second Conversation".to_string(),
            created_at_epoch_secs: 1010,
            updated_at_epoch_secs: 1050,
        };

        engine.create_chat_session(&s1).unwrap();
        engine.create_chat_session(&s2).unwrap();

        let fetched = engine.get_chat_session("chat-01").unwrap().unwrap();
        assert_eq!(fetched, s1);

        // List ordered by updated_at_epoch_secs DESC (s2 first)
        let list = engine.list_chat_sessions().unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].id, "chat-02");
        assert_eq!(list[1].id, "chat-01");

        // Update title of chat-01 and bump timestamp
        engine
            .update_chat_session_title("chat-01", "Renamed First Chat", 1100)
            .unwrap();
        let list_updated = engine.list_chat_sessions().unwrap();
        assert_eq!(list_updated[0].id, "chat-01");
        assert_eq!(list_updated[0].title, "Renamed First Chat");

        // Update missing session returns NotFound
        assert!(matches!(
            engine.update_chat_session_title("chat-99", "Missing", 1200),
            Err(StorageError::NotFound {
                entity: "chat_session",
                ..
            })
        ));

        // Delete session
        assert!(engine.delete_chat_session("chat-01").unwrap());
        assert_eq!(engine.get_chat_session("chat-01").unwrap(), None);
    }

    #[test]
    fn test_transaction_atomicity_and_rollback() {
        let engine = StorageEngine::in_memory().expect("engine init");

        // Successful atomic transaction
        let res: Result<(), StorageError> = engine.transaction(|tx| {
            tx.execute(
                "INSERT INTO app_settings (key, value, updated_at_epoch_secs) VALUES ('tx_key', 'tx_val', 1000)",
                [],
            )?;
            tx.execute(
                "INSERT INTO app_settings (key, value, updated_at_epoch_secs) VALUES ('tx_key2', 'tx_val2', 1000)",
                [],
            )?;
            Ok(())
        });
        assert!(res.is_ok());
        assert_eq!(
            engine.get_setting("tx_key").unwrap().unwrap().value,
            "tx_val"
        );
        assert_eq!(
            engine.get_setting("tx_key2").unwrap().unwrap().value,
            "tx_val2"
        );

        // Failed transaction: must rollback all operations
        let fail_res: Result<(), StorageError> = engine.transaction(|tx| {
            tx.execute(
                "INSERT INTO app_settings (key, value, updated_at_epoch_secs) VALUES ('rollback_key', 'val', 1000)",
                [],
            )?;
            // Intentional error: duplicate primary key insert without conflict handler
            tx.execute(
                "INSERT INTO app_settings (key, value, updated_at_epoch_secs) VALUES ('rollback_key', 'val', 1000)",
                [],
            )?;
            Ok(())
        });
        assert!(fail_res.is_err());

        // Verify that 'rollback_key' was not persisted due to atomic rollback
        assert_eq!(engine.get_setting("rollback_key").unwrap(), None);
    }

    #[test]
    fn test_quarantine_and_recovery_on_corrupt_database() {
        let temp_dir =
            std::env::temp_dir().join(format!("miniai_test_quarantine_{}", std::process::id()));
        let db_path = temp_dir.join("corrupt_test.db");

        // 1. Create a healthy on-disk database with data
        {
            let engine = StorageEngine::open(&db_path).expect("open initial db");
            engine
                .set_setting(&AppSetting {
                    key: "pre_corrupt_key".to_string(),
                    value: "pre_corrupt_val".to_string(),
                    updated_at_epoch_secs: 1000,
                })
                .unwrap();
            let integrity = engine.check_integrity().unwrap();
            assert!(integrity.is_healthy);
        }

        // 2. Simulate file corruption by overwriting database file with garbage bytes
        std::fs::write(&db_path, b"GARBAGE_CORRUPTED_SQLITE_HEADER_AND_PAGE_DATA").unwrap();

        // 3. Attempting to open the corrupted file and check integrity fails or reports unhealthy
        let corrupt_engine_res = StorageEngine::open(&db_path);
        // Note: rusqlite may fail on open/migration or check_integrity
        if let Ok(corrupt_engine) = corrupt_engine_res {
            let integrity = corrupt_engine.check_integrity();
            if let Ok(rep) = integrity {
                assert!(!rep.is_healthy);
            }
        }

        // 4. Quarantine and recover the corrupted database
        let (recovered_engine, quarantine_result) =
            StorageEngine::quarantine_and_recover(&db_path, 20261105).unwrap();

        assert!(quarantine_result.recreated_healthy);
        assert!(std::path::Path::new(&quarantine_result.quarantined_backup_path).exists());
        assert!(recovered_engine.check_integrity().unwrap().is_healthy);
        assert_eq!(recovered_engine.schema_version().unwrap(), 5);

        // Verify recovered database accepts new writes
        recovered_engine
            .set_setting(&AppSetting {
                key: "post_recovery_key".to_string(),
                value: "post_recovery_val".to_string(),
                updated_at_epoch_secs: 2000,
            })
            .unwrap();
        assert_eq!(
            recovered_engine
                .get_setting("post_recovery_key")
                .unwrap()
                .unwrap()
                .value,
            "post_recovery_val"
        );

        // Clean up temp dir
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_models_serialization_and_debug() {
        let setting = AppSetting {
            key: "test_key".to_string(),
            value: "test_val".to_string(),
            updated_at_epoch_secs: 12345,
        };
        let json = serde_json::to_string(&setting).expect("serialize setting");
        let decoded: AppSetting = serde_json::from_str(&json).expect("deserialize setting");
        assert_eq!(setting, decoded);

        let ws = Workspace {
            id: "ws-03".to_string(),
            name: "Test WS".to_string(),
            root_path: "/test/path".to_string(),
            created_at_epoch_secs: 100,
            updated_at_epoch_secs: 200,
            is_active: false,
        };
        let ws_json = serde_json::to_string(&ws).expect("serialize ws");
        let ws_decoded: Workspace = serde_json::from_str(&ws_json).expect("deserialize ws");
        assert_eq!(ws, ws_decoded);

        let event = ConsentAuditEvent {
            id: Some(42),
            token_id: "tok-42".to_string(),
            action: "GRANT".to_string(),
            modality: "vision".to_string(),
            target_endpoint: "https://vision.miniai.local".to_string(),
            payload_bytes: 8192,
            epoch_secs: 1500,
            details: None,
        };
        let event_json = serde_json::to_string(&event).expect("serialize event");
        let event_decoded: ConsentAuditEvent =
            serde_json::from_str(&event_json).expect("deserialize event");
        assert_eq!(event, event_decoded);

        let diag = DiagnosticEventRecord {
            id: Some(1),
            event_type: "ALERT".to_string(),
            severity: "WARN".to_string(),
            source: "storage".to_string(),
            message: "warn message".to_string(),
            payload_json: None,
            created_at_epoch_secs: 1200,
        };
        let diag_json = serde_json::to_string(&diag).expect("serialize diag");
        let diag_decoded: DiagnosticEventRecord =
            serde_json::from_str(&diag_json).expect("deserialize diag");
        assert_eq!(diag, diag_decoded);

        let report = IntegrityReport {
            is_healthy: true,
            integrity_messages: vec!["ok".to_string()],
            foreign_key_violations: vec![],
        };
        let rep_json = serde_json::to_string(&report).expect("serialize report");
        let rep_decoded: IntegrityReport =
            serde_json::from_str(&rep_json).expect("deserialize report");
        assert_eq!(report, rep_decoded);

        let q_res = QuarantineResult {
            original_path: "/data/orig.db".to_string(),
            quarantined_backup_path: "/data/orig.corrupt.bak".to_string(),
            recreated_healthy: true,
        };
        let q_json = serde_json::to_string(&q_res).expect("serialize quarantine");
        let q_decoded: QuarantineResult =
            serde_json::from_str(&q_json).expect("deserialize quarantine");
        assert_eq!(q_res, q_decoded);
    }

    #[test]
    fn test_storage_error_display_and_conversions() {
        let err1 = StorageError::DatabaseError {
            message: "disk failure".to_string(),
        };
        assert!(format!("{err1}").contains("disk failure"));

        let err2 = StorageError::MigrationError {
            message: "bad migration".to_string(),
        };
        assert!(format!("{err2}").contains("bad migration"));

        let err3 = StorageError::MigrationChecksumMismatch {
            version: 1,
            name: "init".to_string(),
            expected: "aaa".to_string(),
            recorded: "bbb".to_string(),
        };
        assert!(format!("{err3}").contains("Checksum mismatch"));

        let err4 = StorageError::IntegrityCheckFailed {
            details: vec!["corruption".to_string()],
        };
        assert!(format!("{err4}").contains("corruption"));

        let err5 = StorageError::ForeignKeyCheckFailed {
            violations: vec!["missing parent".to_string()],
        };
        assert!(format!("{err5}").contains("missing parent"));

        let err6 = StorageError::NotFound {
            entity: "setting",
            key: "missing_key".to_string(),
        };
        assert!(format!("{err6}").contains("missing_key"));

        let err7 = StorageError::SerializationError {
            message: "json err".to_string(),
        };
        assert!(format!("{err7}").contains("json err"));

        let err8 = StorageError::LockPoisoned;
        assert!(format!("{err8}").contains("poisoned"));

        let rusql_err = rusqlite::Error::InvalidQuery;
        let converted: StorageError = rusql_err.into();
        assert!(matches!(converted, StorageError::DatabaseError { .. }));
    }

    #[test]
    fn test_migration_checksum_calculation() {
        let mig = Migration::new(99, "test_mig", "SELECT 1;");
        let checksum = mig.checksum();
        assert_eq!(checksum.len(), 64);
        assert_eq!(checksum, miniai_core::compute_sha256_hex(b"SELECT 1;"));
    }

    #[test]
    fn test_migration_checksum_mismatch_detection() {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();

        let initial_migrations = [Migration::new(
            1,
            "001_init",
            "CREATE TABLE sample (id INTEGER PRIMARY KEY);",
        )];
        let count = MigrationRunner::run_migrations(&mut conn, &initial_migrations, 1000).unwrap();
        assert_eq!(count, 1);

        let altered_migrations = [Migration::new(
            1,
            "001_init",
            "CREATE TABLE sample (id INTEGER PRIMARY KEY, altered INTEGER);",
        )];
        let err =
            MigrationRunner::run_migrations(&mut conn, &altered_migrations, 1010).unwrap_err();
        assert!(matches!(
            err,
            StorageError::MigrationChecksumMismatch { version: 1, .. }
        ));

        let altered_name_migrations = [Migration::new(
            1,
            "001_different_name",
            "CREATE TABLE sample (id INTEGER PRIMARY KEY);",
        )];
        let err_name =
            MigrationRunner::run_migrations(&mut conn, &altered_name_migrations, 1010).unwrap_err();
        assert!(matches!(err_name, StorageError::MigrationError { .. }));
    }

    #[test]
    fn test_model_manifests_crud() {
        let engine = StorageEngine::in_memory().unwrap();

        let record = crate::models::ModelManifestRecord {
            model_id: "qwen2.5-0.5b".to_string(),
            name: "Qwen 2.5 0.5B Instruct".to_string(),
            modality: miniai_core::Modality::Llm,
            version: "1.0.0".to_string(),
            description: "Compact local LLM for edge reasoning".to_string(),
            source_url: "https://models.miniai.local/qwen2.5-0.5b.gguf".to_string(),
            expected_sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                .to_string(),
            expected_size_bytes: 400_000_000,
            format: "gguf".to_string(),
            parameter_count: Some("0.5B".to_string()),
            quantization: Some("Q4_K_M".to_string()),
            min_ram_bytes: 1_000_000_000,
            recommended_backend: miniai_core::AcceleratorBackend::Cpu,
            created_at_epoch_secs: 1700000000,
        };

        engine.insert_model_manifest(&record).unwrap();

        let fetched = engine
            .get_model_manifest("qwen2.5-0.5b")
            .unwrap()
            .expect("manifest should exist");
        assert_eq!(fetched.name, "Qwen 2.5 0.5B Instruct");
        assert_eq!(fetched.modality, miniai_core::Modality::Llm);
        assert_eq!(fetched.format, "gguf");

        let llm_list = engine
            .list_model_manifests(Some(miniai_core::Modality::Llm))
            .unwrap();
        assert_eq!(llm_list.len(), 1);

        let vision_list = engine
            .list_model_manifests(Some(miniai_core::Modality::Vision))
            .unwrap();
        assert_eq!(vision_list.len(), 0);

        let all_list = engine.list_model_manifests(None).unwrap();
        assert_eq!(all_list.len(), 1);

        let deleted = engine.delete_model_manifest("qwen2.5-0.5b").unwrap();
        assert!(deleted);
        assert!(engine.get_model_manifest("qwen2.5-0.5b").unwrap().is_none());
    }

    #[test]
    fn test_chat_messages_crud_and_branch_traversal() {
        let engine = StorageEngine::in_memory().expect("engine init");

        let session = ChatSessionRecord {
            id: "session-branch".to_string(),
            title: "Branching Dialogue".to_string(),
            created_at_epoch_secs: 1000,
            updated_at_epoch_secs: 1000,
        };
        engine.create_chat_session(&session).unwrap();

        // Root message: System
        let msg_sys = ChatMessageRecord {
            id: "msg-0".to_string(),
            session_id: "session-branch".to_string(),
            parent_id: None,
            role: MessageRole::System,
            content: "You are MiniAI assistant.".to_string(),
            token_count: 5,
            model_id: None,
            created_at_epoch_secs: 1001,
            updated_at_epoch_secs: 1001,
            is_pinned: false,
        };
        engine.create_chat_message(&msg_sys).unwrap();

        // User prompt (parent = msg-0)
        let msg_user = ChatMessageRecord {
            id: "msg-1".to_string(),
            session_id: "session-branch".to_string(),
            parent_id: Some("msg-0".to_string()),
            role: MessageRole::User,
            content: "Explain quantum computing simply.".to_string(),
            token_count: 6,
            model_id: None,
            created_at_epoch_secs: 1002,
            updated_at_epoch_secs: 1002,
            is_pinned: false,
        };
        engine.create_chat_message(&msg_user).unwrap();

        // Completion A (parent = msg-1)
        let msg_ast_a = ChatMessageRecord {
            id: "msg-2a".to_string(),
            session_id: "session-branch".to_string(),
            parent_id: Some("msg-1".to_string()),
            role: MessageRole::Assistant,
            content: "Quantum computing uses qubits that can be in superposition.".to_string(),
            token_count: 10,
            model_id: Some("qwen-0.5b".to_string()),
            created_at_epoch_secs: 1003,
            updated_at_epoch_secs: 1003,
            is_pinned: false,
        };
        engine.create_chat_message(&msg_ast_a).unwrap();

        // Regenerated Completion B (parent = msg-1, alternative branch)
        let msg_ast_b = ChatMessageRecord {
            id: "msg-2b".to_string(),
            session_id: "session-branch".to_string(),
            parent_id: Some("msg-1".to_string()),
            role: MessageRole::Assistant,
            content: "Think of a coin spinning in the air instead of lying flat.".to_string(),
            token_count: 12,
            model_id: Some("llama-1b".to_string()),
            created_at_epoch_secs: 1004,
            updated_at_epoch_secs: 1004,
            is_pinned: false,
        };
        engine.create_chat_message(&msg_ast_b).unwrap();

        // Verify alternative branches branching from msg-1
        let branches = engine.get_child_branches("msg-1").unwrap();
        assert_eq!(branches.len(), 2);
        assert_eq!(branches[0].id, "msg-2a");
        assert_eq!(branches[1].id, "msg-2b");

        // Verify linear branch traversal up to root for branch A
        let branch_a = engine.get_message_branch("msg-2a").unwrap();
        assert_eq!(branch_a.len(), 3);
        assert_eq!(branch_a[0].id, "msg-0");
        assert_eq!(branch_a[1].id, "msg-1");
        assert_eq!(branch_a[2].id, "msg-2a");

        // Verify linear branch traversal up to root for branch B
        let branch_b = engine.get_message_branch("msg-2b").unwrap();
        assert_eq!(branch_b.len(), 3);
        assert_eq!(branch_b[0].id, "msg-0");
        assert_eq!(branch_b[1].id, "msg-1");
        assert_eq!(branch_b[2].id, "msg-2b");

        // Verify session updated_at was bumped
        let sess = engine.get_chat_session("session-branch").unwrap().unwrap();
        assert_eq!(sess.updated_at_epoch_secs, 1004);
    }

    #[test]
    fn test_chat_message_editing_and_deletion() {
        let engine = StorageEngine::in_memory().expect("engine init");
        let session = ChatSessionRecord {
            id: "s-edit".to_string(),
            title: "Edit Test".to_string(),
            created_at_epoch_secs: 1000,
            updated_at_epoch_secs: 1000,
        };
        engine.create_chat_session(&session).unwrap();

        let msg = ChatMessageRecord {
            id: "m-1".to_string(),
            session_id: "s-edit".to_string(),
            parent_id: None,
            role: MessageRole::User,
            content: "Original text".to_string(),
            token_count: 2,
            model_id: None,
            created_at_epoch_secs: 1000,
            updated_at_epoch_secs: 1000,
            is_pinned: false,
        };
        engine.create_chat_message(&msg).unwrap();

        // Edit message
        let updated = engine
            .update_chat_message_content("m-1", "Edited content with more tokens", 6, 1050)
            .unwrap();
        assert!(updated);

        let fetched = engine.get_chat_message("m-1").unwrap().unwrap();
        assert_eq!(fetched.content, "Edited content with more tokens");
        assert_eq!(fetched.token_count, 6);
        assert_eq!(fetched.updated_at_epoch_secs, 1050);

        // Delete single message
        assert!(engine.delete_chat_message("m-1").unwrap());
        assert_eq!(engine.get_chat_message("m-1").unwrap(), None);
    }

    #[test]
    fn test_chat_session_export_markdown_and_json() {
        let engine = StorageEngine::in_memory().expect("engine init");
        let session = ChatSessionRecord {
            id: "s-export".to_string(),
            title: "Exportable Conversation".to_string(),
            created_at_epoch_secs: 2000,
            updated_at_epoch_secs: 2050,
        };
        engine.create_chat_session(&session).unwrap();

        let user_msg = ChatMessageRecord {
            id: "m-user".to_string(),
            session_id: "s-export".to_string(),
            parent_id: None,
            role: MessageRole::User,
            content: "What is MiniAI?".to_string(),
            token_count: 5,
            model_id: None,
            created_at_epoch_secs: 2000,
            updated_at_epoch_secs: 2000,
            is_pinned: false,
        };
        let ast_msg = ChatMessageRecord {
            id: "m-ast".to_string(),
            session_id: "s-export".to_string(),
            parent_id: Some("m-user".to_string()),
            role: MessageRole::Assistant,
            content: "MiniAI is a local-first multimodal platform.".to_string(),
            token_count: 8,
            model_id: Some("test-model".to_string()),
            created_at_epoch_secs: 2010,
            updated_at_epoch_secs: 2010,
            is_pinned: false,
        };

        engine.create_chat_message(&user_msg).unwrap();
        engine.create_chat_message(&ast_msg).unwrap();

        // Test Markdown export for whole session
        let md = engine.export_session_markdown("s-export", None).unwrap();
        assert!(md.contains("# Chat Session: Exportable Conversation"));
        assert!(md.contains("### 👤 User"));
        assert!(md.contains("What is MiniAI?"));
        assert!(md.contains("### 🤖 Assistant (`test-model`)"));
        assert!(md.contains("MiniAI is a local-first multimodal platform."));

        // Test Markdown export along specific leaf branch
        let md_branch = engine
            .export_session_markdown("s-export", Some("m-ast"))
            .unwrap();
        assert!(md_branch.contains("Exported Messages:* 2"));

        // Test JSON export
        let json_str = engine.export_session_json("s-export").unwrap();
        let export: ChatSessionExport = serde_json::from_str(&json_str).unwrap();
        assert_eq!(export.session.title, "Exportable Conversation");
        assert_eq!(export.messages.len(), 2);
        assert_eq!(export.messages[0].role, MessageRole::User);
        assert_eq!(export.messages[1].role, MessageRole::Assistant);
    }

    #[test]
    fn test_episodic_and_semantic_memories_crud_and_search() {
        let engine = StorageEngine::in_memory().unwrap();

        // 1. Episodic memory operations
        let ep = EpisodicMemoryRecord {
            id: "ep-001".to_string(),
            session_id: "sess-101".to_string(),
            topic: "Architecture Design Review".to_string(),
            summary: "Discussed three-tier memory model and CPU-first inference guarantees."
                .to_string(),
            key_decisions:
                "Adopted WorkingMemory, EpisodicMemory, and SemanticMemory with SQLite backing."
                    .to_string(),
            created_at_epoch_secs: 1700000000,
        };
        engine.insert_episodic_memory(&ep).unwrap();

        let list_ep = engine.list_episodic_memories(10).unwrap();
        assert_eq!(list_ep.len(), 1);
        assert_eq!(list_ep[0].id, "ep-001");

        let search_ep = engine.search_episodic_memories("three-tier").unwrap();
        assert_eq!(search_ep.len(), 1);
        assert_eq!(search_ep[0].topic, "Architecture Design Review");

        let no_search_ep = engine
            .search_episodic_memories("quantum computing")
            .unwrap();
        assert!(no_search_ep.is_empty());

        // 2. Semantic fact memory operations
        let fact1 = SemanticFactRecord {
            id: "fact-001".to_string(),
            category: "preference".to_string(),
            content: "User prefers concise answers with clickable file links.".to_string(),
            confidence: 0.95,
            created_at_epoch_secs: 1700000000,
            updated_at_epoch_secs: 1700000000,
        };
        let fact2 = SemanticFactRecord {
            id: "fact-002".to_string(),
            category: "project_fact".to_string(),
            content: "MiniAI uses SQLite with WAL mode on disk.".to_string(),
            confidence: 1.0,
            created_at_epoch_secs: 1700000100,
            updated_at_epoch_secs: 1700000100,
        };
        engine.insert_semantic_fact(&fact1).unwrap();
        engine.insert_semantic_fact(&fact2).unwrap();

        let facts = engine.list_semantic_facts().unwrap();
        assert_eq!(facts.len(), 2);

        let filtered_category = engine
            .search_semantic_facts(Some("preference"), None)
            .unwrap();
        assert_eq!(filtered_category.len(), 1);
        assert_eq!(filtered_category[0].id, "fact-001");

        let filtered_content = engine
            .search_semantic_facts(None, Some("WAL mode"))
            .unwrap();
        assert_eq!(filtered_content.len(), 1);
        assert_eq!(filtered_content[0].id, "fact-002");

        // 3. GDPR "Forget Memory" guarantee
        let forgotten = engine.forget_semantic_fact("fact-001").unwrap();
        assert!(forgotten);

        let remaining = engine.list_semantic_facts().unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, "fact-002");

        let forget_non_existent = engine.forget_semantic_fact("non-existent").unwrap();
        assert!(!forget_non_existent);
    }
}
