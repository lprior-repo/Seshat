// Performance benchmarks comparing sync (rusqlite) vs async (sqlx) database operations
//
// This benchmark suite measures:
// - Single event append (append_event)
// - Batch event append (append_batch)
// - Current revision read
// - Fetch events since (query with filter)
//
// Tests are run with realistic data sizes: 100, 1000, and 10000 events

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::store::{
    append_event as sync_append_event, append_batch as sync_append_batch,
    bootstrap_store as sync_bootstrap_store, current_revision as sync_current_revision,
    BatchAppendResult, EventEnvelope as SyncEventEnvelope,
};
use crate::store_async::{
    append_event_async as async_append_event,
    append_batch_async as async_append_batch,
    bootstrap_async_store as async_bootstrap_store,
    current_revision as async_current_revision,
    fetch_events_since as async_fetch_events_since,
    AsyncAppendResult, AsyncBatchAppendResult, EventEnvelope as AsyncEventEnvelope,
};
use criterion::{
    black_box, BenchmarkGroup, BenchmarkId, Criterion, Throughput, measurement::WallTime,
};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

// Note: The async and sync EventEnvelope are actually the same type, but from different modules
// We need to convert between them for benchmarking

// Generate a test event envelope
fn make_envelope(op_id: String, timestamp: i64) -> SyncEventEnvelope {
    SyncEventEnvelope {
        op_id,
        operation: diagram_tool::models::envelope::DomainOp::NodeAdd {
            id: format!("node-{}", timestamp),
            x: (timestamp % 1000) as f64,
            y: ((timestamp / 1000) % 1000) as f64,
            width: 100.0,
            height: 50.0,
            label: format!("Test Node {}", timestamp),
        },
        author: diagram_tool::models::envelope::Author {
            id: "user-test".to_string(),
            name: "Test User".to_string(),
            email: None,
        },
        timestamp,
    }
}

// Convert sync envelope to async envelope
fn to_async_envelope(env: SyncEventEnvelope) -> AsyncEventEnvelope {
    AsyncEventEnvelope {
        op_id: env.op_id,
        operation: env.operation,
        author: env.author,
        timestamp: env.timestamp,
    }
}

// Setup for sync benchmarks
struct SyncBenchmarkSetup {
    _temp_dir: TempDir,
    db_path: PathBuf,
}

fn setup_sync_benchmark() -> SyncBenchmarkSetup {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("bench.db");
    sync_bootstrap_store(&db_path).expect("Failed to bootstrap sync store");
    SyncBenchmarkSetup {
        _temp_dir: temp_dir,
        db_path,
    }

}

// Setup for async benchmarks
struct AsyncBenchmarkSetup {
    _temp_dir: TempDir,
    db_path: PathBuf,
}

fn setup_async_benchmark() -> AsyncBenchmarkSetup {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("bench.db");

    // Create a runtime for the async setup
    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
    rt.block_on(async {
        async_bootstrap_store(&db_path)
            .await
            .expect("Failed to bootstrap async store");
    });

    AsyncBenchmarkSetup {
        _temp_dir: temp_dir,
        db_path,
    }
}

// Populate database with N events (sync)
fn populate_sync_db(db_path: &Path, count: usize) {
    let mut bootstrap = sync_bootstrap_store(db_path).expect("Failed to bootstrap");
    let conn = &mut bootstrap.conn;

    let envelopes: Vec<_> = (0..count as i64)
        .map(|i| make_envelope(format!("op-{}", i), 1_700_000_000 + i))
        .collect();

    // Batch insert for speed
    sync_append_batch(conn, envelopes, None).expect("Failed to populate db");
}

// Populate database with N events (async)
fn populate_async_db(db_path: &Path, count: usize) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
    rt.block_on(async {
        let bootstrap = async_bootstrap_store(db_path)
            .await
            .expect("Failed to bootstrap async store");
        let pool = &bootstrap.pool;

        let envelopes: Vec<_> = (0..count as i64)
            .map(|i| to_async_envelope(make_envelope(format!("op-{}", i), 1_700_000_000 + i)))
            .collect();

        async_append_batch(pool, envelopes, None)
            .await
            .expect("Failed to populate async db");
    });
}

