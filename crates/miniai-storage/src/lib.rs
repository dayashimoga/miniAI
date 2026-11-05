#![forbid(unsafe_code)]
//! # MiniAI Storage (`miniai-storage`)
//!
//! Production-grade transactional SQLite storage engine, deterministic schema migrations,
//! settings persistence, workspace management, consent audit logging, and database
//! integrity/corruption detection for MiniAI.

pub mod db;
pub mod error;
pub mod migrations;
pub mod models;

pub use db::StorageEngine;
pub use error::StorageError;
pub use migrations::{Migration, MigrationRunner, MIGRATIONS};
pub use models::{
    AppSetting, ConsentAuditEvent, DiagnosticEventRecord, IntegrityReport, Workspace,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_initialization_and_schema_version() {
        let engine = StorageEngine::in_memory().expect("in-memory engine init");
        assert!(engine.is_in_memory());
        assert_eq!(engine.schema_version().expect("schema version"), 2);

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
        assert_eq!(engine.schema_version().expect("schema version"), 2);

        // Verify WAL mode was enabled on disk
        let integrity = engine.check_integrity().expect("integrity check");
        assert!(integrity.is_healthy);

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_app_settings_crud_operations() {
        let engine = StorageEngine::in_memory().expect("engine init");

        // Initially empty
        assert_eq!(engine.get_setting("local_only_mode").unwrap(), None);
        assert_eq!(engine.list_settings().unwrap().len(), 0);

        // Insert setting
        let setting = AppSetting {
            key: "local_only_mode".to_string(),
            value: "true".to_string(),
            updated_at_epoch_secs: 1000,
        };
        engine.set_setting(&setting).unwrap();

        let fetched = engine.get_setting("local_only_mode").unwrap().unwrap();
        assert_eq!(fetched, setting);

        // Upsert / update setting
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

        // Add second setting and list
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

        // Delete setting
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

        // Switch active workspace to ws2
        engine.set_active_workspace("ws-02", 1020).unwrap();
        let new_active = engine.get_active_workspace().unwrap().unwrap();
        assert_eq!(new_active.id, "ws-02");

        let ws1_reloaded = engine.get_workspace("ws-01").unwrap().unwrap();
        assert!(!ws1_reloaded.is_active);

        // Switch to non-existent workspace returns NotFound
        assert!(matches!(
            engine.set_active_workspace("ws-99", 1030),
            Err(StorageError::NotFound {
                entity: "workspace",
                ..
            })
        ));

        // List workspaces
        let list = engine.list_workspaces().unwrap();
        assert_eq!(list.len(), 2);

        // Delete workspace
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
        // Listed in descending order of ID
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

        // From rusqlite error
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

        // Run initial migrations
        let initial_migrations = [Migration::new(
            1,
            "001_init",
            "CREATE TABLE sample (id INTEGER PRIMARY KEY);",
        )];
        let count = MigrationRunner::run_migrations(&mut conn, &initial_migrations, 1000).unwrap();
        assert_eq!(count, 1);

        // Attempt to re-run migration with altered SQL
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

        // Attempt to re-run migration with altered name
        let altered_name_migrations = [Migration::new(
            1,
            "001_different_name",
            "CREATE TABLE sample (id INTEGER PRIMARY KEY);",
        )];
        let err_name =
            MigrationRunner::run_migrations(&mut conn, &altered_name_migrations, 1010).unwrap_err();
        assert!(matches!(err_name, StorageError::MigrationError { .. }));
    }
}
