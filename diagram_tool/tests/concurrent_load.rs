//! Concurrent Load Tests for Async Database Operations
//!
//! This test module measures the performance and behavior of the async SQLite
//! implementation under concurrent load. Tests are designed to:
//!
//! - Validate true concurrency (multiple operations running simultaneously)
//! - Measure throughput (operations per second)
//! - Track latency percentiles (p50, p95, p99)
//! - Monitor connection pool usage
//! - Record error rates under stress
//!
//! # Test Scenarios
//!
//! - **10 concurrent append operations**: Light concurrent write load
//! - **50 concurrent read operations**: Read-heavy concurrent load
//! - **Mixed read/write workload**: Realistic concurrent access pattern
//! - **100+ concurrent operations**: Stress test for pool limits and contention
//!
//! # Running the Tests
//!
//! ```bash
//! cargo test --test concurrent_load --features async-db -- --nocapture --test-threads=1
//! ```
//!
//! Or run a specific scenario:
//! ```bash
//! cargo test concurrent_append_light --features async-db -- --nocapture
//! ```

#![allow(clippy::expect_used)] // Test code uses expect for clarity
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use diagram_tool::models::envelope::{Author, DomainOp, EventEnvelope};
use diagram_tool::store_async::{
    append_event_async, append_idempotent_async,
    fetch_all_events, fetch_events_since,
    AsyncStoreError, EventRecord,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::sync::Semaphore;

// Re-export sqlx for use in tests
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

/// Test configuration for load scenarios
#[derive(Debug, Clone)]
struct LoadTestConfig {
    /// Number of concurrent operations
    concurrency: usize,
    /// Number of operations per task
    ops_per_task: usize,
    /// Connection pool size
    pool_size: u32,
}

impl Default for LoadTestConfig {
    fn default() -> Self {
        Self {
            concurrency: 10,
            ops_per_task: 10,
            pool_size: 10,
        }
    }
}

/// Statistics collected during a load test
#[derive(Debug, Clone)]
struct LoadTestStats {
    /// Total operations attempted
    total_ops: usize,
    /// Successful operations
    successful_ops: usize,
    /// Failed operations
    failed_ops: usize,
    /// Total elapsed time
    elapsed: Duration,
    /// Individual operation latencies (microseconds)
    latencies_us: Vec<u128>,
    /// Operations per second
    ops_per_sec: f64,
    /// Error rate (percentage)
    error_rate: f64,
    /// 50th percentile latency (microseconds)
    p50_us: u128,
    /// 95th percentile latency (microseconds)
    p95_us: u128,
    /// 99th percentile latency (microseconds)
    p99_us: u128,
    /// Maximum latency (microseconds)
    max_us: u128,
    /// Minimum latency (microseconds)
    min_us: u128,
}

impl LoadTestStats {
    fn calculate(mut self) -> Self {
        if !self.latencies_us.is_empty() {
            self.latencies_us.sort_unstable();
            let len = self.latencies_us.len();

            self.min_us = *self.latencies_us.first().expect("latencies should have first element");
            self.max_us = *self.latencies_us.last().expect("latencies should have last element");

            // Calculate percentiles
            self.p50_us = Self::percentile(&self.latencies_us, 50);
            self.p95_us = Self::percentile(&self.latencies_us, 95);
            self.p99_us = Self::percentile(&self.latencies_us, 99);
        }

        // Calculate throughput
        let elapsed_secs = self.elapsed.as_secs_f64();
        if elapsed_secs > 0.0 {
            self.ops_per_sec = self.successful_ops as f64 / elapsed_secs;
        }

        // Calculate error rate
        if self.total_ops > 0 {
            self.error_rate = (self.failed_ops as f64 / self.total_ops as f64) * 100.0;
        }

        self
    }

    fn percentile(sorted: &[u128], p: usize) -> u128 {
        if sorted.is_empty() {
            return 0;
        }
        let index = (p * sorted.len() / 100).saturating_sub(1);
        sorted.get(index).copied().unwrap_or(0)
    }

    fn print_summary(&self, scenario_name: &str) {
        println!("\n===== {} =====", scenario_name);
        println!("Total operations:  {}", self.total_ops);
        println!("Successful:        {}", self.successful_ops);
        println!("Failed:            {}", self.failed_ops);
        println!("Error rate:        {:.2}%", self.error_rate);
        println!("Elapsed time:      {:?}", self.elapsed);
        println!("Throughput:        {:.2} ops/sec", self.ops_per_sec);
        println!("\nLatency (microseconds):");
        println!("  Min:  {:>8} us", self.min_us);
        println!("  P50:  {:>8} us", self.p50_us);
        println!("  P95:  {:>8} us", self.p95_us);
        println!("  P99:  {:>8} us", self.p99_us);
        println!("  Max:  {:>8} us", self.max_us);
        println!("========================\n");
    }
}

/// Helper to create a test event envelope
fn create_test_envelope(op_id: String, timestamp: i64) -> EventEnvelope {
    EventEnvelope {
        op_id,
        operation: DomainOp::NodeAdd {
            id: format!("node-{}", timestamp),
            x: 10.0 + (timestamp as f64 % 1000.0),
            y: 20.0 + (timestamp as f64 % 1000.0),
            width: 100.0,
            height: 50.0,
            label: format!("Test Node {}", timestamp),
        },
        author: Author {
            id: "test-user".to_string(),
            name: "Test User".to_string(),
            email: None,
        },
        timestamp,
    }
}

/// Setup a test database with the given pool size
async fn setup_test_db(pool_size: u32) -> Result<(TempDir, PathBuf, SqlitePool), AsyncStoreError> {
    let temp_dir = TempDir::new().map_err(|e| {
        AsyncStoreError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to create temp dir: {}", e),
        ))
    })?;

    let db_path = temp_dir.path().join("load_test.db");

    // Create pool with specified size
    let connection_string = format!("sqlite:{}?mode=rwc", db_path.display());
    let pool = SqlitePoolOptions::new()
        .max_connections(pool_size)
        .connect(&connection_string)
        .await?;

    // Run schema migration
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER NOT NULL DEFAULT 1
        )"
    )
    .execute(&pool)
    .await?;

    sqlx::query("INSERT OR IGNORE INTO schema_version (version) VALUES (1)")
        .execute(&pool)
        .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            operation_id TEXT NOT NULL UNIQUE,
            revision INTEGER NOT NULL,
            payload TEXT NOT NULL,
            timestamp TEXT NOT NULL
        )"
    )
    .execute(&pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_events_revision ON events(revision)")
        .execute(&pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_events_operation_id ON events(operation_id)")
        .execute(&pool)
        .await?;

    Ok((temp_dir, db_path, pool))
}

/// Test scenario: 10 concurrent append operations (light write load)
#[tokio::test]
async fn concurrent_append_light() {
    let config = LoadTestConfig {
        concurrency: 10,
        ops_per_task: 10,
        pool_size: 10,
    };

    let (_temp_dir, _db_path, pool) = setup_test_db(config.pool_size).await
        .expect("Failed to setup test database");
    let pool: Arc<SqlitePool> = Arc::new(pool);

    let start = Instant::now();
    let mut tasks = Vec::new();
    let mut latencies = Vec::new();
    let mut successful = 0usize;
    let mut failed = 0usize;

    // Spawn concurrent append tasks
    for task_id in 0..config.concurrency {
        let pool_clone = pool.clone();
        let task = tokio::spawn(async move {
            let mut task_lats = Vec::new();
            let mut task_success = 0usize;
            let mut task_failed = 0usize;

            for i in 0..config.ops_per_task {
                let op_id = format!("op-{}-{}", task_id, i);
                let envelope = create_test_envelope(op_id, 1700000000 + (task_id * 1000 + i) as i64);

                let op_start = Instant::now();
                let result = append_event_async(&pool_clone, envelope, None).await;
                let latency = op_start.elapsed().as_micros();
                task_lats.push(latency);

                match result {
                    Ok(_) => task_success += 1,
                    Err(_) => task_failed += 1,
                }
            }

            (task_lats, task_success, task_failed)
        });
        tasks.push(task);
    }

    // Collect results
    for task in tasks {
        let (task_lats, task_success, task_failed) = task.await
            .expect("Failed to join task");
        latencies.extend(task_lats);
        successful += task_success;
        failed += task_failed;
    }

    let elapsed = start.elapsed();

    let stats = LoadTestStats {
        total_ops: config.concurrency * config.ops_per_task,
        successful_ops: successful,
        failed_ops: failed,
        elapsed,
        latencies_us: latencies,
        ops_per_sec: 0.0,
        error_rate: 0.0,
        p50_us: 0,
        p95_us: 0,
        p99_us: 0,
        max_us: 0,
        min_us: 0,
    }.calculate();

    stats.print_summary("Concurrent Append (10 concurrent, 10 ops each)");

    // Assertions for validation
    assert_eq!(stats.failed_ops, 0, "Should have no failed operations");
    assert!(stats.ops_per_sec > 0.0, "Should have positive throughput");

    // Pool should handle this load easily
    assert!(stats.p95_us < 10_000_000, "P95 latency should be under 10 seconds");

    // Close pool
    SqlitePool::close(&pool).await;
}

/// Test scenario: 50 concurrent read operations (read-heavy load)
#[tokio::test]
async fn concurrent_read_heavy() {
    let config = LoadTestConfig {
        concurrency: 50,
        ops_per_task: 20,
        pool_size: 10,
    };

    let (_temp_dir, _db_path, pool) = setup_test_db(config.pool_size).await
        .expect("Failed to setup test database");
    let pool: Arc<SqlitePool> = Arc::new(pool);

    // Pre-populate with some data
    for i in 0..100 {
        let envelope = create_test_envelope(format!("preload-op-{}", i), 1700000000 + i as i64);
        append_event_async(&pool, envelope, None).await
            .expect("Failed to preload data");
    }

    let start = Instant::now();
    let mut tasks = Vec::new();
    let mut latencies = Vec::new();
    let mut successful = 0usize;
    let mut failed = 0usize;

    // Spawn concurrent read tasks
    for task_id in 0..config.concurrency {
        let pool_clone = pool.clone();
        let task = tokio::spawn(async move {
            let mut task_lats = Vec::new();
            let mut task_success = 0usize;
            let mut task_failed = 0usize;

            for i in 0..config.ops_per_task {
                // Mix different read operations
                let op_start = Instant::now();
                let result = if i % 3 == 0 {
                    // Fetch all events
                    fetch_all_events(&pool_clone).await
                } else if i % 3 == 1 {
                    // Fetch since revision
                    fetch_events_since(&pool_clone, (task_id % 10) as i64).await
                } else {
                    // Fetch since revision 0
                    fetch_events_since(&pool_clone, 0).await
                };
                let latency = op_start.elapsed().as_micros();
                task_lats.push(latency);

                match result {
                    Ok(_) => task_success += 1,
                    Err(_) => task_failed += 1,
                }
            }

            (task_lats, task_success, task_failed)
        });
        tasks.push(task);
    }

    // Collect results
    for task in tasks {
        let (task_lats, task_success, task_failed) = task.await
            .expect("Failed to join task");
        latencies.extend(task_lats);
        successful += task_success;
        failed += task_failed;
    }

    let elapsed = start.elapsed();

    let stats = LoadTestStats {
        total_ops: config.concurrency * config.ops_per_task,
        successful_ops: successful,
        failed_ops: failed,
        elapsed,
        latencies_us: latencies,
        ops_per_sec: 0.0,
        error_rate: 0.0,
        p50_us: 0,
        p95_us: 0,
        p99_us: 0,
        max_us: 0,
        min_us: 0,
    }.calculate();

    stats.print_summary("Concurrent Read (50 concurrent, 20 ops each)");

    // Assertions
    assert_eq!(stats.failed_ops, 0, "Should have no failed operations");
    assert!(stats.ops_per_sec > 0.0, "Should have positive throughput");

    // Reads should be fast due to WAL mode allowing concurrent readers
    assert!(stats.p95_us < 5_000_000, "P95 read latency should be under 5 seconds");

    // Close pool
    SqlitePool::close(&pool).await;
}

/// Test scenario: Mixed read/write workload
#[tokio::test]
async fn concurrent_mixed_workload() {
    let config = LoadTestConfig {
        concurrency: 20,
        ops_per_task: 25,
        pool_size: 10,
    };

    let (_temp_dir, _db_path, pool) = setup_test_db(config.pool_size).await
        .expect("Failed to setup test database");
    let pool: Arc<SqlitePool> = Arc::new(pool);

    // Pre-populate with some data
    for i in 0..50 {
        let envelope = create_test_envelope(format!("preload-op-{}", i), 1700000000 + i as i64);
        append_event_async(&pool, envelope, None).await
            .expect("Failed to preload data");
    }

    let start = Instant::now();
    let mut tasks = Vec::new();
    let mut latencies = Vec::new();
    let mut successful = 0usize;
    let mut failed = 0usize;

    // Spawn mixed workload tasks
    for task_id in 0..config.concurrency {
        let pool_clone = pool.clone();
        let task = tokio::spawn(async move {
            let mut task_lats = Vec::new();
            let mut task_success = 0usize;
            let mut task_failed = 0usize;

            for i in 0..config.ops_per_task {
                // 60% reads, 40% writes
                let is_write = (task_id + i) % 5 < 2;

                let op_start = Instant::now();
                let result = if is_write {
                    let op_id = format!("mixed-op-{}-{}", task_id, i);
                    let envelope = create_test_envelope(op_id, 1700000000 + (task_id * 1000 + i) as i64);
                    append_event_async(&pool_clone, envelope, None).await
                } else {
                    // For reads, we convert the result to a consistent type
                    fetch_events_since(&pool_clone, 0).await.map(|_| ())
                };
                let latency = op_start.elapsed().as_micros();
                task_lats.push(latency);

                match result {
                    Ok(_) => task_success += 1,
                    Err(_) => task_failed += 1,
                }
            }

            (task_lats, task_success, task_failed)
        });
        tasks.push(task);
    }

    // Collect results
    for task in tasks {
        let (task_lats, task_success, task_failed) = task.await
            .expect("Failed to join task");
        latencies.extend(task_lats);
        successful += task_success;
        failed += task_failed;
    }

    let elapsed = start.elapsed();

    let stats = LoadTestStats {
        total_ops: config.concurrency * config.ops_per_task,
        successful_ops: successful,
        failed_ops: failed,
        elapsed,
        latencies_us: latencies,
        ops_per_sec: 0.0,
        error_rate: 0.0,
        p50_us: 0,
        p95_us: 0,
        p99_us: 0,
        max_us: 0,
        min_us: 0,
    }.calculate();

    stats.print_summary("Mixed Workload (20 concurrent, 25 ops each, 60% read/40% write)");

    // Allow some write conflicts due to concurrent writes
    let failure_tolerance = (config.ops_per_task * config.concurrency / 10) as f64; // 10% tolerance
    assert!(
        stats.failed_ops <= failure_tolerance as usize,
        "Failed operations {} should be within tolerance {}",
        stats.failed_ops,
        failure_tolerance
    );
    assert!(stats.ops_per_sec > 0.0, "Should have positive throughput");

    // Close pool
    SqlitePool::close(&pool).await;
}

/// Test scenario: Stress test with 100+ concurrent operations
#[tokio::test]
async fn concurrent_stress_test() {
    let config = LoadTestConfig {
        concurrency: 150, // Exceeds pool size to test pool behavior
        ops_per_task: 5,
        pool_size: 10,
    };

    let (_temp_dir, _db_path, pool) = setup_test_db(config.pool_size).await
        .expect("Failed to setup test database");
    let pool: Arc<SqlitePool> = Arc::new(pool);

    // Pre-populate with some data
    for i in 0..100 {
        let envelope = create_test_envelope(format!("preload-op-{}", i), 1700000000 + i as i64);
        append_event_async(&pool, envelope, None).await
            .expect("Failed to preload data");
    }

    let start = Instant::now();
    let mut tasks = Vec::new();
    let mut latencies = Vec::new();
    let mut successful = 0usize;
    let mut failed = 0usize;

    // Use a semaphore to control actual concurrency beyond pool limits
    let semaphore = Arc::new(Semaphore::new(config.concurrency));

    // Spawn many concurrent tasks
    for task_id in 0..config.concurrency {
        let permit = semaphore.clone().acquire_owned().await
            .expect("Failed to acquire semaphore permit");
        let pool_clone = pool.clone();
        let task = tokio::spawn(async move {
            let mut task_lats = Vec::new();
            let mut task_success = 0usize;
            let mut task_failed = 0usize;

            for i in 0..config.ops_per_task {
                let op_id = format!("stress-op-{}-{}", task_id, i);
                let envelope = create_test_envelope(op_id, 1700000000 + (task_id * 1000 + i) as i64);

                let op_start = Instant::now();
                let result = append_event_async(&pool_clone, envelope, None).await;
                let latency = op_start.elapsed().as_micros();
                task_lats.push(latency);

                match result {
                    Ok(_) => task_success += 1,
                    Err(_) => task_failed += 1,
                }
            }

            drop(permit);
            (task_lats, task_success, task_failed)
        });
        tasks.push(task);
    }

    // Collect results
    for task in tasks {
        let (task_lats, task_success, task_failed) = task.await
            .expect("Failed to join task");
        latencies.extend(task_lats);
        successful += task_success;
        failed += task_failed;
    }

    let elapsed = start.elapsed();

    let stats = LoadTestStats {
        total_ops: config.concurrency * config.ops_per_task,
        successful_ops: successful,
        failed_ops: failed,
        elapsed,
        latencies_us: latencies,
        ops_per_sec: 0.0,
        error_rate: 0.0,
        p50_us: 0,
        p95_us: 0,
        p99_us: 0,
        max_us: 0,
        min_us: 0,
    }.calculate();

    stats.print_summary("Stress Test (150 concurrent tasks, pool size 10)");

    // Under stress, we expect some delays but system should remain functional
    assert!(stats.successful_ops > 0, "Should have at least some successful operations");
    assert!(stats.ops_per_sec > 0.0, "Should have positive throughput");

    // Pool contention should increase P99 latency significantly
    println!("Note: P99 latency reflects pool contention with {} tasks competing for {} connections",
             config.concurrency, config.pool_size);

    // Close pool
    SqlitePool::close(&pool).await;
}

/// Test scenario: Idempotent append with concurrent duplicates
#[tokio::test]
async fn concurrent_idempotent_operations() {
    let config = LoadTestConfig {
        concurrency: 20,
        ops_per_task: 10,
        pool_size: 10,
    };

    let (_temp_dir, _db_path, pool) = setup_test_db(config.pool_size).await
        .expect("Failed to setup test database");
    let pool: Arc<SqlitePool> = Arc::new(pool);

    let start = Instant::now();
    let mut tasks = Vec::new();
    let mut latencies = Vec::new();
    let mut successful = 0usize;
    let mut failed = 0usize;

    // Use shared op_ids to create duplicates
    let shared_op_ids: Vec<String> = (0..10).map(|i| format!("shared-op-{}", i)).collect();

    // Spawn concurrent idempotent append tasks
    for task_id in 0..config.concurrency {
        let pool_clone = pool.clone();
        let op_ids = shared_op_ids.clone();
        let task = tokio::spawn(async move {
            let mut task_lats = Vec::new();
            let mut task_success = 0usize;
            let mut task_failed = 0usize;

            for i in 0..config.ops_per_task {
                // Use shared op_ids to create intentional duplicates
                let op_id = op_ids[i % op_ids.len()].clone();
                let envelope = create_test_envelope(
                    op_id.clone(),
                    1700000000 + (task_id * 1000 + i) as i64
                );

                let op_start = Instant::now();
                let result = append_idempotent_async(&pool_clone, envelope).await;
                let latency = op_start.elapsed().as_micros();
                task_lats.push(latency);

                match result {
                    Ok(_) => task_success += 1,
                    Err(_) => task_failed += 1,
                }
            }

            (task_lats, task_success, task_failed)
        });
        tasks.push(task);
    }

    // Collect results
    for task in tasks {
        let (task_lats, task_success, task_failed) = task.await
            .expect("Failed to join task");
        latencies.extend(task_lats);
        successful += task_success;
        failed += task_failed;
    }

    let elapsed = start.elapsed();

    let stats = LoadTestStats {
        total_ops: config.concurrency * config.ops_per_task,
        successful_ops: successful,
        failed_ops: failed,
        elapsed,
        latencies_us: latencies,
        ops_per_sec: 0.0,
        error_rate: 0.0,
        p50_us: 0,
        p95_us: 0,
        p99_us: 0,
        max_us: 0,
        min_us: 0,
    }.calculate();

    stats.print_summary("Idempotent Operations (20 concurrent, 10 shared op_ids)");

    // Idempotent operations should succeed (returning existing for duplicates)
    assert_eq!(stats.failed_ops, 0, "Idempotent operations should not fail");
    assert!(stats.ops_per_sec > 0.0, "Should have positive throughput");

    // Verify final database state has exactly 10 unique operations
    let final_events = fetch_all_events(&pool).await
        .expect("Failed to fetch final events");
    assert_eq!(final_events.len(), 10, "Should have exactly 10 unique events");

    // Close pool
    SqlitePool::close(&pool).await;
}

/// Test scenario: Pool size benchmark - compare different pool sizes
#[tokio::test]
async fn pool_size_benchmark() {
    let pool_sizes = vec![1, 5, 10, 20];
    let concurrency = 30;
    let ops_per_task = 10;

    println!("\n===== Pool Size Benchmark =====");
    println!("Testing with {} concurrent tasks, {} ops per task", concurrency, ops_per_task);

    let mut results: HashMap<u32, f64> = HashMap::new();

    for pool_size in pool_sizes {
        let (_temp_dir, _db_path, pool) = setup_test_db(pool_size).await
            .expect("Failed to setup test database");
        let pool: Arc<SqlitePool> = Arc::new(pool);

        let start = Instant::now();
        let mut tasks = Vec::new();

        // Spawn concurrent tasks
        for task_id in 0..concurrency {
            let pool_clone = pool.clone();
            let task = tokio::spawn(async move {
                let mut task_success = 0usize;

                for i in 0..ops_per_task {
                    let op_id = format!("bench-{}-{}-{}", pool_size, task_id, i);
                    let envelope = create_test_envelope(
                        op_id,
                        1700000000 + (task_id * 1000 + i) as i64
                    );

                    if append_event_async(&pool_clone, envelope, None).await.is_ok() {
                        task_success += 1;
                    }
                }

                task_success
            });
            tasks.push(task);
        }

        let mut successful = 0usize;
        for task in tasks {
            successful += task.await
                .expect("Failed to join task");
        }

        let elapsed = start.elapsed();
        let ops_per_sec = successful as f64 / elapsed.as_secs_f64();

        results.insert(pool_size, ops_per_sec);

        println!("Pool size {:>2}: {:.2} ops/sec ({}/{})",
                 pool_size, ops_per_sec, successful, concurrency * ops_per_task);

        // Close pool
        SqlitePool::close(&pool).await;
    }

    println!("\nAnalysis:");
    let best_pool = results.iter()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(k, v)| (*k, *v));

    if let Some((size, throughput)) = best_pool {
        println!("Best pool size: {} with {:.2} ops/sec", size, throughput);
    }

    // Calculate improvement from min to max pool size
    let min_throughput = results.values().cloned().fold(f64::INFINITY, f64::min);
    let max_throughput = results.values().cloned().fold(f64::NEG_INFINITY, f64::max);
    if min_throughput > 0.0 {
        let improvement = ((max_throughput - min_throughput) / min_throughput) * 100.0;
        println!("Improvement from smallest to largest pool: {:.1}%", improvement);
    }

    println!("================================\n");
}