// Benchmark: Single append event
fn bench_append_event(c: &mut Criterion) {
    let mut group = c.benchmark_group("append_event");

    // Baseline: empty database
    group.bench_function("sync/empty", |b| {
        let setup = setup_sync_benchmark();
        b.to_owned(|| {
            let bootstrap = sync_bootstrap_store(&setup.db_path).expect("Failed to bootstrap");
            let conn = &mut bootstrap.conn;
            let env = make_envelope("test-op".to_string(), 1_700_000_000);
            black_box(sync_append_event(conn, env, None).unwrap())
        });
    });

    group.bench_function("async/empty", |b| {
        let setup = setup_async_benchmark();
        let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
        b.to_owned(|| {
            rt.block_on(async {
                let bootstrap = async_bootstrap_store(&setup.db_path)
                    .await
                    .expect("Failed to bootstrap");
                let pool = &bootstrap.pool;
                let env = to_async_envelope(make_envelope("test-op".to_string(), 1_700_000_000));
                black_box(async_append_event(pool, env, None).await.unwrap())
            });
        });
    });

    // With existing data
    for size in [100, 1000, 10000] {
        let sync_setup = setup_sync_benchmark();
        populate_sync_db(&sync_setup.db_path, size);

        group.bench_with_input(BenchmarkId::new("sync", size), &size, |b, _| {
            b.to_owned(|| {
                let bootstrap = sync_bootstrap_store(&sync_setup.db_path).expect("Failed to bootstrap");
                let conn = &mut bootstrap.conn;
                let env = make_envelope(format!("new-op-{}", uuid::Uuid::new_v4()), 1_700_000_000);
                black_box(sync_append_event(conn, env, None).unwrap())
            });
        });

        let async_setup = setup_async_benchmark();
        populate_async_db(&async_setup.db_path, size);

        group.bench_with_input(BenchmarkId::new("async", size), &size, |b, _| {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            b.to_owned(|| {
                rt.block_on(async {
                    let bootstrap = async_bootstrap_store(&async_setup.db_path)
                        .await
                        .expect("Failed to bootstrap");
                    let pool = &bootstrap.pool;
                    let env = to_async_envelope(
                        make_envelope(format!("new-op-{}", uuid::Uuid::new_v4()), 1_700_000_000)
                    );
                    black_box(async_append_event(pool, env, None).await.unwrap())
                });
            });
        });
    }

    group.finish();
}

// Benchmark: Batch append with varying batch sizes
fn bench_append_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("append_batch");

    for batch_size in [10, 50, 100, 500, 1000] {
        group.throughput(Throughput::Elements(batch_size as u64));

        let sync_setup = setup_sync_benchmark();

        group.bench_with_input(BenchmarkId::new("sync", batch_size), &batch_size, |b, &size| {
            let envelopes: Vec<_> = (0..size)
                .map(|i| make_envelope(format!("batch-op-{}", i), 1_700_000_000 + i as i64))
                .collect();

            b.to_owned(|| {
                let bootstrap = sync_bootstrap_store(&sync_setup.db_path).expect("Failed to bootstrap");
                let conn = &mut bootstrap.conn;
                // Clone envelopes for each iteration
                let envelopes = envelopes.clone();
                black_box(sync_append_batch(conn, envelopes, None).unwrap())
            });
        });

        let async_setup = setup_async_benchmark();

        group.bench_with_input(BenchmarkId::new("async", batch_size), &batch_size, |b, &size| {
            let envelopes: Vec<_> = (0..size)
                .map(|i| to_async_envelope(
                    make_envelope(format!("batch-op-{}", i), 1_700_000_000 + i as i64)
                ))
                .collect();

            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            b.to_owned(|| {
                rt.block_on(async {
                    let bootstrap = async_bootstrap_store(&async_setup.db_path)
                        .await
                        .expect("Failed to bootstrap");
                    let pool = &bootstrap.pool;
                    let envelopes = envelopes.clone();
                    black_box(async_append_batch(pool, envelopes, None).await.unwrap())
                });
            });
        });
    }

    group.finish();
}

