# Implementation Summary - Bead oya-r98

## Feature
Self-loop edges (node connected to itself) render without crash (EDG-032)

## Changes Made

### 1. routing.rs - Removed unconditional self-loop rejection
**File**: `diagram_tool/src/core/routing.rs`

- Modified `validate_edge_endpoints` function to allow self-loops at the routing layer
- Self-loop validation is now handled at the policy layer based on CyclePolicy

**Before**:
```rust
fn validate_edge_endpoints(...) -> Result<(), RoutingError> {
    if source == target {
        return Err(RoutingError::SelfLoop(source.clone()));  // Always reject
    }
    // ... node existence checks
}
```

**After**:
```rust
fn validate_edge_endpoints(...) -> Result<(), RoutingError> {
    // Self-loop validation moved to policy layer
    // Only check node existence here
    // ... node existence checks
}
```

### 2. routing_tests.rs - Updated test for new behavior
**File**: `diagram_tool/src/core/routing_tests.rs`

- Changed test from expecting self-loop rejection to expecting self-loop allowance
- Test now verifies that self-loops are allowed at the routing layer

### 3. Policy layer (existing) - Handles self-loop rejection in DAG mode
**File**: `diagram_tool/src/models/policy.rs`, `diagram_tool/src/models/dag.rs`

- The DAG validation already handles self-loops correctly
- When CyclePolicy::Deny, self-loops are rejected as cycles
- When CyclePolicy::Allow, self-loops are permitted

### 4. Rendering - No changes needed
**Files**: `diagram_tool/src/ui/canvas/canvas_view.rs`

- The rendering code handles self-loops gracefully (produces degenerate output but doesn't crash)
- This satisfies the contract requirement Q3: "Self-loop edge renders without panic/crash"

## Contract Clause Mapping

| Contract Clause | Implementation |
|-----------------|----------------|
| P1: Nodes exist | validate_edge_endpoints checks source/target existence |
| P2: Deny mode rejects self-loops | DAG validation (validate_dag) treats self-loops as cycles |
| Q1: Edge stored with source==target | create_edge now allows self-loops |
| Q2: Allow mode permits self-loops | Policy layer passes through in Allow mode |
| Q3: No crash in rendering | Existing code handles degenerate edges |

## Test Results

- ✅ `mutation::pipeline::proptests::edge_self_loop_validation` - PASS
- ✅ `mutation::pipeline::tests::given_preserve_policy_*` - PASS
- ✅ Build successful

## Notes

- The CLI test `given_self_loop_edge_when_validate_runs_then_it_fails_with_dag_error` still works because it uses the default CyclePolicy (Deny) which rejects self-loops
- Self-loops are now allowed in non-DAG mode (CyclePolicy::Allow)
- The implementation follows the functional-rust pattern: Data→Calc→Actions
