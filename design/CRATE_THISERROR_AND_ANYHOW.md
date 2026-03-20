# Error Handling Boundaries (`thiserror` & `anyhow`)

In Seshat, **panics are strictly forbidden** (`#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`). Every single failure must be explicitly typed and handled. 

To achieve this cleanly, we split our error handling into two distinct crates based on the architectural tier: `thiserror` for the Core, and `anyhow` for the Shell.

## The Functional Core: `thiserror`

Code in the `models/` and `core/` directories must use `thiserror`. 
The core is a library. It does not know *how* an error will be presented (CLI, Web, Desktop), so it must define a rigid, exhaustively matched taxonomy of exactly what went wrong.

```rust
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum GeometryError {
    #[error("Cannot calculate intersection: Lines are parallel")]
    ParallelLines,
    
    #[error("Node {0} has zero width or height")]
    ZeroDimension(NodeId),
}

// Pure function returning strictly typed domain errors
pub fn calculate_intersection(l1: Line, l2: Line) -> Result<Point, GeometryError> { ... }
```
**Why?** Because a UI handler might want to match on `GeometryError::ParallelLines` and snap the line differently, rather than crashing.

## The Imperative Shell: `anyhow`

Code in the outer boundaries (`store_sqlx.rs`, `cli.rs`, `main.rs`) must use `anyhow`.
At the boundary, we are no longer interested in exhaustively matching errors. If the database file is locked, or JSON parsing fails, we just want to grab the error, attach some human-readable context, and bubble it up to the user.

```rust
use anyhow::{Context, Result};

pub async fn load_document(path: &Path) -> Result<DiagramDocument> {
    let bytes = std::fs::read(path)
        .with_context(|| format!("Failed to read diagram file at {:?}", path.display()))?;
        
    let doc: DiagramDocument = serde_json::from_slice(&bytes)
        .context("Diagram file is corrupted: invalid JSON format")?;
        
    Ok(doc)
}
```

### The Hand-off
When pure core logic is executed inside the shell, we translate `thiserror` domain errors into `anyhow` context errors seamlessly using the `?` operator.

```rust
// In CLI shell
let next_doc = core::transform::apply_move(&current_doc, op)
    .context("Failed to calculate node translation during move command")?;
```