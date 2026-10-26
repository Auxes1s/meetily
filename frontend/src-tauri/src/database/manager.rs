use sqlx::{sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous}, Result, Sqlite, SqlitePool, Transaction};
use std::{str::FromStr, time::Duration};
use std::fs;
use std::path::Path;
use tauri::Manager;

#[derive(Clone)]
pub struct DatabaseManager {
    pool: SqlitePool,
}

impl DatabaseManager {
    pub async fn new(tauri_db_path: &str, backend_db_path: &str) -> Result<Self> {
        let path = Path::new(tauri_db_path);
        if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
        // SQLite snapshots include committed WAL pages; a raw filesystem copy does not.
        if !path.exists() && Path::new(backend_db_path).exists() {
            let legacy = SqlitePool::connect_with(SqliteConnectOptions::from_str(backend_db_path)?.read_only(true)).await?;
            sqlx::query("VACUUM INTO ?").bind(tauri_db_path).execute(&legacy).await?;
            legacy.close().await;
        }
        let existed = path.exists();
        let options = SqliteConnectOptions::from_str(tauri_db_path)?
            .create_if_missing(true).foreign_keys(true).busy_timeout(Duration::from_secs(5))
            .journal_mode(SqliteJournalMode::Wal).synchronous(SqliteSynchronous::Full);
        // Check the source before changing its connection/journal policy.
        if existed {
            let read = SqlitePool::connect_with(SqliteConnectOptions::from_str(tauri_db_path)?.read_only(true)).await?;
            let integrity: String = sqlx::query_scalar("PRAGMA integrity_check").fetch_one(&read).await?;
            if integrity != "ok" { return Err(sqlx::Error::Protocol("Database integrity check failed; source files preserved".into())); }
            read.close().await;
        }
        let pool = SqlitePoolOptions::new().max_connections(4).connect_with(options).await?;
        let migrator = sqlx::migrate!("./migrations");
        let has_migrations: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sqlite_master WHERE name = '_sqlx_migrations'").fetch_one(&pool).await?;
        let versions: Vec<i64> = if has_migrations > 0 {
            sqlx::query_scalar("SELECT version FROM _sqlx_migrations WHERE success = 1").fetch_all(&pool).await?
        } else { Vec::new() };
        let pending = migrator.iter().any(|m| !versions.contains(&m.version));
        let backup_dir = path.with_extension("sqlite.backups");
        if existed && pending {
            fs::create_dir_all(&backup_dir)?;
            let stem = format!("{}-{}", chrono::Utc::now().format("%Y%m%dT%H%M%S"), uuid::Uuid::new_v4());
            let partial = backup_dir.join(format!("{stem}.partial"));
            sqlx::query("VACUUM INTO ?").bind(partial.to_string_lossy().as_ref()).execute(&pool).await?;
            let snapshot = SqlitePool::connect_with(SqliteConnectOptions::from_str(partial.to_string_lossy().as_ref())?.read_only(true)).await?;
            let integrity: String = sqlx::query_scalar("PRAGMA integrity_check").fetch_one(&snapshot).await?;
            snapshot.close().await;
            if integrity != "ok" { return Err(sqlx::Error::Protocol("Migration backup integrity check failed".into())); }
            fs::File::open(&partial)?.sync_all()?;
            fs::rename(&partial, backup_dir.join(format!("{stem}.sqlite")))?;
            // Windows cannot open a directory with the standard File API.
            // The snapshot itself has already been synced before atomic rename.
            #[cfg(unix)]
            fs::File::open(&backup_dir)?.sync_all()?;
        }
        migrator.run(&pool).await?;
        crate::summary::enhancement::recover_interrupted_enhancements(&pool).await
            .map_err(sqlx::Error::Protocol)?;
        // A previous process cannot still own a live recording after this process starts.
        sqlx::query("UPDATE meetings SET status = 'interrupted' WHERE status IN ('recording', 'finalizing')").execute(&pool).await?;
        sqlx::query("UPDATE recording_sessions SET status='interrupted' WHERE status IN ('starting','recording','finalizing')").execute(&pool).await?;
        if pending && backup_dir.exists() {
            let mut backups: Vec<_> = fs::read_dir(&backup_dir)?.filter_map(|e| e.ok())
                .map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "sqlite")).collect();
            backups.sort();
            let excess = backups.len().saturating_sub(3);
            for backup in backups.into_iter().take(excess) { fs::remove_file(backup)?; }
        }
        Ok(DatabaseManager { pool })
    }

    pub async fn new_from_app_handle(app_handle: &tauri::AppHandle) -> Result<Self> {
        let app_data_dir = app_handle.path().app_data_dir().map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        Self::new(
            app_data_dir.join("meeting_minutes.sqlite").to_string_lossy().as_ref(),
            app_data_dir.join("meeting_minutes.db").to_string_lossy().as_ref(),
        ).await
    }

    /// Check if this is the first launch (sqlite database doesn't exist yet)
    pub async fn is_first_launch(app_handle: &tauri::AppHandle) -> Result<bool> {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .expect("failed to get app data dir");

        let tauri_db_path = app_data_dir.join("meeting_minutes.sqlite");

        Ok(!tauri_db_path.exists())
    }

    /// Import a legacy database from the specified path and initialize
    pub async fn import_legacy_database(
        app_handle: &tauri::AppHandle,
        legacy_db_path: &str,
    ) -> Result<Self> {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .expect("failed to get app data dir");

        if !app_data_dir.exists() {
            fs::create_dir_all(&app_data_dir).map_err(|e| sqlx::Error::Io(e))?;
        }

        // Copy legacy database to app data directory as meeting_minutes.db
        let target_legacy_path = app_data_dir.join("meeting_minutes.db");
        log::info!(
            "Copying legacy database from {} to {}",
            legacy_db_path,
            target_legacy_path.display()
        );

        if target_legacy_path.exists() {
            return Err(sqlx::Error::Protocol("An imported database already exists; it has been preserved".into()));
        }
        let source = SqlitePool::connect_with(SqliteConnectOptions::from_str(legacy_db_path)?.read_only(true)).await?;
        let snapshot = sqlx::query("VACUUM INTO ?").bind(target_legacy_path.to_string_lossy().as_ref()).execute(&source).await;
        source.close().await;
        snapshot?;

        // Now use the standard initialization which will detect and migrate the legacy db
        Self::new_from_app_handle(app_handle).await
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn with_transaction<T, F, Fut>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut Transaction<'_, Sqlite>) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut tx = self.pool.begin().await?;
        let result = f(&mut tx).await;

        match result {
            Ok(val) => {
                tx.commit().await?;
                Ok(val)
            }
            Err(err) => {
                tx.rollback().await?;
                Err(err)
            }
        }
    }

    /// Cleanup database connection and checkpoint WAL
    /// This should be called on application shutdown to ensure:
    /// - All WAL changes are written to the main database file
    /// - The .wal and .shm files are deleted
    /// - Connection pool is gracefully closed
    pub async fn cleanup(&self) -> Result<()> {
        log::info!("Starting database cleanup...");

        // Force checkpoint of WAL to main database file and remove WAL file
        // TRUNCATE mode: checkpoints all pages AND deletes the WAL file
        match sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
            .execute(&self.pool)
            .await
        {
            Ok(_) => log::info!("WAL checkpoint completed successfully"),
            Err(e) => log::warn!("WAL checkpoint failed (non-fatal): {}", e),
        }

        // Close the connection pool gracefully
        self.pool.close().await;
        log::info!("Database connection pool closed");

        Ok(())
    }
}
