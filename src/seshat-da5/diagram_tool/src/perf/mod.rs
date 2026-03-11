//! Performance Baseline Module for Seshat Diagram Tool
//!
//! This module provides FPS measurement utilities, benchmark harnesses,
//! and performance regression testing infrastructure for 3000-node diagrams.
//!
//! ## Target: 120 FPS (8.33ms frame time)
//!
//! ## Design by Contract
//!
//! - **P1**: `NodeCount` is 1-10000 (validated at construction)
//! - **P2**: `DurationMs` >= 100 (minimum measurement window)
//! - **P3**: Warm-up iterations complete before measurement
//! - **P4**: Measurement environment is isolated
//! - **P5**: Sample rate >= 60 Hz (Nyquist compliance)
//!
//! ## Invariants
//!
//! - **INV-1**: No NaN/Infinity in measurements
//! - **INV-2**: Timestamps are monotonic
//! - **INV-3**: Frame time and FPS are reciprocally consistent
//! - **INV-4**: Sample count matches actual samples
//! - **INV-5**: Percentiles are ordered (p50 <= p90 <= p95 <= p99)

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(unused_imports, dead_code)]

mod benchmark;
mod error;
mod fps;
mod harness;
mod metrics;
mod regression;

pub use benchmark::{Benchmark, BenchmarkConfig, BenchmarkResult, NodeCount};
pub use error::PerfError;
pub use fps::{FpsMeasurement, FpsReport};
pub use harness::{generate_test_scene, Baseline, BenchmarkHarness, Operation};
pub use metrics::{FrameSample, Percentiles, Statistics};
pub use regression::{RegressionResult, RegressionTest};

/// Minimum valid node count for benchmarks
pub const MIN_NODE_COUNT: u32 = 1;

/// Maximum valid node count for benchmarks
pub const MAX_NODE_COUNT: u32 = 10_000;

/// Target FPS for all operations
pub const TARGET_FPS: f64 = 120.0;

/// Minimum acceptable FPS (regression threshold)
pub const MIN_ACCEPTABLE_FPS: f64 = 100.0;

/// Minimum benchmark duration in milliseconds
pub const MIN_DURATION_MS: u64 = 100;

/// Default warm-up iterations
pub const DEFAULT_WARMUP_ITERATIONS: u32 = 3;

/// Default benchmark duration in milliseconds
pub const DEFAULT_BENCHMARK_DURATION_MS: u64 = 5000;

/// Node count for baseline benchmarks
pub const BASELINE_NODE_COUNT: u32 = 3000;
