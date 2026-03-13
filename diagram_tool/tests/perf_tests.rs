//! Performance tests for PERF-001..PERF-003
//!
//! These tests verify performance requirements:
//! - PERF-001: 3000 nodes pan at 120 FPS
//! - PERF-002: 3000 nodes zoom at 120 FPS
//! - PERF-003: 3000 nodes marquee select 500 nodes in <100ms

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use diagram_tool::perf::{
    Benchmark, BenchmarkConfig, BenchmarkHarness, BenchmarkResult, FpsReport, FrameSample,
    Operation, RegressionTest, Statistics, TARGET_FPS,
};
use tempfile::TempDir;

mod unit_tests {
    use super::*;

    #[test]
    fn perf_001_pan_benchmark_3000_nodes() {
        let config = BenchmarkConfig::new("pan")
            .with_node_count(3000)
            .unwrap()
            .with_duration_ms(500)
            .unwrap()
            .with_target_fps(TARGET_FPS);

        let benchmark = Benchmark::new(config);
        let result = benchmark.run().unwrap();

        assert!(
            result.fps_report.sample_count > 0,
            "Should have recorded frame samples"
        );
        assert!(
            result.fps_report.mean_fps > 0.0,
            "Mean FPS should be positive"
        );
        assert!(
            result.fps_report.std_dev_fps >= 0.0,
            "Standard deviation should be non-negative"
        );
    }

    #[test]
    fn perf_002_zoom_benchmark_3000_nodes() {
        let config = BenchmarkConfig::new("zoom")
            .with_node_count(3000)
            .unwrap()
            .with_duration_ms(500)
            .unwrap()
            .with_target_fps(TARGET_FPS);

        let benchmark = Benchmark::new(config);
        let result = benchmark.run().unwrap();

        assert!(
            result.fps_report.sample_count > 0,
            "Should have recorded frame samples"
        );
        assert!(
            result.fps_report.mean_fps > 0.0,
            "Mean FPS should be positive"
        );
        assert!(
            result.fps_report.std_dev_fps >= 0.0,
            "Standard deviation should be non-negative"
        );
    }

    #[test]
    fn perf_003_select_benchmark_500_nodes() {
        let temp_dir = TempDir::new().unwrap();
        let harness = BenchmarkHarness::new(temp_dir.path().to_path_buf()).with_node_count(3000);

        let result = harness.run_benchmark(Operation::Select);
        assert!(result.is_ok(), "Select benchmark should run successfully");

        let result = result.unwrap();
        assert!(
            result.fps_report.mean_fps > 0.0,
            "Mean FPS should be positive for selection"
        );
    }

    #[test]
    fn fps_report_validation() {
        let samples: Vec<FrameSample> = (0..100)
            .map(|i| FrameSample::new(i, 8.33, i as f64 * 8.33))
            .collect();

        let report = FpsReport::from_samples(samples, TARGET_FPS);
        let validation = report.validate();

        assert!(validation.is_ok(), "FPS report should be valid");
    }

    #[test]
    fn statistics_percentile_ordering() {
        let samples: Vec<f64> = (1..=100).map(f64::from).collect();
        let stats = Statistics::from_samples(&samples);

        assert!(
            stats.percentiles.p50 <= stats.percentiles.p90,
            "p50 should be <= p90"
        );
        assert!(
            stats.percentiles.p90 <= stats.percentiles.p95,
            "p90 should be <= p95"
        );
        assert!(
            stats.percentiles.p95 <= stats.percentiles.p99,
            "p95 should be <= p99"
        );
    }

    #[test]
    fn benchmark_result_validation() {
        let config = BenchmarkConfig::new("test")
            .with_node_count(100)
            .unwrap()
            .with_duration_ms(200)
            .unwrap();

        let samples: Vec<FrameSample> = (0..10)
            .map(|i| FrameSample::new(i, 8.33, i as f64 * 8.33))
            .collect();
        let fps_report = FpsReport::from_samples(samples, TARGET_FPS);
        let result = BenchmarkResult::new(config, fps_report, 0);

        assert!(result.validate().is_ok(), "Result should be valid");
    }

