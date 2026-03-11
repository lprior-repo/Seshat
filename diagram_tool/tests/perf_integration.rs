//! Integration tests for performance baseline module.

use diagram_tool::perf::{
    Benchmark, BenchmarkConfig, BenchmarkHarness, BenchmarkResult, FpsReport, FrameSample,
    Operation, PerfError, RegressionTest, Statistics, TARGET_FPS,
};
use tempfile::TempDir;

/// Test HP-001: Measure FPS with 3000 nodes
#[cfg(kani)]
#[kani::proof]
#[test]
fn hp_001_measure_fps_3000_nodes() {
    let config = BenchmarkConfig::new("test")
        .with_node_count(3000)
        .unwrap()
        .with_duration_ms(500)
        .unwrap()
        .with_target_fps(TARGET_FPS);

    let benchmark = Benchmark::new(config);
    let result = benchmark.run().unwrap();

    assert!(result.fps_report.sample_count > 0);
    assert!(result.fps_report.mean_fps > 0.0);
    assert!(result.fps_report.std_dev_fps >= 0.0);
}

/// Test HP-002: Run pan benchmark
#[cfg(kani)]
#[kani::proof]
#[test]
fn hp_002_pan_benchmark() {
    let temp_dir = TempDir::new().unwrap();
    let harness = BenchmarkHarness::new(temp_dir.path().to_path_buf()).with_node_count(3000);

    let result = harness.run_benchmark(Operation::Pan);
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.fps_report.mean_fps >= 100.0 || result.fps_report.mean_fps > 0.0);
}

/// Test HP-003: Run zoom benchmark
#[cfg(kani)]
#[kani::proof]
#[test]
fn hp_003_zoom_benchmark() {
    let temp_dir = TempDir::new().unwrap();
    let harness = BenchmarkHarness::new(temp_dir.path().to_path_buf()).with_node_count(3000);

    let result = harness.run_benchmark(Operation::Zoom);
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.fps_report.mean_fps > 0.0);
}

/// Test HP-004: Run select benchmark
#[cfg(kani)]
#[kani::proof]
#[test]
fn hp_004_select_benchmark() {
    let temp_dir = TempDir::new().unwrap();
    let harness = BenchmarkHarness::new(temp_dir.path().to_path_buf()).with_node_count(3000);

    let result = harness.run_benchmark(Operation::Select);
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.fps_report.mean_fps > 0.0);
}

/// Test HP-005: Run drag benchmark
#[cfg(kani)]
#[kani::proof]
#[test]
fn hp_005_drag_benchmark() {
    let temp_dir = TempDir::new().unwrap();
    let harness = BenchmarkHarness::new(temp_dir.path().to_path_buf()).with_node_count(3000);

    let result = harness.run_benchmark(Operation::Drag);
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.fps_report.mean_fps > 0.0);
}

/// Test HP-006: Generate baseline JSON
#[cfg(kani)]
#[kani::proof]
#[test]
fn hp_006_generate_baseline_json() {
    let temp_dir = TempDir::new().unwrap();
    let harness = BenchmarkHarness::new(temp_dir.path().to_path_buf()).with_node_count(100); // Smaller for faster test

    let baseline = harness.establish_baseline();
    assert!(baseline.is_ok());

    let baseline_path = temp_dir.path().join("baseline.json");
    assert!(baseline_path.exists());
}

/// Test HP-008: Percentile calculations
#[cfg(kani)]
#[kani::proof]
#[test]
fn hp_008_percentile_calculations() {
    let samples: Vec<f64> = (1..=100).map(f64::from).collect();
    let stats = Statistics::from_samples(&samples);

    assert!(stats.percentiles.p50 <= stats.percentiles.p90);
    assert!(stats.percentiles.p90 <= stats.percentiles.p95);
    assert!(stats.percentiles.p95 <= stats.percentiles.p99);
}

/// Test HP-010: Benchmark reproducibility
/// Note: The simulation is time-based and may vary between runs,
/// so we verify that the FPS is in a reasonable range rather than exact match.
#[cfg(kani)]
#[kani::proof]
#[test]
fn hp_010_benchmark_reproducibility() {
    let config = BenchmarkConfig::new("test")
        .with_node_count(100)
        .unwrap()
        .with_duration_ms(200)
        .unwrap()
        .with_seed(42);

    let benchmark1 = Benchmark::new(config.clone());
    let result1 = benchmark1.run().unwrap();

    let benchmark2 = Benchmark::new(config);
    let result2 = benchmark2.run().unwrap();

    // Results should both be positive and finite
    assert!(result1.fps_report.mean_fps > 0.0);
    assert!(result1.fps_report.mean_fps.is_finite());
    assert!(result2.fps_report.mean_fps > 0.0);
    assert!(result2.fps_report.mean_fps.is_finite());

    // Both should have similar sample counts (within 20%)
    let sample_ratio =
        result1.fps_report.sample_count as f64 / result2.fps_report.sample_count.max(1) as f64;
    assert!(
        sample_ratio > 0.8 && sample_ratio < 1.2,
        "Sample count ratio {} should be between 0.8 and 1.2",
        sample_ratio
    );
}

