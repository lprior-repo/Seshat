# Implementation: seshat-x0s - PropertiesPanel NodeStyle Select Dropdown

## Contract Summary
Add NodeStyle select dropdown to PropertiesPanel, wire onchange to dispatch UpdateNodeStyle to db_tx.

## Changes Made

### 1. `diagram_tool/src/ui/properties.rs`
- **Imports**: Added `NodeStyle`, `EventEnvelope`, and `dispatch_update_node_style`
- **Helper functions**:
  - `parse_node_style(v: &str) -> Result<NodeStyle, StyleError>` - converts string to NodeStyle enum (now returns Result!)
  - `node_style_str(style: &Option<NodeStyle>) -> &'static str` - converts NodeStyle to string
- **db_tx context**: Added `use_context::<Option<Coroutine<EventEnvelope>>>()` to access the WAL channel
- **NodeStyle select dropdown**: Added in single_node section with:
  - Label: "Style"
  - Options: Box, Cloud, Cylinder, Dashed
  - onchange handler that:
    - Checks if style actually changed (idempotent check - Q5)
    - Pushes history before mutation (Q3)
    - Dispatches UpdateNodeStyle to db_tx (Q4)
    - Updates doc_signal.node.style (Q1)
    - Increments doc.revision (Q2)

### 2. `diagram_tool/src/ui/dispatch.rs` (Pre-existing)
- Already contains `dispatch_update_node_style()` function
- Already contains `create_update_node_style_envelope()` function
- Already contains DomainOp::UpdateNodeStyle in envelope.rs

## Bug Fixes Applied

### Fix 1: Idempotent Check (line 376)
**Issue**: The idempotent check incorrectly compared `None` as `Box`:
```rust
// BEFORE (buggy)
n.style.as_ref().unwrap_or(&NodeStyle::Box) != &new_style
```
This meant changing from None→Box was seen as "no change" - history not pushed, no dispatch.

**Fix**: Compare Option<NodeStyle> directly:
```rust
// AFTER (fixed)
n.style.as_ref() != Some(&new_style)
```
Correctly detects: None→Some as change, Some→Same as no change.

### Fix 2: parse_node_style Returns Result (line 111)
**Issue**: Function silently defaulted to `NodeStyle::Box` for invalid input.

**Fix**: Returns `Result<NodeStyle, StyleError>`:
```rust
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StyleError {
    #[error("Invalid node style: {0}")]
    InvalidNodeStyle(String),
    #[error("Invalid edge style: {0}")]
    InvalidEdgeStyle(String),
    #[error("Invalid arrow type: {0}")]
    InvalidArrowType(String),
}

fn parse_node_style(v: &str) -> Result<NodeStyle, StyleError> {
    match v {
        "box" => Ok(NodeStyle::Box),
        "cloud" => Ok(NodeStyle::Cloud),
        "cylinder" => Ok(NodeStyle::Cylinder),
        "dashed" => Ok(NodeStyle::Dashed),
        _ => Err(StyleError::InvalidNodeStyle(v.to_string())),
    }
}
```

Call site handles Result gracefully:
```rust
let new_style = match parse_node_style(&evt.value()) {
    Ok(style) => style,
    Err(_) => return, // Ignore invalid input from dropdown
};
```

## Constraint Adherence

| Constraint | Implementation |
|------------|----------------|
| Zero panics | Uses `match`, `.ok()`, `is_some_and()` instead of unwrap |
| Zero mut | Uses Signals, functional patterns, no `mut` in logic |
| Result<T, E> | parse_node_style returns Result<StyleError>, dispatch returns Result |
| Expression-based | Uses if-let, match expressions |
| Make illegal states unrepresentable | NodeStyle is an enum with 4 variants |
| Parse at boundary | String → NodeStyle via `parse_node_style()` returns Result |

## Contract Verification

| Precondition | Implementation |
|--------------|----------------|
| P1: Single node selected | Only renders in `if let Some((id, node)) = single_node` block |
| P2: Valid NodeStyle | Enum guarantees valid variants (Box, Cloud, Cylinder, Dashed) |
| P3: Node exists | Checked via `doc.document.nodes.get(&nid)` |
| P4: db_tx available | Passed to dispatch function, `.ok()` handles None |

| Postcondition | Implementation |
|--------------|----------------|
| Q1: Document updated | `n.style = Some(new_style)` |
| Q2: Revision incremented | `doc.revision = doc.revision.increment()` |
| Q3: History pushed | `history.read().push(current)` before mutation |
| Q4: Event dispatched | `dispatch_update_node_style(&db_tx, ...)` |
| Q5: Idempotent check | `has_changes` correctly compares `n.style.as_ref() != Some(&new_style)` |

## Files Modified
- `diagram_tool/src/ui/properties.rs` - Added NodeStyle select dropdown with dispatch wiring, bug fixes applied
