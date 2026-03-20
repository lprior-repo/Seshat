# Persistent Data Structures (`im` & `rpds`)

Seshat relies entirely on **immutable state** to guarantee deterministic UI rendering, robust undo/redo (time travel), and side-effect-free pure calculations. We achieve this using the `im` and `rpds` crates.

## The Problem with `std::collections`
If we used `std::collections::HashMap` for our 3,000-node graph, every time a human dragged a single node, we would either:
1. Mutate the `HashMap` in place, which ruins our pure `Data -> Calc -> Actions` pipeline and destroys time-travel debugging.
2. Perform a deep `clone()` of the entire 3,000-node graph, which allocates massive amounts of memory and blows past our 8ms frame budget, dropping the app to <10 FPS.

## The Solution: Structural Sharing

Both `im` and `rpds` implement **persistent data structures** using Hash Array Mapped Tries (HAMT) or similar tree structures. When you "mutate" a persistent structure, you don't overwrite the original. Instead, you get a new copy that shares 99% of its memory with the old version.

```rust
// The old graph stays in memory unchanged
let old_nodes = doc.nodes.clone(); // O(1) time, simply bumps a reference count!

// Creating the new graph only allocates memory for the branch of the tree that changed
let new_nodes = old_nodes.insert(node_id, new_node);
```

## `im` vs `rpds` in Seshat

### `im` (Used for Document State)
- We use `im::HashMap` and `im::Vector` for the core `DiagramDocument` state.
- **Why?** `im` implements the exact same API as `std::collections`, making it highly ergonomic. It supports in-place mutation (`.insert()`) if you are the sole owner of the data, gracefully degrading to a structural clone if the data is shared (e.g., pushed to the Undo stack).
- `im` structs implement `Serialize`/`Deserialize` flawlessly.

### `rpds` (Used for Agent State / Strict Persistence)
- We use `rpds::List` and `rpds::RedBlackTreeMap` for our Undo/Redo history stack and strict pure-function contexts where we NEVER want accidental in-place mutation.
- **Why?** `rpds` forces a purely functional API. You cannot `.insert()` in place; you must do `let new_list = old_list.push_front(item)`. This guarantees that we don't accidentally mutate the history stack.

## Undo / Redo Implementation Details

Because of structural sharing, our History Stack is shockingly cheap.

```rust
pub struct History {
    undo_stack: rpds::List<DiagramDocument>, // Max 100 entries
    redo_stack: rpds::List<DiagramDocument>, 
}
```

When a user moves a node:
1. We take the current `DiagramDocument`.
2. We `push_front` onto the `undo_stack`. This is an **O(1)** operation. It does not clone 3,000 nodes; it just copies a pointer to the root of the `im::HashMap` tree.
3. If memory hits the 100-snapshot limit, we use a FIFO eviction policy, dropping the oldest reference. The memory of the dropped document is automatically freed *if* no other timeline references those exact nodes.