// Benchmark: Current revision read
fn bench_current_revision(c: &mut Criterion) {
    let mut group = c.benchmark_group("current_revision");

    for size in [100, 1000, 10000] {
        let sync_setup = setup_sync_benchmark();
        populate_sync_db(&sync_setup.db_path, size);

        group.bench_with_input(BenchmarkId::new("sync", size), &size, |b, _| {
            let bootstrap = sync_bootstrap_store(&sync_setup.db_path).expect("Failed to bootstrap");
            let conn = &bootstrap.conn;
            b.iter(|| black_box(sync_current_revision(conn).unwrap()));
        });

        let async_setup = setup_async_benchmark();
        populate_async_db(&async_setup.db_path, size);

        group.bench_with_input(BenchmarkId::new("async", size), &size, |b, _| {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            let bootstrap = rt.block_on(async {
                async_bootstrap_store(&async_setup.db_path)
                    .await
                    .expect("Failed to bootstrap")
            });
            let pool = &bootstrap.pool;
            b.iter(|| {
                rt.block_on(async {
                    black_box(async_current_revision(pool).await.unwrap())
                })
            });
        });
    }

    group.finish();
}

// Benchmark: Fetch events since (query with filter)
fn bench_fetch_events_since(c: &mut Criterion) {
    let mut group = c.benchmark_group("fetch_events_since");

    for (db_size, fetch_size) in [
        (100, 10),
        (100, 50),
        (1000, 100),
        (1000, 500),
        (10000, 1000),
    ] {
        let sync_setup = setup_sync_benchmark();
        populate_sync_db(&sync_setup.db_path, db_size);
        let after_revision = (db_size - fetch_size) as i64;

        group.bench_with_input(
            BenchmarkId::new("sync", format!("{}_from_{}", db_size, fetch_size)),
            &(db_size, fetch_size),
            |b, _| {
                let bootstrap = sync_bootstrap_store(&sync_setup.db_path).expect("Failed to bootstrap");
                let conn = &bootstrap.conn;
                b.iter(|| {
                    black_box(
                        diagram_tool::models::sync::fetch_new_events(conn, after_revision).unwrap()
                    )
                });
            },
        );

        let async_setup = setup_async_benchmark();
        populate_async_db(&async_setup.db_path, db_size);

        group.bench_with_input(
            BenchmarkId::new("async", format!("{}_from_{}", db_size, fetch_size)),
            &(db_size, fetch_size),
            |b, _| {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                let bootstrap = rt.block_on(async {
                    async_bootstrap_store(&async_setup.db_path)
                        .await
                        .expect("Failed to bootstrap")
                });
                let pool = &bootstrap.pool;
                b.iter(|| {
                    rt.block_on(async {
                        black_box(
                            async_fetch_events_since(pool, after_revision)
                                .await
                                .unwrap()
                        )
                    })
                });
            },
        );
    }

    group.finish();
}

// Benchmark: Read throughput (events/second)
fn bench_read_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("read_throughput");
    group.throughput(Throughput::Elements(1));

    for db_size in [100, 1000, 10000] {
        let sync_setup = setup_sync_benchmark();
        populate_sync_db(&sync_setup.db_path, db_size);

        group.bench_with_input(BenchmarkId::new("sync", db_size), &db_size, |b, _| {
            let bootstrap = sync_bootstrap_store(&sync_setup.db_path).expect("Failed to bootstrap");
            let conn = &bootstrap.conn;
            b.iter(|| {
                black_box(
                    diagram_tool::models::sync::fetch_new_events(conn, 0).unwrap()
                        .len()
                )
            });
        });

        let async_setup = setup_async_benchmark();
        populate_async_db(&async_setup.db_path, db_size);

        group.bench_with_input(BenchmarkId::new("async", db_size), &db_size, |b, _| {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            let bootstrap = rt.block_on(async {
                async_bootstrap_store(&async_setup.db_path)
                    .await
                    .expect("Failed to bootstrap")
            });
            let pool = &bootstrap.pool;
            b.iter(|| {
                rt.block_on(async {
                    black_box(async_fetch_events_since(pool, 0).await.unwrap().len())
                })
            });
        });
    }

    group.finish();
}

// Criterion entry point
criterion_group!(
    benches,
    bench_append_event,
    bench_append_batch,
    bench_current_revision,
    bench_fetch_events_since,
    bench_read_throughput
);

criterion_main!(benches);
