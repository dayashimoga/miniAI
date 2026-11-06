use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use miniai_core::{AcceleratorBackend, Modality};
use rusqlite::{params, Connection, OpenFlags};

use crate::error::StorageError;
use crate::migrations::{MigrationRunner, MIGRATIONS};
use crate::models::{
    AppSetting, CachedModelRecord, ChatMessageRecord, ChatSessionExport, ChatSessionRecord,
    ConsentAuditEvent, DiagnosticEventRecord, IntegrityReport, MessageRole, ModelManifestRecord,
    QuarantineResult, Workspace,
};

/// A thread-safe, transactional SQLite storage engine with WAL mode and migration support.
#[derive(Clone)]
pub struct StorageEngine {
    conn: Arc<Mutex<Connection>>,
    is_in_memory: bool,
    path: Option<PathBuf>,
}

impl StorageEngine {
    /// Opens an in-memory SQLite database and applies baseline migrations.
    pub fn in_memory() -> Result<Self, StorageError> {
        let mut conn = Connection::open_in_memory()?;
        Self::configure_pragmas(&mut conn, true)?;
        MigrationRunner::run_migrations(&mut conn, MIGRATIONS, 0)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            is_in_memory: true,
            path: None,
        })
    }

    /// Opens an on-disk SQLite database, enables WAL mode, and applies baseline migrations.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let path_ref = path.as_ref();
        if let Some(parent) = path_ref.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|e| StorageError::DatabaseError {
                    message: format!("Failed to create database directory: {e}"),
                })?;
            }
        }

        let mut conn = Connection::open_with_flags(
            path_ref,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;

        Self::configure_pragmas(&mut conn, false)?;
        MigrationRunner::run_migrations(&mut conn, MIGRATIONS, 0)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            is_in_memory: false,
            path: Some(path_ref.to_path_buf()),
        })
    }

    /// Configures essential pragmas for WAL mode, busy timeout, and foreign key enforcement.
    fn configure_pragmas(conn: &mut Connection, in_memory: bool) -> Result<(), StorageError> {
        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA busy_timeout = 5000;")?;
        if !in_memory {
            conn.execute_batch(
                "PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL; PRAGMA cache_size = -64000;",
            )?;
        }
        Ok(())
    }

    /// Returns true if this engine is running in memory.
    pub fn is_in_memory(&self) -> bool {
        self.is_in_memory
    }

    /// Returns the filesystem path of this storage engine if it is backed by an on-disk file.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Acquires the connection mutex.
    fn lock_conn(&self) -> Result<std::sync::MutexGuard<'_, Connection>, StorageError> {
        self.conn.lock().map_err(|_| StorageError::LockPoisoned)
    }

    /// Executes a closure inside an atomic database transaction.
    /// Automatically commits if the closure returns `Ok(_)`, or rolls back on `Err(_)`.
    pub fn transaction<F, R>(&self, f: F) -> Result<R, StorageError>
    where
        F: FnOnce(&rusqlite::Transaction<'_>) -> Result<R, StorageError>,
    {
        let mut conn = self.lock_conn()?;
        let tx = conn.transaction()?;
        let result = f(&tx)?;
        tx.commit()?;
        Ok(result)
    }

    /// Performs `PRAGMA integrity_check` and `PRAGMA foreign_key_check`.
    pub fn check_integrity(&self) -> Result<IntegrityReport, StorageError> {
        let conn = self.lock_conn()?;

        let mut integrity_messages = Vec::new();
        let mut integrity_stmt = conn.prepare("PRAGMA integrity_check")?;
        let rows = integrity_stmt.query_map([], |row| row.get::<_, String>(0))?;
        for r in rows {
            integrity_messages.push(r?);
        }

        let mut foreign_key_violations = Vec::new();
        let mut fk_stmt = conn.prepare("PRAGMA foreign_key_check")?;
        let fk_rows = fk_stmt.query_map([], |row| {
            let table: String = row.get(0)?;
            let rowid: i64 = row.get(1)?;
            let parent: String = row.get(2)?;
            let fkid: i64 = row.get(3)?;
            Ok(format!(
                "Table '{table}' row {rowid} references missing parent '{parent}' (fkid {fkid})"
            ))
        })?;
        for r in fk_rows {
            foreign_key_violations.push(r?);
        }

        let is_healthy = integrity_messages.len() == 1
            && integrity_messages[0] == "ok"
            && foreign_key_violations.is_empty();

        Ok(IntegrityReport {
            is_healthy,
            integrity_messages,
            foreign_key_violations,
        })
    }

    /// Quarantines an existing corrupted database by renaming the file (and any associated
    /// `-wal` and `-shm` journal files) to a timestamped backup path, and initializes a clean,
    /// healthy database at the original path.
    pub fn quarantine_and_recover<P: AsRef<Path>>(
        path: P,
        timestamp_epoch_secs: u64,
    ) -> Result<(Self, QuarantineResult), StorageError> {
        let path_ref = path.as_ref();
        let backup_path = path_ref.with_extension(format!("{timestamp_epoch_secs}.corrupt.bak"));

        if path_ref.exists() {
            std::fs::rename(path_ref, &backup_path).map_err(|e| StorageError::DatabaseError {
                message: format!(
                    "Failed to quarantine corrupt database from '{}' to '{}': {e}",
                    path_ref.display(),
                    backup_path.display()
                ),
            })?;

            let wal_path = PathBuf::from(format!("{}-wal", path_ref.display()));
            if wal_path.exists() {
                let _ = std::fs::rename(
                    &wal_path,
                    PathBuf::from(format!("{}-wal", backup_path.display())),
                );
            }
            let shm_path = PathBuf::from(format!("{}-shm", path_ref.display()));
            if shm_path.exists() {
                let _ = std::fs::rename(
                    &shm_path,
                    PathBuf::from(format!("{}-shm", backup_path.display())),
                );
            }
        }

        let new_engine = Self::open(path_ref)?;
        let integrity = new_engine.check_integrity()?;

        Ok((
            new_engine,
            QuarantineResult {
                original_path: path_ref.to_string_lossy().to_string(),
                quarantined_backup_path: backup_path.to_string_lossy().to_string(),
                recreated_healthy: integrity.is_healthy,
            },
        ))
    }

    /// Returns the current schema version recorded in `schema_migrations`.
    pub fn schema_version(&self) -> Result<u32, StorageError> {
        let conn = self.lock_conn()?;
        MigrationRunner::current_version(&conn)
    }

    // =========================================================================
    // App Settings Repository
    // =========================================================================

    pub fn get_setting(&self, key: &str) -> Result<Option<AppSetting>, StorageError> {
        let conn = self.lock_conn()?;
        let mut stmt = conn
            .prepare("SELECT key, value, updated_at_epoch_secs FROM app_settings WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(AppSetting {
                key: row.get(0)?,
                value: row.get(1)?,
                updated_at_epoch_secs: row.get(2)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn set_setting(&self, setting: &AppSetting) -> Result<(), StorageError> {
        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT INTO app_settings (key, value, updated_at_epoch_secs) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at_epoch_secs = excluded.updated_at_epoch_secs",
            params![setting.key, setting.value, setting.updated_at_epoch_secs],
        )?;
        Ok(())
    }

    pub fn delete_setting(&self, key: &str) -> Result<bool, StorageError> {
        let conn = self.lock_conn()?;
        let rows_affected =
            conn.execute("DELETE FROM app_settings WHERE key = ?1", params![key])?;
        Ok(rows_affected > 0)
    }

    pub fn list_settings(&self) -> Result<Vec<AppSetting>, StorageError> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT key, value, updated_at_epoch_secs FROM app_settings ORDER BY key ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(AppSetting {
                key: row.get(0)?,
                value: row.get(1)?,
                updated_at_epoch_secs: row.get(2)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    // =========================================================================
    // Workspaces Repository
    // =========================================================================

    pub fn create_workspace(&self, ws: &Workspace) -> Result<(), StorageError> {
        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT INTO workspaces (id, name, root_path, created_at_epoch_secs, updated_at_epoch_secs, is_active)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                ws.id,
                ws.name,
                ws.root_path,
                ws.created_at_epoch_secs,
                ws.updated_at_epoch_secs,
                if ws.is_active { 1 } else { 0 },
            ],
        )?;
        Ok(())
    }

    pub fn get_workspace(&self, id: &str) -> Result<Option<Workspace>, StorageError> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, root_path, created_at_epoch_secs, updated_at_epoch_secs, is_active FROM workspaces WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Workspace {
                id: row.get(0)?,
                name: row.get(1)?,
                root_path: row.get(2)?,
                created_at_epoch_secs: row.get(3)?,
                updated_at_epoch_secs: row.get(4)?,
                is_active: row.get::<_, i64>(5)? != 0,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_active_workspace(&self) -> Result<Option<Workspace>, StorageError> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, root_path, created_at_epoch_secs, updated_at_epoch_secs, is_active FROM workspaces WHERE is_active = 1 LIMIT 1",
        )?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Workspace {
                id: row.get(0)?,
                name: row.get(1)?,
                root_path: row.get(2)?,
                created_at_epoch_secs: row.get(3)?,
                updated_at_epoch_secs: row.get(4)?,
                is_active: true,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn set_active_workspace(
        &self,
        id: &str,
        updated_at_epoch_secs: u64,
    ) -> Result<(), StorageError> {
        let mut conn = self.lock_conn()?;
        let tx = conn.transaction()?;
        tx.execute(
            "UPDATE workspaces SET is_active = 0 WHERE is_active = 1",
            [],
        )?;
        let rows = tx.execute(
            "UPDATE workspaces SET is_active = 1, updated_at_epoch_secs = ?2 WHERE id = ?1",
            params![id, updated_at_epoch_secs],
        )?;
        if rows == 0 {
            return Err(StorageError::NotFound {
                entity: "workspace",
                key: id.to_string(),
            });
        }
        tx.commit()?;
        Ok(())
    }

    pub fn list_workspaces(&self) -> Result<Vec<Workspace>, StorageError> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, root_path, created_at_epoch_secs, updated_at_epoch_secs, is_active FROM workspaces ORDER BY name ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Workspace {
                id: row.get(0)?,
                name: row.get(1)?,
                root_path: row.get(2)?,
                created_at_epoch_secs: row.get(3)?,
                updated_at_epoch_secs: row.get(4)?,
                is_active: row.get::<_, i64>(5)? != 0,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn delete_workspace(&self, id: &str) -> Result<bool, StorageError> {
        let conn = self.lock_conn()?;
        let affected = conn.execute("DELETE FROM workspaces WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    // =========================================================================
    // Consent Audit Log Repository
    // =========================================================================

    pub fn record_consent_event(&self, event: &ConsentAuditEvent) -> Result<i64, StorageError> {
        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT INTO consent_audit_log (token_id, action, modality, target_endpoint, payload_bytes, epoch_secs, details)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                event.token_id,
                event.action,
                event.modality,
                event.target_endpoint,
                event.payload_bytes,
                event.epoch_secs,
                event.details,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list_consent_events(
        &self,
        limit: usize,
    ) -> Result<Vec<ConsentAuditEvent>, StorageError> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, token_id, action, modality, target_endpoint, payload_bytes, epoch_secs, details
             FROM consent_audit_log ORDER BY id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(ConsentAuditEvent {
                id: Some(row.get(0)?),
                token_id: row.get(1)?,
                action: row.get(2)?,
                modality: row.get(3)?,
                target_endpoint: row.get(4)?,
                payload_bytes: row.get(5)?,
                epoch_secs: row.get(6)?,
                details: row.get(7)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn count_consent_events(&self) -> Result<usize, StorageError> {
        let conn = self.lock_conn()?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM consent_audit_log", [], |row| {
            row.get(0)
        })?;
        Ok(count as usize)
    }

    // =========================================================================
    // Diagnostic Events Repository
    // =========================================================================

    pub fn record_diagnostic_event(
        &self,
        event: &DiagnosticEventRecord,
    ) -> Result<i64, StorageError> {
        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT INTO diagnostic_events (event_type, severity, source, message, payload_json, created_at_epoch_secs)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                event.event_type,
                event.severity,
                event.source,
                event.message,
                event.payload_json,
                event.created_at_epoch_secs,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list_diagnostic_events(
        &self,
        limit: usize,
    ) -> Result<Vec<DiagnosticEventRecord>, StorageError> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, event_type, severity, source, message, payload_json, created_at_epoch_secs
             FROM diagnostic_events ORDER BY id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(DiagnosticEventRecord {
                id: Some(row.get(0)?),
                event_type: row.get(1)?,
                severity: row.get(2)?,
                source: row.get(3)?,
                message: row.get(4)?,
                payload_json: row.get(5)?,
                created_at_epoch_secs: row.get(6)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    // =========================================================================
    // Cached Models Repository
    // =========================================================================

    pub fn record_cached_model(&self, model: &CachedModelRecord) -> Result<(), StorageError> {
        let modality_str = serde_json::to_string(&model.modality)
            .map_err(|e| StorageError::SerializationError {
                message: e.to_string(),
            })?
            .trim_matches('"')
            .to_string();

        let backend_str = serde_json::to_string(&model.preferred_backend)
            .map_err(|e| StorageError::SerializationError {
                message: e.to_string(),
            })?
            .trim_matches('"')
            .to_string();

        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT INTO cached_models (id, modality, file_path, sha256, size_bytes, preferred_backend, created_at_epoch_secs)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                 modality = excluded.modality,
                 file_path = excluded.file_path,
                 sha256 = excluded.sha256,
                 size_bytes = excluded.size_bytes,
                 preferred_backend = excluded.preferred_backend,
                 created_at_epoch_secs = excluded.created_at_epoch_secs",
            params![
                model.id,
                modality_str,
                model.file_path,
                model.sha256,
                model.size_bytes as i64,
                backend_str,
                model.created_at_epoch_secs as i64,
            ],
        )?;
        Ok(())
    }

    pub fn insert_cached_model(&self, model: &CachedModelRecord) -> Result<(), StorageError> {
        self.record_cached_model(model)
    }

    pub fn get_cached_model(&self, id: &str) -> Result<Option<CachedModelRecord>, StorageError> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, modality, file_path, sha256, size_bytes, preferred_backend, created_at_epoch_secs
             FROM cached_models WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            let modality_raw: String = row.get(1)?;
            let modality: Modality =
                serde_json::from_str(&format!("\"{modality_raw}\"")).map_err(|e| {
                    StorageError::SerializationError {
                        message: e.to_string(),
                    }
                })?;

            let backend_raw: String = row.get(5)?;
            let preferred_backend: AcceleratorBackend =
                serde_json::from_str(&format!("\"{backend_raw}\"")).map_err(|e| {
                    StorageError::SerializationError {
                        message: e.to_string(),
                    }
                })?;

            Ok(Some(CachedModelRecord {
                id: row.get(0)?,
                modality,
                file_path: row.get(2)?,
                sha256: row.get(3)?,
                size_bytes: row.get::<_, i64>(4)? as u64,
                preferred_backend,
                created_at_epoch_secs: row.get::<_, i64>(6)? as u64,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_cached_models(
        &self,
        filter_modality: Option<Modality>,
    ) -> Result<Vec<CachedModelRecord>, StorageError> {
        let conn = self.lock_conn()?;
        let mut out = Vec::new();

        if let Some(filter) = filter_modality {
            let filter_str = serde_json::to_string(&filter)
                .map_err(|e| StorageError::SerializationError {
                    message: e.to_string(),
                })?
                .trim_matches('"')
                .to_string();

            let mut stmt = conn.prepare(
                "SELECT id, modality, file_path, sha256, size_bytes, preferred_backend, created_at_epoch_secs
                 FROM cached_models WHERE modality = ?1 ORDER BY id ASC",
            )?;
            let rows = stmt.query_map(params![filter_str], |row| {
                let id: String = row.get(0)?;
                let modality_raw: String = row.get(1)?;
                let file_path: String = row.get(2)?;
                let sha256: String = row.get(3)?;
                let size_bytes: i64 = row.get(4)?;
                let backend_raw: String = row.get(5)?;
                let created_at: i64 = row.get(6)?;
                Ok((
                    id,
                    modality_raw,
                    file_path,
                    sha256,
                    size_bytes,
                    backend_raw,
                    created_at,
                ))
            })?;

            for r in rows {
                let (id, mod_raw, file_path, sha256, size_bytes, backend_raw, created_at) = r?;
                let modality: Modality =
                    serde_json::from_str(&format!("\"{mod_raw}\"")).map_err(|e| {
                        StorageError::SerializationError {
                            message: e.to_string(),
                        }
                    })?;
                let preferred_backend: AcceleratorBackend =
                    serde_json::from_str(&format!("\"{backend_raw}\"")).map_err(|e| {
                        StorageError::SerializationError {
                            message: e.to_string(),
                        }
                    })?;
                out.push(CachedModelRecord {
                    id,
                    modality,
                    file_path,
                    sha256,
                    size_bytes: size_bytes as u64,
                    preferred_backend,
                    created_at_epoch_secs: created_at as u64,
                });
            }
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, modality, file_path, sha256, size_bytes, preferred_backend, created_at_epoch_secs
                 FROM cached_models ORDER BY id ASC",
            )?;
            let rows = stmt.query_map([], |row| {
                let id: String = row.get(0)?;
                let modality_raw: String = row.get(1)?;
                let file_path: String = row.get(2)?;
                let sha256: String = row.get(3)?;
                let size_bytes: i64 = row.get(4)?;
                let backend_raw: String = row.get(5)?;
                let created_at: i64 = row.get(6)?;
                Ok((
                    id,
                    modality_raw,
                    file_path,
                    sha256,
                    size_bytes,
                    backend_raw,
                    created_at,
                ))
            })?;

            for r in rows {
                let (id, mod_raw, file_path, sha256, size_bytes, backend_raw, created_at) = r?;
                let modality: Modality =
                    serde_json::from_str(&format!("\"{mod_raw}\"")).map_err(|e| {
                        StorageError::SerializationError {
                            message: e.to_string(),
                        }
                    })?;
                let preferred_backend: AcceleratorBackend =
                    serde_json::from_str(&format!("\"{backend_raw}\"")).map_err(|e| {
                        StorageError::SerializationError {
                            message: e.to_string(),
                        }
                    })?;
                out.push(CachedModelRecord {
                    id,
                    modality,
                    file_path,
                    sha256,
                    size_bytes: size_bytes as u64,
                    preferred_backend,
                    created_at_epoch_secs: created_at as u64,
                });
            }
        }

        Ok(out)
    }

    pub fn delete_cached_model(&self, id: &str) -> Result<bool, StorageError> {
        let conn = self.lock_conn()?;
        let affected = conn.execute("DELETE FROM cached_models WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    // =========================================================================
    // Chat Sessions Repository
    // =========================================================================

    pub fn create_chat_session(&self, session: &ChatSessionRecord) -> Result<(), StorageError> {
        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT INTO chat_sessions (id, title, created_at_epoch_secs, updated_at_epoch_secs)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO UPDATE SET
                 title = excluded.title,
                 updated_at_epoch_secs = excluded.updated_at_epoch_secs",
            params![
                session.id,
                session.title,
                session.created_at_epoch_secs as i64,
                session.updated_at_epoch_secs as i64,
            ],
        )?;
        Ok(())
    }

    pub fn get_chat_session(&self, id: &str) -> Result<Option<ChatSessionRecord>, StorageError> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, created_at_epoch_secs, updated_at_epoch_secs FROM chat_sessions WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(ChatSessionRecord {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at_epoch_secs: row.get::<_, i64>(2)? as u64,
                updated_at_epoch_secs: row.get::<_, i64>(3)? as u64,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_chat_sessions(&self) -> Result<Vec<ChatSessionRecord>, StorageError> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, created_at_epoch_secs, updated_at_epoch_secs
             FROM chat_sessions ORDER BY updated_at_epoch_secs DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ChatSessionRecord {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at_epoch_secs: row.get::<_, i64>(2)? as u64,
                updated_at_epoch_secs: row.get::<_, i64>(3)? as u64,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn update_chat_session_title(
        &self,
        id: &str,
        title: &str,
        updated_at_epoch_secs: u64,
    ) -> Result<(), StorageError> {
        let conn = self.lock_conn()?;
        let affected = conn.execute(
            "UPDATE chat_sessions SET title = ?2, updated_at_epoch_secs = ?3 WHERE id = ?1",
            params![id, title, updated_at_epoch_secs as i64],
        )?;
        if affected == 0 {
            return Err(StorageError::NotFound {
                entity: "chat_session",
                key: id.to_string(),
            });
        }
        Ok(())
    }

    pub fn delete_chat_session(&self, id: &str) -> Result<bool, StorageError> {
        let conn = self.lock_conn()?;
        let affected = conn.execute("DELETE FROM chat_sessions WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    pub fn insert_model_manifest(&self, record: &ModelManifestRecord) -> Result<(), StorageError> {
        let conn = self.lock_conn()?;
        let modality_str = serde_json::to_string(&record.modality)
            .map_err(|e| StorageError::SerializationError {
                message: e.to_string(),
            })?
            .trim_matches('"')
            .to_string();

        let backend_str = serde_json::to_string(&record.recommended_backend)
            .map_err(|e| StorageError::SerializationError {
                message: e.to_string(),
            })?
            .trim_matches('"')
            .to_string();

        conn.execute(
            "INSERT INTO model_manifests (
                model_id, name, modality, version, description, source_url,
                expected_sha256, expected_size_bytes, format, parameter_count,
                quantization, min_ram_bytes, recommended_backend, created_at_epoch_secs
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
             ON CONFLICT(model_id) DO UPDATE SET
                name = excluded.name,
                modality = excluded.modality,
                version = excluded.version,
                description = excluded.description,
                source_url = excluded.source_url,
                expected_sha256 = excluded.expected_sha256,
                expected_size_bytes = excluded.expected_size_bytes,
                format = excluded.format,
                parameter_count = excluded.parameter_count,
                quantization = excluded.quantization,
                min_ram_bytes = excluded.min_ram_bytes,
                recommended_backend = excluded.recommended_backend",
            params![
                record.model_id,
                record.name,
                modality_str,
                record.version,
                record.description,
                record.source_url,
                record.expected_sha256,
                record.expected_size_bytes as i64,
                record.format,
                record.parameter_count,
                record.quantization,
                record.min_ram_bytes as i64,
                backend_str,
                record.created_at_epoch_secs as i64,
            ],
        )?;
        Ok(())
    }

    pub fn get_model_manifest(
        &self,
        model_id: &str,
    ) -> Result<Option<ModelManifestRecord>, StorageError> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT model_id, name, modality, version, description, source_url,
                    expected_sha256, expected_size_bytes, format, parameter_count,
                    quantization, min_ram_bytes, recommended_backend, created_at_epoch_secs
             FROM model_manifests WHERE model_id = ?1",
        )?;
        let mut rows = stmt.query(params![model_id])?;
        if let Some(row) = rows.next()? {
            let record = Self::decode_manifest_row(row)?;
            Ok(Some(record))
        } else {
            Ok(None)
        }
    }

    pub fn list_model_manifests(
        &self,
        filter_modality: Option<Modality>,
    ) -> Result<Vec<ModelManifestRecord>, StorageError> {
        let conn = self.lock_conn()?;
        let mut out = Vec::new();

        if let Some(filter) = filter_modality {
            let filter_str = serde_json::to_string(&filter)
                .map_err(|e| StorageError::SerializationError {
                    message: e.to_string(),
                })?
                .trim_matches('"')
                .to_string();

            let mut stmt = conn.prepare(
                "SELECT model_id, name, modality, version, description, source_url,
                        expected_sha256, expected_size_bytes, format, parameter_count,
                        quantization, min_ram_bytes, recommended_backend, created_at_epoch_secs
                 FROM model_manifests WHERE modality = ?1 ORDER BY model_id ASC",
            )?;
            let mut rows = stmt.query(params![filter_str])?;
            while let Some(row) = rows.next()? {
                out.push(Self::decode_manifest_row(row)?);
            }
        } else {
            let mut stmt = conn.prepare(
                "SELECT model_id, name, modality, version, description, source_url,
                        expected_sha256, expected_size_bytes, format, parameter_count,
                        quantization, min_ram_bytes, recommended_backend, created_at_epoch_secs
                 FROM model_manifests ORDER BY model_id ASC",
            )?;
            let mut rows = stmt.query([])?;
            while let Some(row) = rows.next()? {
                out.push(Self::decode_manifest_row(row)?);
            }
        }

        Ok(out)
    }

    pub fn delete_model_manifest(&self, model_id: &str) -> Result<bool, StorageError> {
        let conn = self.lock_conn()?;
        let affected = conn.execute(
            "DELETE FROM model_manifests WHERE model_id = ?1",
            params![model_id],
        )?;
        Ok(affected > 0)
    }

    fn decode_manifest_row(row: &rusqlite::Row) -> Result<ModelManifestRecord, StorageError> {
        let modality_raw: String = row.get(2)?;
        let modality: Modality =
            serde_json::from_str(&format!("\"{modality_raw}\"")).map_err(|e| {
                StorageError::SerializationError {
                    message: e.to_string(),
                }
            })?;

        let backend_raw: String = row.get(12)?;
        let recommended_backend: AcceleratorBackend =
            serde_json::from_str(&format!("\"{backend_raw}\"")).map_err(|e| {
                StorageError::SerializationError {
                    message: e.to_string(),
                }
            })?;

        Ok(ModelManifestRecord {
            model_id: row.get(0)?,
            name: row.get(1)?,
            modality,
            version: row.get(3)?,
            description: row.get(4)?,
            source_url: row.get(5)?,
            expected_sha256: row.get(6)?,
            expected_size_bytes: row.get::<_, i64>(7)? as u64,
            format: row.get(8)?,
            parameter_count: row.get(9)?,
            quantization: row.get(10)?,
            min_ram_bytes: row.get::<_, i64>(11)? as u64,
            recommended_backend,
            created_at_epoch_secs: row.get::<_, i64>(13)? as u64,
        })
    }

    // =========================================================================
    // Chat Messages Repository & Branching Tree
    // =========================================================================

    pub fn create_chat_message(&self, message: &ChatMessageRecord) -> Result<(), StorageError> {
        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT INTO chat_messages (id, session_id, parent_id, role, content, token_count, model_id, created_at_epoch_secs, updated_at_epoch_secs, is_pinned)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(id) DO UPDATE SET
                 content = excluded.content,
                 token_count = excluded.token_count,
                 updated_at_epoch_secs = excluded.updated_at_epoch_secs,
                 is_pinned = excluded.is_pinned",
            params![
                message.id,
                message.session_id,
                message.parent_id,
                message.role.as_str(),
                message.content,
                message.token_count as i64,
                message.model_id,
                message.created_at_epoch_secs as i64,
                message.updated_at_epoch_secs as i64,
                if message.is_pinned { 1 } else { 0 },
            ],
        )?;

        // Bump session updated_at timestamp
        let _ = conn.execute(
            "UPDATE chat_sessions SET updated_at_epoch_secs = ?2 WHERE id = ?1",
            params![message.session_id, message.updated_at_epoch_secs as i64],
        );

        Ok(())
    }

    pub fn get_chat_message(&self, id: &str) -> Result<Option<ChatMessageRecord>, StorageError> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, session_id, parent_id, role, content, token_count, model_id, created_at_epoch_secs, updated_at_epoch_secs, is_pinned
             FROM chat_messages WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::decode_message_row(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn list_chat_messages(
        &self,
        session_id: &str,
    ) -> Result<Vec<ChatMessageRecord>, StorageError> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, session_id, parent_id, role, content, token_count, model_id, created_at_epoch_secs, updated_at_epoch_secs, is_pinned
             FROM chat_messages WHERE session_id = ?1 ORDER BY created_at_epoch_secs ASC, id ASC",
        )?;
        let rows = stmt.query_map(params![session_id], Self::decode_message_row)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn update_chat_message_content(
        &self,
        id: &str,
        new_content: &str,
        new_token_count: usize,
        updated_at_epoch_secs: u64,
    ) -> Result<bool, StorageError> {
        let conn = self.lock_conn()?;
        let affected = conn.execute(
            "UPDATE chat_messages SET content = ?2, token_count = ?3, updated_at_epoch_secs = ?4 WHERE id = ?1",
            params![id, new_content, new_token_count as i64, updated_at_epoch_secs as i64],
        )?;
        Ok(affected > 0)
    }

    pub fn delete_chat_message(&self, id: &str) -> Result<bool, StorageError> {
        let conn = self.lock_conn()?;
        let affected = conn.execute("DELETE FROM chat_messages WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    pub fn delete_chat_messages_for_session(
        &self,
        session_id: &str,
    ) -> Result<usize, StorageError> {
        let conn = self.lock_conn()?;
        let affected = conn.execute(
            "DELETE FROM chat_messages WHERE session_id = ?1",
            params![session_id],
        )?;
        Ok(affected)
    }

    /// Traverses up the `parent_id` hierarchy starting from `leaf_message_id` to assemble
    /// the full linear conversation branch from root to leaf in chronological order.
    pub fn get_message_branch(
        &self,
        leaf_message_id: &str,
    ) -> Result<Vec<ChatMessageRecord>, StorageError> {
        let conn = self.lock_conn()?;
        let mut branch = Vec::new();
        let mut current_id = Some(leaf_message_id.to_string());

        let mut stmt = conn.prepare(
            "SELECT id, session_id, parent_id, role, content, token_count, model_id, created_at_epoch_secs, updated_at_epoch_secs, is_pinned
             FROM chat_messages WHERE id = ?1",
        )?;

        while let Some(id) = current_id {
            let mut rows = stmt.query(params![id])?;
            if let Some(row) = rows.next()? {
                let msg = Self::decode_message_row(row)?;
                current_id = msg.parent_id.clone();
                branch.push(msg);
            } else {
                break;
            }
        }

        // Reverse so branch is in chronological order [root -> ... -> leaf]
        branch.reverse();
        Ok(branch)
    }

    /// Finds alternative child messages that branch off from a specified parent message.
    pub fn get_child_branches(
        &self,
        parent_message_id: &str,
    ) -> Result<Vec<ChatMessageRecord>, StorageError> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, session_id, parent_id, role, content, token_count, model_id, created_at_epoch_secs, updated_at_epoch_secs, is_pinned
             FROM chat_messages WHERE parent_id = ?1 ORDER BY created_at_epoch_secs ASC",
        )?;
        let rows = stmt.query_map(params![parent_message_id], Self::decode_message_row)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    /// Exports a conversation session to formatted Markdown.
    pub fn export_session_markdown(
        &self,
        session_id: &str,
        leaf_message_id: Option<&str>,
    ) -> Result<String, StorageError> {
        let session = self
            .get_chat_session(session_id)?
            .ok_or_else(|| StorageError::NotFound {
                entity: "ChatSession",
                key: session_id.to_string(),
            })?;

        let messages = if let Some(leaf_id) = leaf_message_id {
            self.get_message_branch(leaf_id)?
        } else {
            self.list_chat_messages(session_id)?
        };

        let mut md = String::new();
        md.push_str(&format!("# Chat Session: {}\n\n", session.title));
        md.push_str(&format!("*Session ID:* `{}`  \n", session.id));
        md.push_str(&format!(
            "*Exported Messages:* {}\n\n---\n\n",
            messages.len()
        ));

        for msg in messages {
            let header = match msg.role {
                MessageRole::System => "### ⚙️ System".to_string(),
                MessageRole::User => "### 👤 User".to_string(),
                MessageRole::Assistant => {
                    if let Some(ref m) = msg.model_id {
                        format!("### 🤖 Assistant (`{m}`)")
                    } else {
                        "### 🤖 Assistant".to_string()
                    }
                }
            };
            md.push_str(&format!("{header}\n\n{}\n\n", msg.content.trim()));
        }

        Ok(md)
    }

    /// Exports a conversation session and all its messages to JSON.
    pub fn export_session_json(&self, session_id: &str) -> Result<String, StorageError> {
        let session = self
            .get_chat_session(session_id)?
            .ok_or_else(|| StorageError::NotFound {
                entity: "ChatSession",
                key: session_id.to_string(),
            })?;

        let messages = self.list_chat_messages(session_id)?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let export = ChatSessionExport {
            session,
            messages,
            exported_at_epoch_secs: now,
        };

        serde_json::to_string_pretty(&export).map_err(|e| StorageError::SerializationError {
            message: e.to_string(),
        })
    }

    fn decode_message_row(row: &rusqlite::Row) -> rusqlite::Result<ChatMessageRecord> {
        let role_raw: String = row.get(3)?;
        let role = match role_raw.to_ascii_lowercase().as_str() {
            "system" => MessageRole::System,
            "assistant" => MessageRole::Assistant,
            _ => MessageRole::User,
        };
        let is_pinned_int: i64 = row.get(9)?;

        Ok(ChatMessageRecord {
            id: row.get(0)?,
            session_id: row.get(1)?,
            parent_id: row.get(2)?,
            role,
            content: row.get(4)?,
            token_count: row.get::<_, i64>(5)? as usize,
            model_id: row.get(6)?,
            created_at_epoch_secs: row.get::<_, i64>(7)? as u64,
            updated_at_epoch_secs: row.get::<_, i64>(8)? as u64,
            is_pinned: is_pinned_int != 0,
        })
    }
}
