use crate::error::StorageError;
use miniai_core::compute_sha256_hex;
use rusqlite::Connection;

/// A deterministic database schema migration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Migration {
    pub version: u32,
    pub name: &'static str,
    pub sql: &'static str,
}

impl Migration {
    pub const fn new(version: u32, name: &'static str, sql: &'static str) -> Self {
        Self { version, name, sql }
    }

    /// Computes the SHA-256 hex digest of the migration SQL string.
    pub fn checksum(&self) -> String {
        compute_sha256_hex(self.sql.as_bytes())
    }
}

/// Baseline migration set for MiniAI.
pub const MIGRATIONS: &[Migration] = &[
    Migration::new(
        1,
        "001_initial_schema",
        r#"
        CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at_epoch_secs INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS workspaces (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            root_path TEXT NOT NULL,
            created_at_epoch_secs INTEGER NOT NULL,
            updated_at_epoch_secs INTEGER NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS consent_audit_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            token_id TEXT NOT NULL,
            action TEXT NOT NULL,
            modality TEXT NOT NULL,
            target_endpoint TEXT NOT NULL,
            payload_bytes INTEGER NOT NULL,
            epoch_secs INTEGER NOT NULL,
            details TEXT
        );

        CREATE TABLE IF NOT EXISTS diagnostic_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            event_type TEXT NOT NULL,
            severity TEXT NOT NULL,
            source TEXT NOT NULL,
            message TEXT NOT NULL,
            payload_json TEXT,
            created_at_epoch_secs INTEGER NOT NULL
        );
        "#,
    ),
    Migration::new(
        2,
        "002_model_registry_and_sessions",
        r#"
        CREATE TABLE IF NOT EXISTS cached_models (
            id TEXT PRIMARY KEY,
            modality TEXT NOT NULL,
            file_path TEXT NOT NULL,
            sha256 TEXT NOT NULL,
            size_bytes INTEGER NOT NULL,
            preferred_backend TEXT NOT NULL,
            created_at_epoch_secs INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS chat_sessions (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            created_at_epoch_secs INTEGER NOT NULL,
            updated_at_epoch_secs INTEGER NOT NULL
        );
        "#,
    ),
    Migration::new(
        3,
        "003_model_manifests",
        r#"
        CREATE TABLE IF NOT EXISTS model_manifests (
            model_id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            modality TEXT NOT NULL,
            version TEXT NOT NULL,
            description TEXT NOT NULL,
            source_url TEXT NOT NULL,
            expected_sha256 TEXT NOT NULL,
            expected_size_bytes INTEGER NOT NULL,
            format TEXT NOT NULL,
            parameter_count TEXT,
            quantization TEXT,
            min_ram_bytes INTEGER NOT NULL,
            recommended_backend TEXT NOT NULL,
            created_at_epoch_secs INTEGER NOT NULL
        );
        "#,
    ),
];

/// Manages and executes database schema migrations.
pub struct MigrationRunner;

impl MigrationRunner {
    /// Applies all pending migrations up to the current set.
    pub fn run_migrations(
        conn: &mut Connection,
        migrations: &[Migration],
        current_epoch_secs: u64,
    ) -> Result<usize, StorageError> {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at_epoch_secs INTEGER NOT NULL,
                checksum TEXT NOT NULL
            );
            "#,
        )?;

        let applied: Vec<(u32, String, String)> = {
            let mut stmt = conn.prepare(
                "SELECT version, name, checksum FROM schema_migrations ORDER BY version ASC",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, u32>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?;
            let mut items = Vec::new();
            for r in rows {
                items.push(r?);
            }
            items
        };

        let mut applied_map = std::collections::BTreeMap::new();
        for (v, n, c) in applied {
            applied_map.insert(v, (n, c));
        }

        let mut newly_applied_count = 0;

        for migration in migrations {
            let expected_checksum = migration.checksum();
            if let Some((existing_name, existing_checksum)) = applied_map.get(&migration.version) {
                if existing_checksum != &expected_checksum {
                    return Err(StorageError::MigrationChecksumMismatch {
                        version: migration.version,
                        name: migration.name.to_string(),
                        expected: expected_checksum,
                        recorded: existing_checksum.clone(),
                    });
                }
                if existing_name != migration.name {
                    return Err(StorageError::MigrationError {
                        message: format!(
                            "Migration version {} name mismatch: expected '{}', found '{}'",
                            migration.version, migration.name, existing_name
                        ),
                    });
                }
            } else {
                // Apply pending migration within an atomic transaction
                let tx = conn.transaction()?;
                tx.execute_batch(migration.sql)?;
                tx.execute(
                    "INSERT INTO schema_migrations (version, name, applied_at_epoch_secs, checksum) VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![
                        migration.version,
                        migration.name,
                        current_epoch_secs,
                        expected_checksum
                    ],
                )?;
                tx.commit()?;
                newly_applied_count += 1;
            }
        }

        Ok(newly_applied_count)
    }

    /// Returns the highest applied schema version.
    pub fn current_version(conn: &Connection) -> Result<u32, StorageError> {
        let mut stmt = match conn.prepare("SELECT MAX(version) FROM schema_migrations") {
            Ok(stmt) => stmt,
            Err(_) => return Ok(0),
        };
        let version: Option<u32> = stmt.query_row([], |row| row.get(0)).unwrap_or(None);
        Ok(version.unwrap_or(0))
    }
}
