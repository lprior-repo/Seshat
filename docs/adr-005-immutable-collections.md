# ADR-005: im crate for Immutable Collections

## Status
Accepted

## Date
2026-03-08

## Context
Immutable state requires immutable collections. We need efficient add/remove/update without mutation.

## Decision
We will use the **im crate** for collections:
- **HashMap<NodeId, Node>** - O(1) node lookup
- **HashMap<EdgeId, Edge>** - O(1) edge lookup
- **Vector<T>** - Immutable vector with structural sharing

## Usage Pattern

```rust
// Old way (mutable - forbidden)
// nodes.insert(node_id, node);
// nodes.get(&node_id).unwrap();

// New way (immutable)
let nodes: HashMap<NodeId, Node> = doc.document.nodes
    .insert(node_id, node)  // Returns new HashMap
    .remove(&old_id);       // Chain operations

// Signal update
doc_signal.set(DiagramDocument {
    document: DocumentData { nodes, .. },
    ..doc_signal.read().clone()
});
```

## Consequences

### Positive
- **Structural sharing** - Modified copies share unchanged parts
- **No clone boilerplate** - O(1) clone due to persistence
- **Thread-safe** - Immutable, so no locks needed
- **Familiar API** - Similar to std::collections

### Negative
- **Learning curve** - Different from mutable patterns
- **Performance** - Slight overhead vs mutable (acceptable)
- **Debugging** - Cannot inspect via println!

### Risks
- Must benchmark 3000-node operations
- Consider `Vec` vs `HashMap` for small collections
