#![forbid(unsafe_code)]

use miniai_storage::{
    models::{AppSetting, ChatMessageRecord, ChatSessionRecord, MessageRole},
    StorageEngine,
};
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use tempfile::tempdir;

#[test]
fn test_zero_byte_database_initialization() {
    let dir = tempdir().expect("create temp dir");
    let db_path = dir.path().join("empty_zero_byte.db");

    // Create a 0-byte file
    {
        let _empty = File::create(&db_path).expect("create empty 0-byte file");
    }
    assert_eq!(std::fs::metadata(&db_path).unwrap().len(), 0);

    // Open storage engine on the 0-byte file
    let engine = StorageEngine::open(&db_path).expect("open empty database");
    let integrity = engine.check_integrity().expect("check integrity");
    assert!(
        integrity.is_healthy,
        "Zero-byte database must migrate to healthy state"
    );

    // Verify read/write functionality on newly initialized database
    let setting = AppSetting {
        key: "system.test_initialized".to_string(),
        value: "true".to_string(),
        updated_at_epoch_secs: 1234567,
    };
    engine.set_setting(&setting).expect("set setting");
    let retrieved = engine
        .get_setting("system.test_initialized")
        .expect("get setting")
        .expect("setting must exist");
    assert_eq!(retrieved.value, "true");
}

#[test]
fn test_unclosed_transaction_rollback_and_atomic_isolation() {
    let dir = tempdir().expect("create temp dir");
    let db_path = dir.path().join("transaction_rollback.db");
    let engine = StorageEngine::open(&db_path).expect("open database");

    // Insert baseline record
    let base_setting = AppSetting {
        key: "baseline_key".to_string(),
        value: "original_val".to_string(),
        updated_at_epoch_secs: 100,
    };
    engine.set_setting(&base_setting).expect("set base");

    // Execute transaction that fails midway
    let tx_res: Result<(), _> = engine.transaction(|tx| {
        tx.execute(
            "INSERT INTO app_settings (key, value, updated_at_epoch_secs) VALUES (?1, ?2, ?3)",
            rusqlite::params!["midway_key", "midway_val", 200i64],
        )?;

        // Return error simulating failure before commit
        Err(miniai_storage::StorageError::DatabaseError {
            message: "simulated abort".to_string(),
        })
    });
    assert!(tx_res.is_err(), "Transaction must fail and roll back");

    // Verify midway_key does NOT exist
    let midway = engine.get_setting("midway_key").expect("get setting");
    assert!(
        midway.is_none(),
        "Uncommitted transaction must be rolled back"
    );

    // Verify baseline key still exists and is untouched
    let base = engine
        .get_setting("baseline_key")
        .expect("get setting")
        .expect("base exists");
    assert_eq!(base.value, "original_val");

    // Verify subsequent transaction commits successfully
    let commit_res = engine.transaction(|tx| {
        tx.execute(
            "INSERT INTO app_settings (key, value, updated_at_epoch_secs) VALUES (?1, ?2, ?3)",
            rusqlite::params!["committed_key", "committed_val", 300i64],
        )?;
        Ok(())
    });
    assert!(commit_res.is_ok());

    let committed = engine
        .get_setting("committed_key")
        .expect("get setting")
        .expect("committed key exists");
    assert_eq!(committed.value, "committed_val");
}