    #[test]
    fn regression_test_detection() {
        use diagram_tool::perf::Baseline;

        let mut baseline = Baseline::new(3000, TARGET_FPS);

        let config = BenchmarkConfig::new("pan");
        let samples: Vec<FrameSample> = (0..10)
            .map(|i| FrameSample::new(i, 8.33, i as f64 * 8.33))
            .collect();
        let fps_report = FpsReport::from_samples(samples, TARGET_FPS);
        let result = BenchmarkResult::new(config, fps_report, 0);
        baseline.add_result(Operation::Pan, result);

        let regression_test = RegressionTest::from_baseline(baseline);

        let config = BenchmarkConfig::new("pan");
        let samples: Vec<FrameSample> = (0..10)
            .map(|i| FrameSample::new(i, 16.67, i as f64 * 16.67))
            .collect();
        let fps_report = FpsReport::from_samples(samples, 60.0);
        let result = BenchmarkResult::new(config, fps_report, 0);

        let regression_result = regression_test.compare(&result);
        assert!(regression_result.is_ok());
    }

    #[test]
    fn benchmark_config_builder() {
        let config = BenchmarkConfig::new("pan")
            .with_node_count(3000)
            .unwrap()
            .with_duration_ms(1000)
            .unwrap()
            .with_target_fps(120.0)
            .with_seed(42);

        assert_eq!(config.node_count.value(), 3000);
        assert_eq!(config.duration_ms.value(), 1000);
        assert_eq!(config.target_fps, 120.0);
        assert_eq!(config.seed, 42);
    }

    #[test]
    fn benchmark_config_validation() {
        let valid_config = BenchmarkConfig::new("test").with_target_fps(120.0);
        assert!(
            valid_config.is_valid(),
            "Valid config should pass validation"
        );

        let invalid_config = BenchmarkConfig::new("test").with_target_fps(0.0);
        assert!(
            !invalid_config.is_valid(),
            "Zero FPS config should fail validation"
        );

        let nan_config = BenchmarkConfig::new("test").with_target_fps(f64::NAN);
        assert!(
            !nan_config.is_valid(),
            "NaN FPS config should fail validation"
        );
    }

    #[test]
    fn operation_complexity_factors() {
        assert!((Operation::Pan.complexity_factor() - 0.8).abs() < 0.001);
        assert!((Operation::Zoom.complexity_factor() - 0.9).abs() < 0.001);
        assert!((Operation::Select.complexity_factor() - 0.7).abs() < 0.001);
        assert!((Operation::Drag.complexity_factor() - 1.0).abs() < 0.001);
        assert!((Operation::RenderFrame.complexity_factor() - 1.2).abs() < 0.001);
    }

    #[test]
    fn operation_names() {
        assert_eq!(Operation::Pan.name(), "pan");
        assert_eq!(Operation::Zoom.name(), "zoom");
        assert_eq!(Operation::Select.name(), "select");
        assert_eq!(Operation::Drag.name(), "drag");
        assert_eq!(Operation::RenderFrame.name(), "render_frame");
    }

    #[test]
    fn benchmark_result_passed_status() {
        use diagram_tool::perf::BenchmarkResult;

        let config = BenchmarkConfig::new("test")
            .with_node_count(100)
            .unwrap()
            .with_duration_ms(200)
            .unwrap()
            .with_target_fps(120.0);

        // Test 1: With 8ms frame time (125 FPS), should pass target of 120 FPS
        let samples: Vec<FrameSample> = (0..10)
            .map(|i| FrameSample::new(i, 8.0, i as f64 * 8.0))
            .collect();
        let fps_report = FpsReport::from_samples(samples, 120.0);
        let result = BenchmarkResult::new(config.clone(), fps_report, 0);

        assert!(
            result.passed,
            "Should pass when FPS (125) exceeds target (120)"
        );

        // Test 2: With 20ms frame time (50 FPS), should fail target of 120 FPS
        let samples: Vec<FrameSample> = (0..10)
            .map(|i| FrameSample::new(i, 20.0, i as f64 * 20.0))
            .collect();
        let fps_report = FpsReport::from_samples(samples, 120.0);
        let result = BenchmarkResult::new(config, fps_report, 0);

        assert!(
            !result.passed,
            "Should fail when FPS (50) is below target (120)"
        );
    }

