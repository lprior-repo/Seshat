# Functional Pipelines (`itertools` & `tap`)

In Tier 2 of our architecture (the Pure Calculation Core), we explicitly ban imperative `for` and `while` loops whenever possible. Loops require mutable state (`let mut results = Vec::new()`), which violates our immutable, functional programming mandate.

Instead, we rely on **Iterator Pipelines** via `itertools` and **Suffix Pipelines** via `tap`.

## `itertools` (Data Transformation)
The standard Rust `Iterator` trait is powerful, but `itertools` provides advanced combinators that allow us to process diagram nodes without intermediate allocations or mutability.

### Example: Finding Unique Connected Nodes
```rust
use itertools::Itertools;

// Imperative (BANNED):
// let mut unique = Vec::new();
// for edge in edges {
//     if edge.source == my_node && !unique.contains(&edge.target) {
//         unique.push(edge.target.clone());
//     }
// }

// Functional (REQUIRED):
let unique_targets = edges.iter()
    .filter(|e| e.source == my_node)
    .map(|e| e.target.clone())
    .unique() // provided by itertools
    .sorted() // provided by itertools
    .collect::<Vec<_>>();
```
This is declarative. We state *what* we want, not *how* to construct it. 

## `tap` (Suffix Pipelining)
The `tap` crate allows us to chain operations on values without breaking the functional flow, particularly useful for debugging or side-effect injection at the boundary without `let` bindings.

### `pipe`
Use `.pipe()` to pass a value into a function without nesting parentheses.
```rust
use tap::Pipe;

// Nested (Hard to read)
// let json = serde_json::to_string(&compress_data(filter_nodes(doc)))?;

// Piped (Flows left-to-right)
let json = doc
    .pipe(filter_nodes)
    .pipe(compress_data)
    .pipe(|d| serde_json::to_string(&d))?;
```

### `tap`
Use `.tap()` to peek at a value for debugging or logging, returning the value unmodified to continue the chain.
```rust
use tap::Tap;

let calculated_node = calculate_translation(node, dx, dy)?
    .tap(|n| println!("Node translated to: {}, {}", n.x.0, n.y.0)); 
    // Passes the node seamlessly to the next caller
```

**Rule**: In the pure core, `.tap()` should only be used for `tracing`/logging (which is an acceptable minor side-effect for debugging), but never for mutating state.