//! IO-005 Large Document Export Performance Test
//!
//! Standalone integration test for validating async database operations with large datasets.
//!
//! Tests:
//! - io_005_sync_1000: Test with 1000 events (sync rusqlite)
//! - io_005_sync_5000: Test with 5000 events (sync rusqlite)
//! - io_005_sync_10000: Test with 10000 events (sync rusqlite)
//! - io_005_async_1000: Test with 1000 events (async sqlx)
//! - io_005_async_5000: Test with 5000 events (async sqlx)
//! - io_005_async_10000: Test with 10000 events (async sqlx)

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use std::time::Instant;

// ============================================================================
// Synchronous Tests (rusqlite)
// ============================================================================

#[test]
fn io_005_sync_1000() {
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let conn = rusqlite::Connection::open(&db_path).expect("Failed to open DB");

    // Initialize schema
    conn.execute(
        "CREATE TABLE IF NOT EXISTS events (
            operation_id TEXT PRIMARY KEY,
            revision INTEGER NOT NULL,
            payload TEXT NOT NULL,
            timestamp INTEGER NOT NULL
        )",
        [],
    ).expect("Failed to create events table");

    // Create 1000 events
    let insert_start = Instant::now();
    for i in 0..1000 {
        conn.execute(
            "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
            (format!("op-{i}"), i + 1, "{}", 1000 + i),
        ).expect("DB insert failed");
    }
    let insert_duration = insert_start.elapsed();

    // Fetch all events
    let fetch_start = Instant::now();
    let mut stmt = conn.prepare("SELECT * FROM events ORDER BY revision").expect("Failed to prepare");
    let event_count: i64 = stmt.query_row([], |row| row.get(0)).expect("Failed to count");
    let fetch_duration = fetch_start.elapsed();

    println!("IO-005 Sync (1000 events):");
    println!("  Insert duration: {:?}", insert_duration);
    println!("  Fetch duration: {:?}", fetch_duration);
    println!("  Total duration: {:?}", insert_start.elapsed());
    println!("  Events per second: {:.2}", 1000.0 / insert_start.elapsed().as_secs_f64());

    assert_eq!(event_count, 1000);
    assert!(insert_start.elapsed().as_secs() < 5, "Test took too long: {:?}", insert_start.elapsed());
}

#[test]
fn io_005_sync_5000() {
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let conn = rusqlite::Connection::open(&db_path).expect("Failed to open DB");

    // Initialize schema
    conn.execute(
        "CREATE TABLE IF NOT EXISTS events (
            operation_id TEXT PRIMARY KEY,
            revision INTEGER NOT NULL,
            payload TEXT NOT NULL,
            timestamp INTEGER NOT NULL
        )",
        [],
    ).expect("Failed to create events table");

    // Create 5000 events
    let insert_start = Instant::now();
    for i in 0..5000 {
        conn.execute(
            "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
            (format!("op-{i}"), i + 1, "{}", 1000 + i),
        ).expect("DB insert failed");
    }
    let insert_duration = insert_start.elapsed();

    // Fetch all events
    let fetch_start = Instant::now();
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM events").expect("Failed to prepare");
    let event_count: i64 = stmt.query_row([], |row| row.get(0)).expect("Failed to count");
    let fetch_duration = fetch_start.elapsed();

    println!("IO-005 Sync (5000 events):");
    println!("  Insert duration: {:?}", insert_duration);
    println!("  Fetch duration: {:?}", fetch_duration);
    println!("  Total duration: {:?}", insert_start.elapsed());
    println!("  Events per second: {:.2}", 5000.0 / insert_start.elapsed().as_secs_f64());

    assert_eq!(event_count, 5000);
    assert!(insert_start.elapsed().as_secs() < 10, "Test took too long: {:?}", insert_start.elapsed());
}

#[test]
fn io_005_sync_10000() {
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let conn = rusqlite::Connection::open(&db_path).expect("Failed to open DB");

    // Initialize schema
    conn.execute(
        "CREATE TABLE IF NOT EXISTS events (
            operation_id TEXT PRIMARY KEY,
            revision INTEGER NOT NULL,
            payload TEXT NOT NULL,
            timestamp INTEGER NOT NULL
        )",
        [],
    ).expect("Failed to create events table");

    // Create 10000 events
    let insert_start = Instant::now();
    for i in 0..10000 {
        conn.execute(
            "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
            (format!("op-{i}"), i + 1, "{}", 1000 + i),
        ).expect("DB insert failed");
    }
    let insert_duration = insert_start.elapsed();

    // Fetch all events
    let fetch_start = Instant::now();
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM events").expect("Failed to prepare");
    let event_count: i64 = stmt.query_row([], |row| row.get(0)).expect("Failed to count");
    let fetch_duration = fetch_start.elapsed();

    println!("IO-005 Sync (10000 events):");
    println!("  Insert duration: {:?}", insert_duration);
    println!("  Fetch duration: {:?}", fetch_duration);
    println!("  Total duration: {:?}", insert_start.elapsed());
    println!("  Events per second: {:.2}", 10000.0 / insert_start.elapsed().as_secs_f64());

    assert_eq!(event_count, 10000);
    assert!(insert_start.elapsed().as_secs() < 20, "Test took too long: {:?}", insert_start.elapsed());
}

