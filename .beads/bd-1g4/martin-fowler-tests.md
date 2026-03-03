bead_id: bd-1g4
bead_title: perf-baseline: Establish 3000-node performance benchmarks (120 FPS target)
phase: p0
updated_at: 2026-03-03T00:00:00Z

# Martin Fowler Test Plan: Performance Baseline

## Test Categories

### 1. Happy Path Tests

| Test ID | Description | Verification |
|---------|-------------|--------------|
| HP-001 | Measure FPS with 3000 nodes | Returns valid FpsReport with mean, std_dev, samples |
| HP-002 | Run pan benchmark | Completes within timeout, FPS >= 100 |
| HP-003 | Run zoom benchmark | Completes within timeout, FPS >= 100 |
| HP-004 | Run select benchmark | Completes within timeout, FPS >= 100 |
| HP-005 | Run drag benchmark | Completes within timeout, FPS >= 100 |
| HP-006 | Generate baseline JSON | Creates valid JSON at target/perf/baseline.json |
| HP-007 | Run regression test against baseline | Returns RegressionResult::Passed |
| HP-008 | Percentile calculations | P50 < P90 < P95 < P99 |
| HP-009 | Confidence interval computation | 95% CI bounds are valid |
| HP-010 | Benchmark reproducibility | Same seed produces same results (+/- 5 FPS) |

### 2. Error Path Tests

| Test ID | Description | Expected Error |
|---------|-------------|----------------|
| EP-001 | Invalid node count (0) | PerfError::InvalidNodeCount |
| EP-002 | Invalid node count (10001) | PerfError::InvalidNodeCount |
| EP-003 | Invalid duration (0ms) | PerfError::InvalidDuration |
| EP-004 | Invalid duration (50ms) | PerfError::InvalidDuration |
| EP-005 | Baseline file not found | PerfError::BaselineNotFound |
| EP-006 | Insufficient samples collected | PerfError::InsufficientSamples |
| EP-007 | Benchmark timeout exceeded | PerfError::Timeout |
| EP-008 | Environment not suitable | PerfError::Environment |

### 3. Edge Case Tests

| Test ID | Description | Verification |
|---------|-------------|---------------|
| EC-001 | Single node benchmark | Completes successfully |
| EC-002 | Maximum nodes (10000) | Completes or graceful degradation |
| EC-003 | Empty scene (0 nodes) | Returns meaningful result or error |
| EC-004 | Very long benchmark (60s) | Produces valid statistics |
| EC-005 | Rapid consecutive measurements | No state contamination |
| EC-006 | Concurrent benchmark requests | Returns error or queues properly |
| EC-007 | Extreme zoom levels (0.01x, 100x) | No NaN/Infinity in results |
| EC-008 | All nodes at same position | No division by zero |
| EC-009 | Negative coordinates | Handles gracefully |
| EC-010 | Very large coordinates (1e9) | No overflow |

### 4. Contract Violation Tests

| Test ID | Contract | Violation | Expected Result |
|---------|----------|-----------|-----------------|
| CV-001 | INV-1 | Inject NaN into measurements | PerfError::InvariantViolation |
| CV-002 | INV-1 | Inject Infinity into measurements | PerfError::InvariantViolation |
| CV-003 | INV-2 | Non-monotonic timestamps | PerfError::InvariantViolation |
| CV-004 | INV-3 | Inconsistent frame time/FPS | PerfError::InvariantViolation |
| CV-005 | INV-4 | Sample count mismatch | PerfError::InvariantViolation |
| CV-006 | INV-5 | Unordered percentiles | PerfError::InvariantViolation |
| CV-007 | P1 | NodeCount outside bounds | Compile-time or runtime error |
| CV-008 | P2 | Duration below minimum | PerfError::InvalidDuration |

### 5. Performance Stress Tests

| Test ID | Description | Target |
|---------|-------------|--------|
| ST-001 | 3000-node pan operation | >= 120 FPS |
| ST-002 | 3000-node zoom operation | >= 120 FPS |
| ST-003 | 3000-node select operation | >= 120 FPS |
| ST-004 | 3000-node drag operation | >= 120 FPS |
| ST-005 | 3000-node frame render | >= 120 FPS |
| ST-006 | Memory usage at 3000 nodes | < 500MB |
| ST-007 | Benchmark overhead | < 1% of measured time |

## Test Implementation Strategy

### Unit Tests (diagram_tool/src/perf/tests.rs)

```rust
#[cfg(test)]
mod tests {
    // Happy Path
    #[test]
    fn hp_001_measure_fps_3000_nodes() { ... }

    #[test]
    fn hp_002_pan_benchmark() { ... }

    // Error Path
    #[test]
    fn ep_001_invalid_node_count_zero() { ... }

    // Edge Cases
    #[test]
    fn ec_001_single_node_benchmark() { ... }

    // Contract Violations
    #[test]
    #[should_panic(expected = "InvariantViolation")]
    fn cv_001_nan_in_measurements() { ... }
}
```

### Integration Tests (diagram_tool/tests/perf_integration.rs)

```rust
#[test]
fn integration_3000_node_baseline() {
    // Full benchmark suite with real document
}

#[test]
fn regression_detection_works() {
    // Verify regression detection against known baseline
}
```

### Property-Based Tests (using proptest)

```rust
proptest! {
    #[test]
    fn fps_frame_time_reciprocal(fps in 1.0f64..1000.0) {
        // INV-3: Verify frame_time = 1000/fps
    }

    #[test]
    fn percentiles_ordered(samples in prop::collection::vec(0.1f64..100.0, 10..1000)) {
        // INV-5: Verify percentile ordering
    }
}
```

## Test Execution Order

1. Run unit tests first (fast feedback)
2. Run integration tests (realistic scenarios)
3. Run stress tests (performance validation)
4. Run property tests (edge case coverage)

## Coverage Requirements

- Line coverage: >= 80%
- Branch coverage: >= 70%
- All error variants must be tested
- All invariants must have violation tests

## Test Data Fixtures

Located in `diagram_tool/tests/fixtures/perf/`:
- `small_scene.json` (10 nodes)
- `medium_scene.json` (500 nodes)
- `large_scene.json` (3000 nodes)
- `max_scene.json` (10000 nodes)
