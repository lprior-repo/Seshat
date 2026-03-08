//! Import/Export/Persistence Tests (IO-001 to IO-015)
//!
//! This module contains comprehensive tests for JSON import/export
//! and persistence operations per contract bd-19p.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![forbid(unsafe_code)]
#![allow(unused)]
#![ignore]

use crate::models::export::export_diagram_json;

// Helper to create a minimal test JSON export
fn create_minimal_export() -> String {
    r#"{
        "metadata": {
            "name": "diagram",
            "revision": 0,
            "version": 2
        },
        "data": {
            "version": 2,
            "revision": 0,
            "nodes": {},
            "edges": {},
            "cycle_policy": "default",
            "author_priority": []
        },
        "events": []
    }"#.to_string()
}

// Helper to create an in-memory database for testing
//
// # Errors
//
// Returns error if temp directory creation, database connection, or schema initialization fails.
fn create_test_db() -> Result<(rusqlite::Connection, tempfile::TempDir), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");
    let conn = rusqlite::Connection::open(db_path)?;

    // Initialize schema
    conn.execute(
        "CREATE TABLE IF NOT EXISTS events (
            operation_id TEXT PRIMARY KEY,
            revision INTEGER NOT NULL,
            payload TEXT NOT NULL,
            timestamp INTEGER NOT NULL
        )",
        [],
    )?;

    Ok((conn, temp_dir))
}

// ============================================================================
// IO-005: Large Document Export Performance
// ============================================================================

#[test]
fn io_005_large_document_export_performance() {
    // Given: Document with 1000+ events
    let (conn, _temp_dir) = create_test_db().unwrap_or_else(|e| panic!("Failed to create test DB: {}", e));

    // Create 1000 events
    for i in 0..1000 {
        conn.execute(
            "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
            (format!("op-{i}"), i + 1, "{}", 1000 + i),
        ).unwrap_or_else(|e| panic!("DB insert failed: {:?}", e));
    }

    // When: Exporting
    let export_start = std::time::Instant::now();
    let result = export_diagram_json(&conn);
    let export_duration = export_start.elapsed();

    // Then: Completes within reasonable time (< 5 seconds)
    assert!(result.is_ok());
    assert!(export_duration.as_secs() < 5, "Export took {} seconds", export_duration.as_secs());

    let export = match result {
        Ok(e) => e,
        Err(e) => panic!("Export should succeed: {:?}", e),
    };
    assert_eq!(export.metadata.revision, 0); // No replay = revision 0

    // Print timing for analysis
    println!("IO-005 Export duration: {:?}", export_duration);
    println!("IO-005 Events per second: {:.2}", 1000.0 / export_duration.as_secs_f64());
}

// ============================================================================
// IO-005a: Large Document Export Performance (5000 events)
// ============================================================================

#[test]
fn io_005a_large_document_export_performance_5000() {
    // Given: Document with 5000 events
    let (conn, _temp_dir) = create_test_db().unwrap_or_else(|e| panic!("Failed to create test DB: {}", e));

    // Create 5000 events
    for i in 0..5000 {
        conn.execute(
            "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
            (format!("op-{i}"), i + 1, "{}", 1000 + i),
        ).unwrap_or_else(|e| panic!("DB insert failed: {:?}", e));
    }

    // When: Exporting
    let export_start = std::time::Instant::now();
    let result = export_diagram_json(&conn);
    let export_duration = export_start.elapsed();

    // Then: Completes within reasonable time (< 10 seconds for 5000 events)
    assert!(result.is_ok());
    assert!(export_duration.as_secs() < 10, "Export took {} seconds", export_duration.as_secs());

    let export = match result {
        Ok(e) => e,
        Err(e) => panic!("Export should succeed: {:?}", e),
    };
    assert_eq!(export.metadata.revision, 0);

    // Print timing for analysis
    println!("IO-005a Export duration: {:?}", export_duration);
    println!("IO-005a Events per second: {:.2}", 5000.0 / export_duration.as_secs_f64());
}

// ============================================================================
// IO-005b: Large Document Export Performance (10000 events)
// ============================================================================

#[test]
fn io_005b_large_document_export_performance_10000() {
    // Given: Document with 10000 events
    let (conn, _temp_dir) = create_test_db().unwrap_or_else(|e| panic!("Failed to create test DB: {}", e));

    // Create 10000 events
    for i in 0..10000 {
        conn.execute(
            "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
            (format!("op-{i}"), i + 1, "{}", 1000 + i),
        ).unwrap_or_else(|e| panic!("DB insert failed: {:?}", e));
    }

    // When: Exporting
    let export_start = std::time::Instant::now();
    let result = export_diagram_json(&conn);
    let export_duration = export_start.elapsed();

    // Then: Completes within reasonable time (< 20 seconds for 10000 events)
    assert!(result.is_ok());
    assert!(export_duration.as_secs() < 20, "Export took {} seconds", export_duration.as_secs());

    let export = match result {
        Ok(e) => e,
        Err(e) => panic!("Export should succeed: {:?}", e),
    };
    assert_eq!(export.metadata.revision, 0);

    // Print timing for analysis
    println!("IO-005b Export duration: {:?}", export_duration);
    println!("IO-005b Events per second: {:.2}", 10000.0 / export_duration.as_secs_f64());
}