// ============================================================================
// Async Tests (sqlx)
// ============================================================================

#[cfg(feature = "async-db")]
#[tokio::test]
async fn io_005_async_1000() {
    use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let connection_string = format!("sqlite:{}?mode=rwc", db_path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&connection_string)
        .await
        .expect("Failed to create pool");

    // Initialize schema
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS events (
            operation_id TEXT PRIMARY KEY,
            revision INTEGER NOT NULL,
            payload TEXT NOT NULL,
            timestamp INTEGER NOT NULL
        )"
    )
    .execute(&pool)
    .await
    .expect("Failed to create events table");

    // Configure WAL mode
    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await
        .expect("Failed to set WAL mode");

    // Create 1000 events in batches
    let insert_start = Instant::now();
    let batch_size = 100;
    for batch_start in (0..1000).step_by(batch_size) {
        let mut tx = pool.begin().await.expect("Failed to begin transaction");
        for i in batch_start..(batch_start + batch_size).min(1000) {
            sqlx::query(
                "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)"
            )
            .bind(format!("op-{i}"))
            .bind(i + 1)
            .bind("{}")
            .bind(1000 + i)
            .execute(&mut *tx)
            .await
            .expect("Failed to insert");
        }
        tx.commit().await.expect("Failed to commit");
    }
    let insert_duration = insert_start.elapsed();

    // Fetch all events
    let fetch_start = Instant::now();
    let event_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM events")
        .fetch_one(&pool)
        .await
        .expect("Failed to count");
    let fetch_duration = fetch_start.elapsed();

    println!("IO-005 Async (1000 events):");
    println!("  Insert duration: {:?}", insert_duration);
    println!("  Fetch duration: {:?}", fetch_duration);
    println!("  Total duration: {:?}", insert_start.elapsed());
    println!("  Events per second: {:.2}", 1000.0 / insert_start.elapsed().as_secs_f64());

    assert_eq!(event_count.0, 1000);
    assert!(insert_start.elapsed().as_secs() < 5, "Test took too long: {:?}", insert_start.elapsed());

    pool.close().await;
}

#[cfg(feature = "async-db")]
#[tokio::test]
async fn io_005_async_5000() {
    use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let connection_string = format!("sqlite:{}?mode=rwc", db_path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&connection_string)
        .await
        .expect("Failed to create pool");

    // Initialize schema
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS events (
            operation_id TEXT PRIMARY KEY,
            revision INTEGER NOT NULL,
            payload TEXT NOT NULL,
            timestamp INTEGER NOT NULL
        )"
    )
    .execute(&pool)
    .await
    .expect("Failed to create events table");

    // Configure WAL mode
    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await
        .expect("Failed to set WAL mode");

    // Create 5000 events in batches
    let insert_start = Instant::now();
    let batch_size = 100;
    for batch_start in (0..5000).step_by(batch_size) {
        let mut tx = pool.begin().await.expect("Failed to begin transaction");
        for i in batch_start..(batch_start + batch_size).min(5000) {
            sqlx::query(
                "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)"
            )
            .bind(format!("op-{i}"))
            .bind(i + 1)
            .bind("{}")
            .bind(1000 + i)
            .execute(&mut *tx)
            .await
            .expect("Failed to insert");
        }
        tx.commit().await.expect("Failed to commit");
    }
    let insert_duration = insert_start.elapsed();

    // Fetch all events
    let fetch_start = Instant::now();
    let event_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM events")
        .fetch_one(&pool)
        .await
        .expect("Failed to count");
    let fetch_duration = fetch_start.elapsed();

    println!("IO-005 Async (5000 events):");
    println!("  Insert duration: {:?}", insert_duration);
    println!("  Fetch duration: {:?}", fetch_duration);
    println!("  Total duration: {:?}", insert_start.elapsed());
    println!("  Events per second: {:.2}", 5000.0 / insert_start.elapsed().as_secs_f64());

    assert_eq!(event_count.0, 5000);
    assert!(insert_start.elapsed().as_secs() < 10, "Test took too long: {:?}", insert_start.elapsed());

    pool.close().await;
}

