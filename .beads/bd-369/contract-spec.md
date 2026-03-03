# Contract Specification: Test Infrastructure (bd-369)

## Context
- **Feature:** Extend existing test infrastructure to support all 240 test cases from architecture spec
- **Domain terms:**
  - Golden Scene: Canonical JSON fixture representing a diagram state
  - Operation Snapshot: before/after JSON pair for an operation
  - Test Category: Group of related tests (SEL, CLP, HIS, MUL, SUB, EDG, CAM, GEO, SNP, IO)
  - Property-based Test: Proptest/quickcheck style fuzzing with invariants
  - Visual Regression: Screenshot comparison with delta threshold
- **Assumptions:**
  - Playwright is configured and working (playwright.config.ts exists)
  - Golden scene fixtures exist for basic cases (mixed_selection.json, nested_subgraph.json)
  - Existing harness.rs provides fuzz testing infrastructure
  - E2E tests run against Dioxus web renderer
- **Open questions:**
  - How to run property-based tests against Dioxus UI state?
  - Visual regression baseline update workflow?
  - How to generate 5000-node stress test fixtures programmatically?

## Preconditions

| ID | Condition | Enforcement Level | Type/Pattern |
|----|-----------|-------------------|--------------|
| P1 | Test category ID is valid | Compile-time | `enum TestCategory { Sel, Clp, His, Mul, Sub, Edg, Cam, Geo, Snp, Io, Inp }` |
| P2 | Golden scene file exists | Runtime Result | `fn load_fixture(name: &str) -> Result<Value, FixtureError::NotFound>` |
| P3 | Golden scene is valid JSON | Runtime Result | `fn parse_fixture(json: &str) -> Result<Value, FixtureError::InvalidJson>` |
| P4 | Golden scene has required schema version | Runtime Result | `fn validate_schema(doc: &Value) -> Result<(), FixtureError::SchemaMismatch>` |
| P5 | Test environment is isolated (no external network) | Compile-time | No network-using types in test module |
| P6 | Test database path is unique per test | Debug-only | `debug_assert!(!db_path.exists())` at test start |
| P7 | Playwright browser is available | Runtime Result | `fn ensure_browser() -> Result<Browser, E2eError::BrowserUnavailable>` |

## Postconditions

| ID | Guarantee | Verification |
|----|-----------|--------------|
| Q1 | All 240 test cases have test stubs | Count test functions per category, assert >= expected |
| Q2 | Golden scene fixtures load and validate | Each fixture has corresponding load test |
| Q3 | Test runner reports pass/fail per category | Assert runner output contains category summary |
| Q4 | CI integration runs tests on commit | Assert CI workflow calls test runner |
| Q5 | Flaky tests are quarantined, not merged | Assert quarantine mechanism exists |
| Q6 | Visual regression baselines require explicit approval | Assert baseline update requires flag/confirm |
| Q7 | Property-based tests shrink failures to minimal case | Assert proptest shrinking configured |

## Invariants

| ID | Condition | Enforcement | Broken During |
|----|-----------|-------------|---------------|
| I1 | Test environment is reproducible | Type system (no external deps) | Never |
| I2 | Golden scenes are version-controlled | Git (fixture files in repo) | Never |
| I3 | Test execution is deterministic given same seed | Proptest seed parameter | Never |
| I4 | No test depends on execution order | Each test creates fresh state | Never |
| I5 | Test failures produce actionable diagnostics | Error type with context | Never |

## Error Taxonomy

```rust
#[derive(Debug, thiserror::Error)]
pub enum TestHarnessError {
    #[error("Fixture not found: {0}")]
    FixtureNotFound(String),

    #[error("Fixture invalid JSON: {source}")]
    InvalidJson {
        name: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("Schema mismatch: expected version {expected}, got {found}")]
    SchemaMismatch { expected: u32, found: u32 },

    #[error("Missing required field: {field} in {fixture}")]
    MissingRequiredField { fixture: String, field: String },

    #[error("Test category not implemented: {0:?}")]
    CategoryNotImplemented(TestCategory),

    #[error("Browser unavailable: {0}")]
    BrowserUnavailable(String),

    #[error("Visual regression: {baseline} differs by {delta}%")]
    VisualRegression { baseline: String, delta: f64 },

    #[error("Property test failed after {shrinks} shrinks: {case}")]
    PropertyFailure { shrinks: usize, case: String },

    #[error("Test timeout after {ms}ms: {test_name}")]
    Timeout { test_name: String, ms: u64 },

    #[error("CI integration failure: {0}")]
    CiIntegration(String),
}
```

## Contract Signatures