#[test]
fn test_foreign_key_invariant_preservation() {
    let dir = tempdir().expect("create temp dir");
    let db_path = dir.path().join("foreign_key_invariants.db");
    let engine = StorageEngine::open(&db_path).expect("open database");

    // Attempt to insert a chat message referencing a non-existent session
    let orphan_msg = ChatMessageRecord {
        id: "msg-orphan".to_string(),
        session_id: "non-existent-session-id".to_string(),
        parent_id: None,
        role: MessageRole::User,
        content: "Orphan message".to_string(),
        token_count: 2,
        model_id: None,
        created_at_epoch_secs: 1000,
        updated_at_epoch_secs: 1000,
        is_pinned: false,
    };

    let err = engine.create_chat_message(&orphan_msg);
    assert!(
        err.is_err(),
        "Inserting orphan message without valid session must violate foreign key"
    );

    // Create valid session
    let session = ChatSessionRecord {
        id: "sess-valid-1".to_string(),
        title: "Session 1".to_string(),
        created_at_epoch_secs: 1000,
        updated_at_epoch_secs: 1000,
    };
    engine
        .create_chat_session(&session)
        .expect("create session");

    // Now insert message with valid session
    let valid_msg = ChatMessageRecord {
        id: "msg-child".to_string(),
        session_id: "sess-valid-1".to_string(),
        parent_id: None,
        role: MessageRole::User,
        content: "Valid child message".to_string(),
        token_count: 3,
        model_id: None,
        created_at_epoch_secs: 1001,
        updated_at_epoch_secs: 1001,
        is_pinned: false,
    };
    engine
        .create_chat_message(&valid_msg)
        .expect("create message");

    // Verify foreign key integrity check
    let integrity = engine.check_integrity().expect("check integrity");
    assert!(integrity.is_healthy);
    assert!(integrity.foreign_key_violations.is_empty());
}

#[test]
fn test_corrupt_database_quarantine_and_recovery() {
    let dir = tempdir().expect("create temp dir");
    let db_path = dir.path().join("corrupt_test.db");

    // Write garbage bytes simulating header / page corruption
    {
        let mut file = File::create(&db_path).expect("create file");
        file.write_all(b"NOT_A_VALID_SQLITE_DATABASE_HEADER_CORRUPTED_BYTES_1234567890")
            .expect("write corrupt bytes");
        file.flush().expect("flush");
    }

    // Quarantine and recover
    let (recovered_engine, quarantine_res) =
        StorageEngine::quarantine_and_recover(&db_path, 0).expect("quarantine and recover");

    assert!(quarantine_res.recreated_healthy);
    assert!(std::path::Path::new(&quarantine_res.quarantined_backup_path).exists());

    // Verify recovered engine is fully operational
    let integrity = recovered_engine.check_integrity().expect("integrity check");
    assert!(integrity.is_healthy);

    let setting = AppSetting {
        key: "quarantine_recovered".to_string(),
        value: "true".to_string(),
        updated_at_epoch_secs: 500,
    };
    recovered_engine.set_setting(&setting).expect("set setting");
    let res = recovered_engine
        .get_setting("quarantine_recovered")
        .expect("get")
        .unwrap();
    assert_eq!(res.value, "true");
}

#[test]
fn test_concurrent_read_write_stress() {
    let dir = tempdir().expect("create temp dir");
    let db_path = dir.path().join("concurrent_stress.db");
    let engine = Arc::new(StorageEngine::open(&db_path).expect("open database"));

    // Spawn 8 threads concurrently writing and reading
    let mut handles = Vec::new();
    for thread_idx in 0..8 {
        let engine_clone = Arc::clone(&engine);
        handles.push(std::thread::spawn(move || {
            for i in 0..25 {
                let key = format!("thread_{thread_idx}_key_{i}");
                let val = format!("val_{i}");
                let setting = AppSetting {
                    key: key.clone(),
                    value: val.clone(),
                    updated_at_epoch_secs: 1000 + i as u64,
                };
                engine_clone.set_setting(&setting).expect("set in thread");
                let read_back = engine_clone
                    .get_setting(&key)
                    .expect("read in thread")
                    .expect("must exist");
                assert_eq!(read_back.value, val);
            }
        }));
    }

    for h in handles {
        h.join().expect("thread join");
    }

    // Check integrity after concurrent multi-thread load
    let integrity = engine.check_integrity().expect("check integrity");
    assert!(integrity.is_healthy);
}
