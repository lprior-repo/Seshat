# Data → Calc → Actions

We strictly follow the Functional Core, Imperative Shell pattern (Data → Calc → Actions). 

This prevents mixing state mutation, network calls, and complex logic, making our core perfectly testable.

## 1. Data (Immutable Domain Models)
Located in `diagram_tool/src/models/`. These are pure structs.

```rust
// diagram_tool/src/models/document.rs
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Node {
    pub kind: NodeKind,
    pub label: String,
    pub x: OrderedFloat,
    pub y: OrderedFloat,
    pub width: OrderedFloat,
    pub height: OrderedFloat,
    pub locked: bool,
    // ...
}
```

## 2. Calc (Pure Functions)
Located in `diagram_tool/src/core/` and `diagram_tool/src/mutation/`. These functions take Data and return new Data (or Operations) using `Result`.

```rust
// diagram_tool/src/core/transform.rs
pub fn calculate_translation(node: &Node, dx: f64, dy: f64) -> Result<Node, TransformError> {
    if node.locked {
        return Err(TransformError::NodeLocked);
    }
    
    Ok(Node {
        x: OrderedFloat(node.x.0 + dx),
        y: OrderedFloat(node.y.0 + dy),
        ..node.clone()
    })
}
```
*Notice: Zero panics, zero unwraps, returns a Result. Zero side effects.*

## 3. Actions (Side Effects)
Located in `diagram_tool/src/store_sqlx.rs`, `cli.rs`, or UI event handlers. This is where we talk to the outside world (SQLite, DOM).

```rust
// diagram_tool/src/store_sqlx.rs
pub async fn apply_node_translation(pool: &SqlitePool, id: NodeId, dx: f64, dy: f64) -> Result<(), Error> {
    // 1. Fetch Data (Action)
    let node = fetch_node(pool, &id).await?;
    
    // 2. Calc (Pure)
    let updated_node = core::transform::calculate_translation(&node, dx, dy)?;
    
    // 3. Save Data (Action)
    save_node(pool, &updated_node).await?;
    
    Ok(())
}
```

## Rules for AI Agents
1. **Never** put SQL queries or IO inside `models/` or `core/`.
2. **Never** use `.unwrap()` or `.expect()` in calculations.
3. If you need to write complex logic, extract it into a pure `Calc` function and write exhaustive unit tests for it.
