bead_id: bd-1g4
bead_title: perf-baseline: Establish 3000-node performance benchmarks (120 FPS target)
phase: p0
updated_at: 2026-03-03T00:00:00Z

# Contract Specification: Performance Baseline

## System Under Test

Target: `diagram_tool/src/perf/` (new module)
Purpose: Establish baseline performance measurements for 3000-node diagrams

## Target FPS: 120 FPS (8.33ms frame time)

## Preconditions (Type Encoding)

### P1: Valid Node Count
- **Type**: `struct NodeCount(u32)`
- **Constraint**: `1 <= NodeCount <= 10_000`
- **Encoding**: Newtype with constructor validation
- **Rationale**: Prevents invalid benchmark configurations

### P2: Valid Benchmark Duration
- **Type**: `struct DurationMs(u64)`
- **Constraint**: `DurationMs >= 100` (minimum 100ms)
- **Encoding**: Newtype with constructor validation
- **Rationale**: Ensures statistically meaningful measurements

### P3: Warm-up Iterations Complete
- **Type**: `struct WarmupComplete { iterations: u32 }`
- **Constraint**: `iterations >= 3`
- **Encoding**: State machine token
- **Rationale**: JIT compilation stabilization

### P4: Measurement Environment Isolated
- **Type**: `enum IsolationLevel { Process, Thread, None }`
- **Constraint**: Tests use `Process` isolation for critical benchmarks
- **Encoding**: Compile-time enum
- **Rationale**: Prevents measurement contamination

### P5: FPS Sample Rate Valid
- **Type**: `struct SampleRate(u32)`
- **Constraint**: `SampleRate >= 60` (Hz)
- **Encoding**: Newtype with validation
- **Rationale**: Nyquist theorem - must sample at 2x target

## Postconditions

### POST-1: FPS Measurement Accuracy
After `measure_fps()` completes:
- `result.mean_fps >= 0.0`
- `result.std_dev >= 0.0`
- `result.samples.len() >= 10`
- `result.confidence_interval_95` is computed

### POST-2: Benchmark Reproducibility
After running the same benchmark twice with same seed:
- `abs(run1.mean_fps - run2.mean_fps) < 5.0` (within 5 FPS)
- `abs(run1.p50_ms - run2.p50_ms) < 0.5` (within 0.5ms)

### POST-3: Performance Baseline Recorded
After `establish_baseline()`:
- Baseline JSON file exists at `target/perf/baseline.json`
- Contains entries for: pan, zoom, select, drag operations
- Each entry has: mean_fps, p50_ms, p99_ms, samples_count

### POST-4: Regression Detection Ready
After `run_regression_test()`:
- Returns `RegressionResult { passed: bool, delta_fps: f64 }`
- `delta_fps` represents change from baseline
- `passed` is `true` if `delta_fps > -10.0` (no more than 10 FPS drop)

## Invariants

### INV-1: No NaN in Measurements
All FPS and timing measurements must be finite:
```rust
forall m in measurements: !m.is_nan() && !m.is_infinite()
```

### INV-2: Monotonic Timestamps
Timestamp sequence is strictly non-decreasing:
```rust
forall i < j: timestamps[i] <= timestamps[j]
```

### INV-3: Frame Time Consistency
Frame time and FPS are reciprocally consistent:
```rust
forall sample: abs(sample.fps - 1000.0 / sample.frame_time_ms) < 0.01
```

### INV-4: Sample Count Matches
Reported sample count equals actual samples:
```rust
report.sample_count == report.samples.len()
```

### INV-5: Percentile Ordering
Percentiles are ordered: p50 <= p90 <= p95 <= p99

## Error Taxonomy

```rust
#[derive(Debug, Clone, thiserror::Error)]
pub enum PerfError {
    #[error("invalid node count: {0} (must be 1-10000)")]
    InvalidNodeCount(u32),

    #[error("invalid duration: {0}ms (must be >= 100ms)")]
    InvalidDuration(u64),

    #[error("measurement failed: {0}")]
    MeasurementFailed(String),

    #[error("benchmark timeout after {ms}ms")]
    Timeout { ms: u64 },

    #[error("insufficient samples: got {got}, need {need}")]
    InsufficientSamples { got: usize, need: usize },

    #[error("baseline not found: {0}")]
    BaselineNotFound(String),

    #[error("regression detected: {delta} FPS drop (threshold: {threshold})")]
    RegressionDetected { delta: f64, threshold: f64 },

    #[error("IO error: {0}")]
    Io(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("environment error: {0}")]
    Environment(String),

    #[error("invariant violation: {invariant} - {details}")]
    InvariantViolation { invariant: &'static str, details: String },
}
```

## Operations to Benchmark

| Operation | Description | Target FPS | Notes |
|-----------|-------------|------------|-------|
| `pan` | Pan viewport by offset | 120 | Continuous movement |
| `zoom` | Zoom in/out at point | 120 | Smooth zoom animation |
| `select` | Click to select node | 120 | Single node selection |
| `drag` | Drag node to new position | 120 | Continuous drag |
| `render_frame` | Full frame render | 120 | Baseline render cost |

## Performance Targets

| Metric | Target | Threshold |
|--------|--------|-----------|
| Mean FPS | 120 | >= 100 |
| P50 Frame Time | 8.33ms | <= 10ms |
| P99 Frame Time | 16.67ms | <= 20ms |
| Std Dev | < 2ms | < 5ms |

## Success Criteria

1. All benchmark operations produce valid measurements
2. Baseline JSON file is generated and valid
3. Regression test infrastructure is functional
4. All measurements satisfy INV-1 through INV-5
5. 120 FPS target is documented as achieved or risk noted
