# Testing Strategy

We treat testing rigor as a first-class citizen, inspired by Martin Fowler, Kent Beck, and David Farley.

## Standard of Rigor
We expect exhaustive combinatorial unit testing (happy path, unhappy path, edge cases). A test that just asserts `result.is_ok()` is not enough. 

### 1. Unit Tests (Pure Calculations)
Place tests inline using `#[cfg(test)]` for internal pure functions, or in `src/tests/` for broader boundary tests.
- Extensively test the `Calc` layer.
- Use `proptest` for property-based testing (e.g., ensuring zoom properties hold regardless of input float, ensuring no `NaN` floats break the application). See `diagram_tool/src/models/document.rs` for examples.

### 2. Contract Tests (Strictly Protected)
Certain boundary files define the precise JSON interactions and core invariants. These are protected.
**DO NOT modify without explicit permission:**
- `diagram_tool/src/models/io_tests.rs`
- `diagram_tool/src/test_infrastructure_tests.rs`
- `diagram_tool/src/geometry/mod.rs` (tests)
*(See `TEST_PROTECTION.md` for more details).*

### 3. Fixtures and Test Data Builders
Use domain builders or default structs rather than mocking from scratch.
```rust
let doc = DiagramDocument::default();
// modify doc.document.nodes for specific test setup
```

### 4. Running the Tests
Before any commit, the full hardening pipeline must pass:
```bash
moon run :ci-source
```
For explicit testing during development:
```bash
moon run :test --force
moon run :clippy --force
```

Any code written by an AI agent *must* pass the strict clippy rules (Zero unwrap, zero panics) and the functional tests.