/// Test scenario: Latency under sustained load
#[tokio::test]
async fn sustained_load_latency() {
    let config = LoadTestConfig {
        concurrency: 15,
        ops_per_task: 50, // Higher ops per task for sustained load
        pool_size: 10,
    };

    let (_temp_dir, _db_path, pool) = setup_test_db(config.pool_size).await
        .expect("Failed to setup test database");
    let pool: Arc<SqlitePool> = Arc::new(pool);

    let start = Instant::now();
    let mut tasks = Vec::new();
    let mut latencies = Vec::new();
    let mut successful = 0usize;
    let mut failed = 0usize;

    // Track latencies over time buckets
    let mut time_buckets: Vec<Vec<u128>> = vec![Vec::new(); 5];

    // Spawn concurrent tasks
    for task_id in 0..config.concurrency {
        let pool_clone = pool.clone();
        let task = tokio::spawn(async move {
            let mut task_lats = Vec::new();
            let mut task_success = 0usize;
            let mut task_failed = 0usize;

            for i in 0..config.ops_per_task {
                let op_id = format!("sustained-op-{}-{}", task_id, i);
                let envelope = create_test_envelope(
                    op_id,
                    1700000000 + (task_id * 1000 + i) as i64
                );

                let op_start = Instant::now();
                let result = append_event_async(&pool_clone, envelope, None).await;
                let latency = op_start.elapsed().as_micros();

                // Track time bucket (20% intervals)
                let bucket = (i * 5 / config.ops_per_task).min(4);
                task_lats.push((latency, bucket));

                match result {
                    Ok(_) => task_success += 1,
                    Err(_) => task_failed += 1,
                }
            }

            (task_lats, task_success, task_failed)
        });
        tasks.push(task);
    }

    // Collect results
    for task in tasks {
        let (task_lats, task_success, task_failed) = task.await
            .expect("Failed to join task");
        for (latency, bucket) in task_lats {
            latencies.push(latency);
            time_buckets[bucket].push(latency);
        }
        successful += task_success;
        failed += task_failed;
    }

    let elapsed = start.elapsed();

    let stats = LoadTestStats {
        total_ops: config.concurrency * config.ops_per_task,
        successful_ops: successful,
        failed_ops: failed,
        elapsed,
        latencies_us: latencies.clone(),
        ops_per_sec: 0.0,
        error_rate: 0.0,
        p50_us: 0,
        p95_us: 0,
        p99_us: 0,
        max_us: 0,
        min_us: 0,
    }.calculate();

    stats.print_summary("Sustained Load (15 concurrent, 50 ops each)");

    // Analyze latency over time
    println!("\nLatency progression over time (median per bucket):");
    for (i, bucket) in time_buckets.iter().enumerate() {
        if !bucket.is_empty() {
            let mut sorted = bucket.clone();
            sorted.sort_unstable();
            let median = sorted[bucket.len() / 2];
            let percentile_20 = sorted[bucket.len() / 5];
            let percentile_80 = sorted[bucket.len() * 4 / 5];
            println!("  Bucket {} ({:>3}%): P50={:>8}us, P20={:>8}us, P80={:>8}us",
                     i, (i + 1) * 20, median, percentile_20, percentile_80);
        }
    }

    // Close pool
    SqlitePool::close(&pool).await;
}