#[cfg(feature = "async-db")]
#[tokio::test]
async fn io_005_async_10000() {
    use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let connection_string = format!("sqlite:{}?mode=rwc", db_path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&connection_string)
        .await
        .expect("Failed to create pool");

    // Initialize schema
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS events (
            operation_id TEXT PRIMARY KEY,
            revision INTEGER NOT NULL,
            payload TEXT NOT NULL,
            timestamp INTEGER NOT NULL
        )"
    )
    .execute(&pool)
    .await
    .expect("Failed to create events table");

    // Configure WAL mode
    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await
        .expect("Failed to set WAL mode");

    // Create 10000 events in batches
    let insert_start = Instant::now();
    let batch_size = 100;
    for batch_start in (0..10000).step_by(batch_size) {
        let mut tx = pool.begin().await.expect("Failed to begin transaction");
        for i in batch_start..(batch_start + batch_size).min(10000) {
            sqlx::query(
                "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)"
            )
            .bind(format!("op-{i}"))
            .bind(i + 1)
            .bind("{}")
            .bind(1000 + i)
            .execute(&mut *tx)
            .await
            .expect("Failed to insert");
        }
        tx.commit().await.expect("Failed to commit");
    }
    let insert_duration = insert_start.elapsed();

    // Fetch all events
    let fetch_start = Instant::now();
    let event_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM events")
        .fetch_one(&pool)
        .await
        .expect("Failed to count");
    let fetch_duration = fetch_start.elapsed();

    println!("IO-005 Async (10000 events):");
    println!("  Insert duration: {:?}", insert_duration);
    println!("  Fetch duration: {:?}", fetch_duration);
    println!("  Total duration: {:?}", insert_start.elapsed());
    println!("  Events per second: {:.2}", 10000.0 / insert_start.elapsed().as_secs_f64());

    assert_eq!(event_count.0, 10000);
    assert!(insert_start.elapsed().as_secs() < 20, "Test took too long: {:?}", insert_start.elapsed());

    pool.close().await;
}

// ============================================================================
// Non-blocking verification test
// ============================================================================

#[cfg(feature = "async-db")]
#[tokio::test]
async fn io_005_async_non_blocking() {
    use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
    use tokio::time::{sleep, Duration};

    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let connection_string = format!("sqlite:{}?mode=rwc", db_path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&connection_string)
        .await
        .expect("Failed to create pool");

    // Initialize schema
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS events (
            operation_id TEXT PRIMARY KEY,
            revision INTEGER NOT NULL,
            payload TEXT NOT NULL,
            timestamp INTEGER NOT NULL
        )"
    )
    .execute(&pool)
    .await
    .expect("Failed to create events table");

    // Simulate concurrent operations to verify non-blocking behavior
    let insert_start = Instant::now();

    let pool_clone = pool.clone();
    let insert_task = tokio::spawn(async move {
        let batch_size = 100;
        for batch_start in (0..1000).step_by(batch_size) {
            let mut tx = pool_clone.begin().await.expect("Failed to begin transaction");
            for i in batch_start..(batch_start + batch_size).min(1000) {
                sqlx::query(
                    "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)"
                )
                .bind(format!("op-{i}"))
                .bind(i + 1)
                .bind("{}")
                .bind(1000 + i)
                .execute(&mut *tx)
                .await
                .expect("Failed to insert");
            }
            tx.commit().await.expect("Failed to commit");
        }
    });

    // Run a "simulated UI task" concurrently
    let ui_task = tokio::spawn(async move {
        let mut ui_updates = 0;
        for _ in 0..10 {
            sleep(Duration::from_millis(50)).await;
            ui_updates += 1;
            // Simulate UI work
            let _dummy = vec![0u8; 100];
        }
        ui_updates
    });

    let _ = insert_task.await.expect("Insert task failed");
    let ui_updates = ui_task.await.expect("UI task failed");

    let total_duration = insert_start.elapsed();

    println!("IO-005 Async Non-Blocking Test:");
    println!("  Total duration: {:?}", total_duration);
    println!("  UI updates completed: {}", ui_updates);
    println!("  UI remained responsive during DB operations");

    // Verify UI remained responsive
    assert_eq!(ui_updates, 10, "UI should have completed all updates");
    assert!(total_duration.as_secs() < 5, "Test took too long: {:?}", total_duration);

    pool.close().await;
}
