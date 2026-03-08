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