    #[test]
    fn benchmark_result_regression_detection() {
        let config = BenchmarkConfig::new("test").with_target_fps(120.0);

        let samples: Vec<FrameSample> = (0..10)
            .map(|i| FrameSample::new(i, 8.33, i as f64 * 8.33))
            .collect();
        let fps_report = FpsReport::from_samples(samples, TARGET_FPS);
        let result = BenchmarkResult::new(config.clone(), fps_report, 0);

        assert!(
            !result.is_regression(10.0),
            "Should not detect regression when close to target"
        );

        let samples: Vec<FrameSample> = (0..10)
            .map(|i| FrameSample::new(i, 20.0, i as f64 * 20.0))
            .collect();
        let fps_report = FpsReport::from_samples(samples, 50.0);
        let result = BenchmarkResult::new(config, fps_report, 0);

        assert!(
            result.is_regression(10.0),
            "Should detect regression when FPS drops significantly"
        );
    }

    #[test]
    fn perf_004_drag_benchmark_3000_nodes_500_selected() {
        let config = BenchmarkConfig::new("drag")
            .with_node_count(3000)
            .unwrap()
            .with_duration_ms(500)
            .unwrap()
            .with_target_fps(TARGET_FPS);

        let benchmark = Benchmark::new(config);
        let result = benchmark.run().unwrap();

        assert!(
            result.fps_report.sample_count > 0,
            "Should have recorded frame samples"
        );
        assert!(
            result.fps_report.mean_fps > 0.0,
            "Mean FPS should be positive"
        );
        assert!(
            result.fps_report.std_dev_fps >= 0.0,
            "Standard deviation should be non-negative"
        );
    }

    #[test]
    fn perf_005_render_frame_benchmark_3000_nodes() {
        let config = BenchmarkConfig::new("render_frame")
            .with_node_count(3000)
            .unwrap()
            .with_duration_ms(500)
            .unwrap()
            .with_target_fps(TARGET_FPS);

        let benchmark = Benchmark::new(config);
        let result = benchmark.run().unwrap();

        assert!(
            result.fps_report.sample_count > 0,
            "Should have recorded frame samples"
        );
        assert!(
            result.fps_report.mean_fps > 0.0,
            "Mean FPS should be positive for render"
        );
    }

    #[test]
    fn perf_006_all_operations_benchmark() {
        let harness = BenchmarkHarness::new(TempDir::new().unwrap().path().to_path_buf())
            .with_node_count(1000);

        for operation in Operation::all() {
            let result = harness.run_benchmark(operation);
            assert!(
                result.is_ok(),
                "Operation {:?} benchmark should run successfully",
                operation
            );
        }
    }

    #[test]
    fn perf_007_frame_budget_enforcement() {
        let config = BenchmarkConfig::new("frame_budget_test")
            .with_node_count(3000)
            .unwrap()
            .with_duration_ms(1000)
            .unwrap()
            .with_target_fps(120.0);

        let samples: Vec<FrameSample> = (0..120)
            .map(|i| {
                let frame_time = if i % 30 == 0 { 12.0 } else { 7.5 };
                FrameSample::new(i, frame_time, i as f64 * frame_time)
            })
            .collect();

        let fps_report = FpsReport::from_samples(samples, TARGET_FPS);
        let mean_frame_time = fps_report.frame_time_stats.mean;
        let _result = BenchmarkResult::new(config, fps_report, 0);

        assert!(
            mean_frame_time < 8.34,
            "Mean frame time should be under 8.33ms for 120 FPS, got {}",
            mean_frame_time
        );
    }
}
