# Domain-Driven Design & Functional Rust

Seshat relies on an uncompromising adherence to **Domain-Driven Design (DDD)** and the **Functional Core, Imperative Shell (Data → Calc → Actions)** pattern, heavily inspired by Scott Wlaschin's "Domain Modeling Made Functional."

We do not view code as just a sequence of steps; we view it as a series of strictly typed state transitions.

---

## 🛑 1. Make Illegal States Unrepresentable

We use Rust’s type system to ensure that invalid business logic simply **cannot compile**. 

### No Primitive Obsession
Do not pass raw `String` or `f64` types when the domain dictates a specific meaning. Use **newtypes** to wrap primitives so that we cannot accidentally mix up a `NodeId` with an `AuthorId`.

```rust
// BAD: Primitive obsession
fn update_node(id: String, author: String, x: f64) { ... }

// GOOD: Domain-typed boundaries
pub struct NodeId(pub String);
pub struct AuthorId(pub String);
pub struct OrderedFloat(pub f64);

fn update_node(id: NodeId, author: AuthorId, x: OrderedFloat) { ... }
```

### Enums Over Booleans
Never use boolean flags (`is_locked`, `is_draft`) to represent mutually exclusive states. This inevitably leads to logic errors when multiple flags conflict.

```rust
// BAD: Invalid state is possible (what if both are true?)
struct DiagramNode {
    is_draft: bool,
    is_published: bool,
}

// GOOD: Explicit state machine transitions via Enums
enum NodeState {
    Draft(DraftNode),
    Published(PublishedNode),
}
```

---

## 🛡️ 2. Parse, Don't Validate

Validation functions that return `bool` (e.g., `is_valid(email)`) are dangerous because they force downstream code to re-validate or blindly trust the data.

Instead, we **parse** at the system boundaries. The parsing function takes an untrusted input and returns a `Result<TrustedType, Error>`. Once instantiated, the TrustedType inherently guarantees its validity.

```rust
// Untrusted input from API or CLI
let input_id: String = payload.id;

// Parse it into a trusted domain type at the edge
let node_id = NodeId::parse(input_id)?; 

// Downstream functions now simply require `NodeId` and never need to validate it
```

---

## 🧮 3. Data → Calc → Actions

### Tier 1: Data (Immutable Domain Models)
- Located in `models/`.
- Pure, inert, serializable structs and enums. 
- Use `im` crate for immutable collections (e.g., `im::HashMap`, `rpds::Vector`) to allow efficient, structural sharing without deep cloning.

### Tier 2: Calculations (The Functional Core)
- Located in `core/` and `mutation/`.
- **Pure Functions**: Time-independent and referentially transparent. 
- **Signature**: `Data -> Result<Data, Error>`
- **No Side Effects**: Never put SQL queries, `reqwest` calls, or `println!` side effects here.
- **Error Handling**: Zero panics (`#![deny(clippy::unwrap_used)]`). Always return an explicit `Result<T, Error>`.
- **No Mutability**: Avoid `let mut`. Prefer iterator pipelines (`itertools`), suffix pipelines (`tap`), and persistent state (`rpds` / `im`).

### Tier 3: Actions (The Imperative Shell)
- Located in `store_sqlx.rs`, `cli.rs`, and UI event handlers.
- This is where the dirty work happens (I/O, database interactions, async runtimes).
- Keep this layer as thin as possible. 
- **The Flow**: 
  1. Fetch Data (Action)
  2. Transform Data (Calc)
  3. Save Data (Action)

```rust
// Action Shell Example
pub async fn execute_node_move(pool: &SqlitePool, op: MoveOp) -> Result<(), AppError> {
    // 1. Fetch
    let current_doc = fetch_document(pool).await?; 
    
    // 2. Calc (Pure core logic, fully unit testable without mocking DB)
    let next_doc = core::transform::apply_move(&current_doc, op)?;
    
    // 3. Save
    save_document(pool, next_doc).await?;
    
    Ok(())
}
```

---

## 💥 4. Strict Error Handling Taxonomy (ADR-007)

We use explicit, nested enumerations for all failures. `thiserror` is used in the pure core to define exact domain errors. `anyhow` is strictly reserved for the imperative shell (CLI/Actions) to wrap errors with user-facing context.

```rust
#[derive(thiserror::Error, Debug)]
pub enum ValidationError {
    #[error("Node {node} creates a cycle with {parent}")]
    DagCycle { node: NodeId, parent: NodeId },
    
    #[error("Target node {0} does not exist")]
    NodeNotFound(NodeId),
}
```