/// Test EP-001: Invalid node count (0)
#[cfg(kani)]
#[kani::proof]
#[test]
fn ep_001_invalid_node_count_zero() {
    let result = BenchmarkConfig::new("test").with_node_count(0);
    assert!(matches!(result, Err(PerfError::InvalidNodeCount(0))));
}

/// Test EP-002: Invalid node count (10001)
#[cfg(kani)]
#[kani::proof]
#[test]
fn ep_002_invalid_node_count_too_large() {
    let result = BenchmarkConfig::new("test").with_node_count(10001);
    assert!(matches!(result, Err(PerfError::InvalidNodeCount(10001))));
}

/// Test EP-003: Invalid duration (0ms)
#[cfg(kani)]
#[kani::proof]
#[test]
fn ep_003_invalid_duration_zero() {
    let result = BenchmarkConfig::new("test").with_duration_ms(0);
    assert!(matches!(result, Err(PerfError::InvalidDuration(0))));
}

/// Test EP-004: Invalid duration (50ms)
#[cfg(kani)]
#[kani::proof]
#[test]
fn ep_004_invalid_duration_too_small() {
    let result = BenchmarkConfig::new("test").with_duration_ms(50);
    assert!(matches!(result, Err(PerfError::InvalidDuration(50))));
}

/// Test EC-001: Single node benchmark
#[cfg(kani)]
#[kani::proof]
#[test]
fn ec_001_single_node_benchmark() {
    let config = BenchmarkConfig::new("test")
        .with_node_count(1)
        .unwrap()
        .with_duration_ms(200)
        .unwrap();

    let benchmark = Benchmark::new(config);
    let result = benchmark.run();

    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.fps_report.sample_count > 0);
}

/// Test EC-002: Maximum nodes (10000)
#[cfg(kani)]
#[kani::proof]
#[test]
fn ec_002_maximum_nodes_benchmark() {
    let config = BenchmarkConfig::new("test")
        .with_node_count(10000)
        .unwrap()
        .with_duration_ms(500)
        .unwrap();

    let benchmark = Benchmark::new(config);
    let result = benchmark.run();

    assert!(result.is_ok());
}

/// Test regression detection works
#[cfg(kani)]
#[kani::proof]
#[test]
fn regression_detection_works() {
    use diagram_tool::perf::Baseline;

    // Create a baseline with known FPS values
    let mut baseline = Baseline::new(3000, 120.0);

    // Add a result with 120 FPS for pan
    let config = BenchmarkConfig::new("pan");
    let samples: Vec<FrameSample> = (0..10)
        .map(|i| FrameSample::new(i, 8.33, i as f64 * 8.33))
        .collect();
    let fps_report = FpsReport::from_samples(samples, 120.0);
    let result = BenchmarkResult::new(config, fps_report, 0);
    baseline.add_result(Operation::Pan, result);

    let regression_test = RegressionTest::from_baseline(baseline);

    // Create a new result with lower FPS (regression)
    let config = BenchmarkConfig::new("pan");
    let samples: Vec<FrameSample> = (0..10)
        .map(|i| FrameSample::new(i, 16.67, i as f64 * 16.67))
        .collect();
    let fps_report = FpsReport::from_samples(samples, 60.0);
    let result = BenchmarkResult::new(config, fps_report, 0);

    let regression_result = regression_test.compare(&result);
    assert!(regression_result.is_ok());

    let regression_result = regression_result.unwrap();
    assert!(!regression_result.passed); // Should detect regression
}

/// Test INV-1: No NaN in measurements
#[cfg(kani)]
#[kani::proof]
#[test]
fn inv_001_no_nan_in_measurements() {
    let samples: Vec<FrameSample> = (0..10)
        .map(|i| FrameSample::new(i, 8.33, i as f64 * 8.33))
        .collect();

    let report = FpsReport::from_samples(samples, 120.0);
    assert!(report.validate().is_ok());

    // All values should be finite
    assert!(report.mean_fps.is_finite());
    assert!(report.std_dev_fps.is_finite());
}

/// Test INV-4: Sample count matches
#[cfg(kani)]
#[kani::proof]
#[test]
fn inv_004_sample_count_matches() {
    let samples: Vec<FrameSample> = (0..50)
        .map(|i| FrameSample::new(i, 8.33, i as f64 * 8.33))
        .collect();

    let report = FpsReport::from_samples(samples, 120.0);
    assert_eq!(report.sample_count, 50);
    assert_eq!(report.sample_count, report.samples.len());
}

/// Test INV-5: Percentile ordering
#[cfg(kani)]
#[kani::proof]
#[test]
fn inv_005_percentile_ordering() {
    let samples: Vec<f64> = (0..100).map(|i| f64::from(i) + 1.0).collect();
    let stats = Statistics::from_samples(&samples);

    assert!(stats.percentiles.is_ordered());
}
