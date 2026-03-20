# Property-Based Fuzzing (`proptest`)

Because our Core logic (`Data -> Calc`) is entirely pure (no I/O, no mutability, no side-effects), it is the perfect candidate for **Property-Based Testing**. We use the `proptest` crate for this.

## Why Proptest?
Traditional unit tests are "Example-Based" (e.g., `assert_eq!(2 + 2, 4)`). The developer has to think of edge cases.
Property-based tests define a **universal truth** (a property), and `proptest` throws thousands of randomized, adversarial inputs at the function to find an input that breaks the truth.

If it finds a failure, it automatically **shrinks** the input down to the smallest possible reproducible test case.

## Properties in Seshat

We test mathematical invariants on our geometries and CRDTs.

### Example 1: Inverses (Undo should be perfect)
If I translate a node by `(dx, dy)`, and then translate it by `(-dx, -dy)`, it MUST end up in the exact original position.

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_translation_is_invertible(
        x in -10000.0..10000.0f64,
        y in -10000.0..10000.0f64,
        dx in -500.0..500.0f64,
        dy in -500.0..500.0f64
    ) {
        let original = Node::new(x, y);
        let moved = translate(&original, dx, dy).unwrap();
        let inverted = translate(&moved, -dx, -dy).unwrap();
        
        assert_eq!(original.x, inverted.x);
        assert_eq!(original.y, inverted.y);
    }
}
```

### Example 2: Commutativity (CRDT Conflict Resolution)
Because we use CRDTs (LWW-Element-Set) for our Two-Way Sync, the order in which events are merged must not matter.
Property: `merge(A, B) == merge(B, A)`

```rust
proptest! {
    #[test]
    fn crdt_merge_is_commutative(op1 in any_crdt_op(), op2 in any_crdt_op()) {
        let merge_a = resolve_conflict(&op1, &op2);
        let merge_b = resolve_conflict(&op2, &op1);
        
        assert_eq!(merge_a.payload, merge_b.payload);
        assert_eq!(merge_a.hlc_timestamp, merge_b.hlc_timestamp);
    }
}
```

## AI Agent Rules
When writing complex core logic (especially geometry or math in `canvas_math`), you are required to write at least one property test validating its invariants. Do not just write a happy-path example test.