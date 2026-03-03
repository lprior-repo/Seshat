bead_id: bd-1g4
bead_title: perf-baseline: Establish 3000-node performance benchmarks (120 FPS target)
phase: p1
updated_at: 2026-03-03T00:00:00Z

# Implementation: Performance Baseline

## Target Files

- `diagram_tool/src/perf/mod.rs` - Module root and constants
- `diagram_tool/src/perf/error.rs` - Error taxonomy
- `diagram_tool/src/perf/metrics.rs` - Statistics and metrics
- `diagram_tool/src/perf/fps.rs` - FPS measurement utilities
- `diagram_tool/src/perf/benchmark.rs` - Benchmark configuration and execution
- `diagram_tool/src/perf/harness.rs` - Benchmark harness and scene generation
- `diagram_tool/src/perf/regression.rs` - Regression testing infrastructure
- `diagram_tool/tests/perf_integration.rs` - Integration tests
- `diagram_tool/tests/fixtures/perf/small_scene.json` - Test fixture

## Implementation Summary

### Module Structure

```
diagram_tool/src/perf/
  mod.rs          # Public API exports and constants
  error.rs        # PerfError enum with 12 variants
  metrics.rs      # FrameSample, Percentiles, Statistics
  fps.rs          # FpsMeasurement, FpsReport
  benchmark.rs    # NodeCount, DurationMs, BenchmarkConfig, Benchmark, BenchmarkResult
  harness.rs      # Operation, Baseline, BenchmarkHarness, generate_test_scene
  regression.rs   # RegressionResult, RegressionTest, PerformanceReport
```

### Key Design Decisions

1. **Functional Rust Compliance**
   - All modules use `#![deny(clippy::unwrap_used)]`
   - All modules use `#![deny(clippy::expect_used)]`
   - All modules use `#![deny(clippy::panic)]`
   - All modules use `#![forbid(unsafe_code)]`
   - All errors return `Result<T, PerfError>`

2. **Type-Safe Preconditions (P1, P2)**
   - `NodeCount` newtype validates range [1, 10000]
   - `DurationMs` newtype validates minimum 100ms
   - Invalid values return `PerfError` at construction

3. **Invariant Enforcement**
   - INV-1: No NaN/Infinity checked via `is_finite()`
   - INV-2: Monotonic timestamps validated in `FpsReport::validate()`
   - INV-3: Frame time/FPS consistency verified
   - INV-4: Sample count verified against actual length
   - INV-5: Percentile ordering verified via `is_ordered()`

4. **Benchmark Operations**
   - Pan, Zoom, Select, Drag, RenderFrame
   - Each operation has a complexity factor for simulation

5. **Baseline and Regression**
   - Baseline stored as JSON in `target/perf/baseline.json`
   - Regression test compares current vs baseline FPS
   - 20 FPS drop threshold for regression detection

## Test Coverage

### Unit Tests (82 tests in perf:: modules)

- Error taxonomy tests
- Metrics calculation tests
- FPS measurement tests
- Benchmark configuration tests
- Harness functionality tests
- Regression detection tests

### Integration Tests (18 tests)

| Test ID | Description | Status |
|---------|-------------|--------|
| HP-001 | Measure FPS with 3000 nodes | PASS |
| HP-002 | Run pan benchmark | PASS |
| HP-003 | Run zoom benchmark | PASS |
| HP-004 | Run select benchmark | PASS |
| HP-005 | Run drag benchmark | PASS |
| HP-006 | Generate baseline JSON | PASS |
| HP-008 | Percentile calculations | PASS |
| HP-010 | Benchmark reproducibility | PASS |
| EP-001 | Invalid node count (0) | PASS |
| EP-002 | Invalid node count (10001) | PASS |
| EP-003 | Invalid duration (0ms) | PASS |
| EP-004 | Invalid duration (50ms) | PASS |
| EC-001 | Single node benchmark | PASS |
| EC-002 | Maximum nodes (10000) | PASS |
| INV-001 | No NaN in measurements | PASS |
| INV-004 | Sample count matches | PASS |
| INV-005 | Percentile ordering | PASS |
| - | Regression detection | PASS |

## Functional Rust Compliance

- Zero `unwrap()` calls in source code
- Zero `expect()` calls in source code
- Zero `panic!()` calls in source code
- Zero `unsafe` blocks
- All failures return `Result::Err`

## Files Created

| File | Lines | Purpose |
|------|-------|---------|
| `src/perf/mod.rs` | 56 | Module exports and constants |
| `src/perf/error.rs` | 134 | Error taxonomy |
| `src/perf/metrics.rs` | 265 | Statistics and metrics |
| `src/perf/fps.rs` | 225 | FPS measurement |
| `src/perf/benchmark.rs` | 323 | Benchmark configuration |
| `src/perf/harness.rs` | 368 | Benchmark harness |
| `src/perf/regression.rs` | 395 | Regression testing |
| `tests/perf_integration.rs` | 245 | Integration tests |
| `tests/fixtures/perf/small_scene.json` | 146 | Test fixture |
| `src/lib.rs` | 22 | Library exports |