```rust
// Core fixture loading
pub fn load_fixture(name: &str) -> Result<Value, TestHarnessError>;
pub fn validate_fixture_schema(doc: &Value) -> Result<(), TestHarnessError>;
pub fn get_nodes(doc: &Value) -> Result<&Map<String, Value>, TestHarnessError>;
pub fn get_edges(doc: &Value) -> Result<&Map<String, Value>, TestHarnessError>;

// Golden scene management
pub fn create_golden_scene(
    name: &str,
    nodes: Vec<NodeSpec>,
    edges: Vec<EdgeSpec>
) -> Result<Value, TestHarnessError>;

pub fn save_golden_scene(
    name: &str,
    doc: &Value
) -> Result<PathBuf, TestHarnessError>;

// Operation snapshots
pub fn create_operation_snapshot(
    before: &DiagramDocument,
    operation: Operation,
    after: &DiagramDocument
) -> Result<OperationSnapshot, TestHarnessError>;

pub fn verify_operation_snapshot(
    snapshot: &OperationSnapshot,
    actual_after: &DiagramDocument
) -> Result<(), TestHarnessError>;

// Test runner
pub fn run_test_category(
    category: TestCategory,
    filter: Option<&str>
) -> Result<CategoryReport, TestHarnessError>;

pub fn run_all_tests(
    categories: &[TestCategory]
) -> Result<TestSuiteReport, TestHarnessError>;

// Property-based testing
pub fn fuzz_document_operations(
    seed: u64,
    operations: usize
) -> Result<FuzzReport, TestHarnessError>;

pub fn verify_invariant(
    invariant: Invariant,
    doc: &DiagramDocument
) -> Result<(), TestHarnessError>;

// Visual regression
pub fn capture_screenshot(
    page: &Page,
    name: &str
) -> Result<Screenshot, TestHarnessError>;

pub fn compare_to_baseline(
    screenshot: &Screenshot,
    baseline_name: &str,
    threshold_percent: f64
) -> Result<ComparisonResult, TestHarnessError>;

pub fn update_baseline(
    name: &str,
    screenshot: &Screenshot
) -> Result<(), TestHarnessError>;
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Test category ID valid | Compile-time (strongest) | `enum TestCategory { ... }` |
| P2: Golden scene exists | Runtime Result | `load_fixture() -> Result<_, FixtureError::NotFound>` |
| P3: Valid JSON | Runtime Result | `serde_json::from_str() -> Result<_, Error>` |
| P4: Schema version match | Runtime Result | `validate_schema() -> Result<_, FixtureError::SchemaMismatch>` |
| P5: No external network | Compile-time | No `reqwest`/`tokio::net` imports in test module |
| P6: Unique DB path | Debug-only | `debug_assert!(!path.exists())` |
| P7: Browser available | Runtime Result | `ensure_browser() -> Result<_, E2eError>` |

## Violation Examples (REQUIRED)

### Precondition Violations

- **VIOLATES P2:** `load_fixture("nonexistent.json")` -- should produce `Err(TestHarnessError::FixtureNotFound("nonexistent.json"))`
- **VIOLATES P3:** `load_fixture("corrupted.json")` where file contains `{invalid json}` -- should produce `Err(TestHarnessError::InvalidJson { name: "corrupted.json", source: ... })`
- **VIOLATES P4:** `validate_fixture_schema(doc)` where `doc.version = 99` -- should produce `Err(TestHarnessError::SchemaMismatch { expected: 2, found: 99 })`
- **VIOLATES P7:** `ensure_browser()` when chromium not installed -- should produce `Err(TestHarnessError::BrowserUnavailable("chromium not found"))`

### Postcondition Violations

- **VIOLATES Q1:** `run_all_tests(&[TestCategory::Sel])` returns report with `sel_tests: 0` -- should fail CI with "Missing SEL test implementations"
- **VIOLATES Q2:** `load_fixture("mixed_selection.json")` returns doc without "rect-1" node -- should produce `Err(TestHarnessError::MissingRequiredField { fixture: "mixed_selection.json", field: "nodes.rect-1" })`
- **VIOLATES Q5:** Test passes 9/10 runs but flaky -- should be detected by retry mechanism and quarantined
- **VIOLATES Q6:** `update_baseline("test", &screenshot)` without explicit flag -- should produce `Err(TestHarnessError::CiIntegration("baseline update requires --update-baselines flag"))`

### Invariant Violations

- **VIOLATES I3:** `fuzz_document_operations(seed1, 100)` produces different result than `fuzz_document_operations(seed1, 100)` -- should produce `Err(TestHarnessError::PropertyFailure { shrinks: 0, case: "determinism violation" })`
- **VIOLATES I4:** Test A passes alone but fails when run after Test B -- should produce `Err(TestHarnessError::CiIntegration("test isolation failure: Test A depends on Test B state"))`

## Ownership Contracts

### `create_golden_scene`
- **Ownership:** Takes `nodes: Vec<NodeSpec>` and `edges: Vec<EdgeSpec>` by value
- **Why:** Caller constructs specs, function consumes them to build immutable fixture
- **Mutation:** None (pure function)

### `verify_operation_snapshot`
- **Borrow:** Takes `snapshot: &OperationSnapshot` (shared) and `actual_after: &DiagramDocument` (shared)
- **Why:** Read-only comparison, no mutation needed
- **Mutation:** None

### `run_test_category`
- **Borrow:** Takes `filter: Option<&str>` (shared)
- **Why:** Optional filter string, read-only
- **Mutation:** None (returns report)

### `fuzz_document_operations`
- **Ownership:** Returns `Result<FuzzReport, TestHarnessError>` by value
- **Why:** Creates new report object
- **Mutation:** None (deterministic from seed)

## Non-goals

- [ ] Real-time test execution monitoring dashboard
- [ ] Test parallelization across multiple machines
- [ ] Automatic test case generation from specification
- [ ] Visual regression AI-powered diff explanation
- [ ] Test coverage metrics (out of scope for MVP)
