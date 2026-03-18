//! Benchmark harness for diagram operations.

mod baseline;
mod core;
mod driver;
mod operation;
mod scene;

pub use baseline::Baseline;
pub use core::BenchmarkHarness;
pub use driver::PerformanceDriver;
pub use operation::Operation;
pub use scene::generate_test_scene;
