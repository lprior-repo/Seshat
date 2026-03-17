use crate::store::sqlx::error::*;
use crate::store::sqlx::models::*;
use crate::store::sqlx::db_init::create_pool;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::{Path, PathBuf};

/// Runs integrity check on the database at startup (async version)
///
/// # Errors
///
/// Returns a `RecoveryError` if the integrity check fails.
pub async fn startup_integrity_check(db_path: &Path) -> Result<IntegrityStatus, RecoveryError> {
    if !db_path.exists() {
        return Ok(IntegrityStatus {
            is_valid: false,
            page_count: 0,
            free_pages: 0,
            corrupted_pages: 0,
            schema_version: None,
            event_count: 0,
            latest_revision: None,
            error_message: Some("Database file does not exist".to_string()),
        });
    }

    let pool = create_pool(db_path).await?;

    let integrity_result: String = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_one(&pool)
        .await
        .map_err(RecoveryError::Sqlx)?;

    let is_valid = integrity_result == "ok";

    let page_count: u32 = sqlx::query_scalar("PRAGMA page_count")
        .fetch_one(&pool)
        .await
        .map_err(RecoveryError::Sqlx)?;

    let free_pages: u32 = sqlx::query_scalar("PRAGMA freelist_count")
        .fetch_one(&pool)
        .await
        .map_err(RecoveryError::Sqlx)?;

    let corrupted_pages: u32 = u32::from(!is_valid && integrity_result.contains("corrupt"));

    let schema_version: Option<i32> = sqlx::query_scalar("SELECT version FROM schema_version")
        .fetch_optional(&pool)
        .await
        .map_err(RecoveryError::Sqlx)?;

    let event_count: u64 = sqlx::query_scalar("SELECT COUNT(*) FROM events")
        .fetch_one(&pool)
        .await
        .map_err(RecoveryError::Sqlx)?;

    let latest_revision: Option<i64> =
        sqlx::query_scalar::<_, Option<i64>>("SELECT COALESCE(MAX(revision), 0) FROM events")
            .fetch_optional(&pool)
            .await
            .map_err(RecoveryError::Sqlx)?
            .flatten()
            .filter(|&rev| rev > 0);

    pool.close().await;

    let error_message = if !is_valid {
        Some(integrity_result)
    } else if corrupted_pages > 0 {
        Some(format!("{corrupted_pages} corrupted pages found"))
    } else {
        None
    };

    Ok(IntegrityStatus {
        is_valid,
        page_count,
        free_pages,
        corrupted_pages,
        schema_version,
        event_count,
        latest_revision,
        error_message,
    })
}

/// Opens the database in read-only recovery mode (async version)
///
/// # Errors
///
/// Returns a `RecoveryError` if the database is corrupt or cannot be opened.
pub async fn open_recovery_mode(db_path: &Path) -> Result<RecoveryHandle, RecoveryError> {
    let connection_string = format!("sqlite:{}?mode=ro", db_path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&connection_string)
        .await
        .map_err(RecoveryError::Sqlx)?;

    let integrity_result: String = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_one(&pool)
        .await
        .map_err(RecoveryError::Sqlx)?;

    if integrity_result != "ok" {
        pool.close().await;
        return Err(RecoveryError::CorruptDatabase(integrity_result));
    }

    Ok(RecoveryHandle {
        pool,
        db_path: db_path.to_path_buf(),
        export_path: None,
    })
}

/// Opens the database in recovery-only mode (async version - alias)
///
/// # Errors
///
/// Returns a `RecoveryError` if the database is corrupt or cannot be opened.
pub async fn open_recovery_only(db_path: &Path) -> Result<RecoverySession, RecoveryError> {
    open_recovery_mode(db_path).await
}

/// Runs integrity check on the database (async version - alias)
///
/// # Errors
///
/// Returns a `RecoveryError` if the integrity check fails.
pub async fn integrity_check(db_path: &Path) -> Result<IntegrityStatus, RecoveryError> {
    startup_integrity_check(db_path).await
}
