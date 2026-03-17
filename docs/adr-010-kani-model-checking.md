# ADR-010: Kani Model Checking

## Status
Accepted

## Date
2026-03-15

## Context
Kani is a bit-precise model checker for Rust. It can prove the absence of panics, overflow, and certain classes of bugs in pure functions. The GO Skill lifecycle (ADR-008) requires Kani verification at State 5.7.

## Decision
We will use **Kani** for formal verification of critical state machines and geometry calculations.

## When to Use Kani

| Use Case | Example |
|----------|---------|
| State machine transitions | Document lifecycle, selection states |
| Geometry calculations | Transform functions, collision detection |
| Invariant verification | Edge cases, boundary conditions |
| Panic prevention | Division by zero, overflow |

## When Kani is NOT Required

- Simple CRUD operations
- UI rendering code
- Database I/O
- Tests already covered by proptest

## Kani Proof Template

```rust
#[kani::proof]
fn verify_<function>_<property>() {
    // 1. Generate symbolic inputs
    let input: f64 = kani::any();
    
    // 2. Constrain to valid domain
    kani::assume(input.is_finite());
    kani::assume(input.abs() < MAX_VALUE);
    
    // 3. Call function under test
    let result = my_function(input);
    
    // 4. Assert postcondition
    assert!(result.is_valid());
}
```

## Existing Kani Harnesses

| File | Coverage |
|------|----------|
| `geometry/transforms_kani.rs` | Rotation, scaling, transforms |
| `geometry/operations_kani.rs` | Geometry operations |

## Kani Command

```bash
cd diagram_tool && cargo kani
```

## Skip Justification Template

If Kani is not needed for a bead, create `kani-justification.md`:

```markdown
# Kani Justification

## State Machines
<list state machines or "None">

## Why Kani Not Needed
<explain why contract/tests provide sufficient guarantees>

## Formal Reasoning
<explain why invalid states are unreachable>
```

## Rules
- Always constrain symbolic inputs with `kani::assume()`
- Use `kani::any()` for symbolic values
- Run `cargo kani` before merging to main
