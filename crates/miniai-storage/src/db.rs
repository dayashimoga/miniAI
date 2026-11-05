use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::{params, Connection, OpenFlags};

use crate::error::StorageError;
use crate::migrations::{MigrationRunner, MIGRATIONS};
use crate::models::{
    AppSetting, ConsentAuditEvent, DiagnosticEventRecord, IntegrityReport, Workspace,
};

/// A thread-safe, transactional SQLite storage engine with WAL mode and migration support.
#[derive(Clone)]
pub struct StorageEngine {
    conn: Arc<Mutex<Connection>>,
    is_in_memory: bool,
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

    /// Acquires the connection mutex.
    fn lock_conn(&self) -> Result<std::sync::MutexGuard<'_, Connection>, StorageError> {
        self.conn.lock().map_err(|_| StorageError::LockPoisoned)
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
}