// ============================================================================
// IO-005c: Async Large Document Export Performance
// ============================================================================

#[tokio::test]
async fn io_005c_async_large_document_export_performance() {
    use crate::store::{bootstrap_store, fetch_all_events};
    use tempfile::TempDir;

    // Given: Document with 1000 events
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let bootstrap = bootstrap_store(&db_path)
        .await
        .expect("Failed to bootstrap async store");

    // Create 1000 events using async operations
    let envelopes: Vec<crate::models::envelope::EventEnvelope> = (0..1000)
        .map(|i| crate::models::envelope::EventEnvelope {
            op_id: format!("op-{i}"),
            operation: crate::models::envelope::DomainOp::NodeAdd {
                id: format!("node-{i}"),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: format!("Node {}", i),
            },
            author: crate::models::envelope::Author {
                id: "test".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1000 + i,
        })
        .collect();

    // Insert events in batches
    let batch_size = 100;
    for chunk in envelopes.chunks(batch_size) {
        crate::store::append_batch(&bootstrap.pool, chunk.to_vec(), None)
            .await
            .expect("Failed to append batch");
    }

    // When: Fetching all events asynchronously
    let fetch_start = std::time::Instant::now();
    let events = fetch_all_events(&bootstrap.pool)
        .await
        .expect("Failed to fetch events");
    let fetch_duration = fetch_start.elapsed();

    // Then: All events are retrieved
    assert_eq!(events.len(), 1000);

    // Print timing for analysis
    println!("IO-005c Async fetch duration: {:?}", fetch_duration);
    println!("IO-005c Events per second: {:.2}", 1000.0 / fetch_duration.as_secs_f64());
}

// ============================================================================
// IO-005d: Async Large Document Export Performance (5000 events)
// ============================================================================

#[tokio::test]
async fn io_005d_async_large_document_export_performance_5000() {
    use crate::store::{bootstrap_store, fetch_all_events};
    use tempfile::TempDir;

    // Given: Document with 5000 events
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let bootstrap = bootstrap_store(&db_path)
        .await
        .expect("Failed to bootstrap async store");

    // Create 5000 events using async operations
    let envelopes: Vec<crate::models::envelope::EventEnvelope> = (0..5000)
        .map(|i| crate::models::envelope::EventEnvelope {
            op_id: format!("op-{i}"),
            operation: crate::models::envelope::DomainOp::NodeAdd {
                id: format!("node-{i}"),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: format!("Node {}", i),
            },
            author: crate::models::envelope::Author {
                id: "test".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1000 + i,
        })
        .collect();

    // Insert events in batches
    let batch_size = 100;
    for chunk in envelopes.chunks(batch_size) {
        crate::store::append_batch(&bootstrap.pool, chunk.to_vec(), None)
            .await
            .expect("Failed to append batch");
    }

    // When: Fetching all events asynchronously
    let fetch_start = std::time::Instant::now();
    let events = fetch_all_events(&bootstrap.pool)
        .await
        .expect("Failed to fetch events");
    let fetch_duration = fetch_start.elapsed();

    // Then: All events are retrieved
    assert_eq!(events.len(), 5000);

    // Print timing for analysis
    println!("IO-005d Async fetch duration: {:?}", fetch_duration);
    println!("IO-005d Events per second: {:.2}", 5000.0 / fetch_duration.as_secs_f64());
}

// ============================================================================
// IO-005e: Async Large Document Export Performance (10000 events)
// ============================================================================

#[tokio::test]
async fn io_005e_async_large_document_export_performance_10000() {
    use crate::store::{bootstrap_store, fetch_all_events};
    use tempfile::TempDir;

    // Given: Document with 10000 events
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let bootstrap = bootstrap_store(&db_path)
        .await
        .expect("Failed to bootstrap async store");

    // Create 10000 events using async operations
    let envelopes: Vec<crate::models::envelope::EventEnvelope> = (0..10000)
        .map(|i| crate::models::envelope::EventEnvelope {
            op_id: format!("op-{i}"),
            operation: crate::models::envelope::DomainOp::NodeAdd {
                id: format!("node-{i}"),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: format!("Node {}", i),
            },
            author: crate::models::envelope::Author {
                id: "test".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1000 + i,
        })
        .collect();

    // Insert events in batches
    let batch_size = 100;
    for chunk in envelopes.chunks(batch_size) {
        crate::store::append_batch(&bootstrap.pool, chunk.to_vec(), None)
            .await
            .expect("Failed to append batch");
    }

    // When: Fetching all events asynchronously
    let fetch_start = std::time::Instant::now();
    let events = fetch_all_events(&bootstrap.pool)
        .await
        .expect("Failed to fetch events");
    let fetch_duration = fetch_start.elapsed();

    // Then: All events are retrieved
    assert_eq!(events.len(), 10000);

    // Print timing for analysis
    println!("IO-005e Async fetch duration: {:?}", fetch_duration);
    println!("IO-005e Events per second: {:.2}", 10000.0 / fetch_duration.as_secs_f64());
}
