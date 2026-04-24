# Red Queen Findings: se-0dt — canvas_domain/src/stubs.rs

## Verdict: CROWN CONTESTED

3 generations, 30 attacks, 7 survivors (23.3% kill rate).
Threat level: MODERATE — defenses hold but gaps exist.

## Survivors (done_when entries)

### CRITICAL (1)
| ID | Dimension | Finding | Location |
|----|-----------|---------|----------|
| GEN-2-2 | test-coverage | Zero tests for stubs module | stubs.rs had 0 test coverage |

### MAJOR (5)
| ID | Dimension | Finding | Location |
|----|-----------|---------|----------|
| GEN-1-1 | fp-gate-exhaustive | Wildcard enum match arms | diagram_models (NOT stubs.rs) |
| GEN-3-3 | fp-gate-exhaustive | Wildcard enum match arms | diagram_models (NOT stubs.rs) |
| GEN-3-5 | fowler-exhaustive | Wildcard enum match arms | diagram_models (NOT stubs.rs) |
| GEN-3-6 | fowler-security | rand v0.8.5 advisory | dependency (NOT stubs.rs) |
| GEN-3-7 | fowler-licenses | License issues | dependency (NOT stubs.rs) |

### MINOR (1)
| ID | Dimension | Finding | Location |
|----|-----------|---------|----------|
| GEN-3-4 | fp-gate-format | Formatting violations | stubs.rs (FIXED) |

## Stubs.rs-Specific Findings

### CRITICAL: Zero test coverage (FIXED)
- Before: 0 tests for stubs module
- After: 17 tests covering all public API surface
- Tests added: dispatch_update_label, dispatch_node_resize, ResizeBounds, DispatchError, LabelTargetType, drag_original_positions (8 scenarios including empty, nonexistent, single, multiple, parent-child, deep hierarchy, sibling exclusion, circular parent)

### MAJOR: API drift between stubs and real implementations
1. `dispatch_update_label` return type: stub `Result<(), DispatchError>` vs real `Result<DispatchResult, DispatchError>` (real returns a DispatchResult with mutation tracking)
2. `dispatch_node_resize` return type: stub `Result<(), DispatchError>` vs real `Result<DispatchResult, DispatchError>`
3. `ResizeBounds` stub: unit struct with `new()` accepting 8 params but storing nothing. Real has 8 named fields (id, original_x/y/w/h, new x/y/w/h)
4. Parameter style: stub uses `Option<&Coroutine>` (owned Option), real uses `&Option<Coroutine>` (reference)
5. `DispatchError` stub: single `Failed` variant. Real has 11 variants (WalDisconnected, ChannelMissing, NoTx, SendFailed, InvalidCoordinates, NoSelection, DispatchIncomplete, EdgeNotFound, NotSelected, CycleDetected, SelfLoop)

### MINOR: Type duplication
- `LabelTargetType { Node, Edge }` defined in stubs.rs instead of re-exporting from `diagram_models::envelope::LabelTargetType`
- Consumer `commit.rs` imports from stubs, not from diagram_models

### OBSERVATION: Non-stub functions in stubs.rs
- `mutate_doc_with_history` — REAL implementation logic (not a stub). Clones doc, applies closure, pushes old to history. Requires Dioxus Signal runtime context.
- `drag_original_positions` — IDENTICAL copy of `diagram_tool/src/ui/interaction/drag.rs`. Real production logic duplicated as a "stub".

### OBSERVATION: Cannot unit-test mutate_doc_with_history
- Requires Dioxus VirtualDom context for Signal::new()
- Existing tests in commit_tests.rs wrap it in VirtualDom::new()
- Integration-level concern, not stub-level unit test

## Out-of-Scope Findings (NOT stubs.rs)
These were flagged by automated weapons but exist in other crates:
- Wildcard enum match arms in diagram_models
- rand v0.8.5 security advisory (dev dependency)
- License issues in cargo deny

## Quality Gates for stubs.rs
- PASS: No Panic (clippy unwrap/expect/panic denied)
- PASS: Format (fixed, was failing)
- PASS: Lint (clippy warnings denied)
- PASS: Tests (17 new tests + 112 existing pass)
- PASS: DRY
- PASS: Dead code / unused imports
- PASS: .unwrap() / .expect() / todo!() / unimplemented!() free

## done_when (stubs.rs-specific, validated)
1. `cargo test -p canvas_domain --lib stubs` [expect_exit=0] — PASS
2. `cargo clippy -p canvas_domain -- -D clippy::unwrap_used` [expect_exit=0] — PASS
3. `cargo fmt --check` [expect_exit=0] — PASS
