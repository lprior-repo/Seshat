//! Benchmark configuration and execution.
//!
//! Refactored for DDD:
//! - config: Domain types for inputs
//! - result: Domain types for outputs
//! - runner: The execution engine

mod config;
mod result;
mod runner;

pub use config::{BenchmarkConfig, DurationMs, NodeCount, WarmupConfig};
pub use result::BenchmarkResult;
pub use runner::Benchmark;
