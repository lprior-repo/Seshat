# Contract Specification: LockState Enum Migration (seshat-sr3b)

## Context

- **Feature**: Replace `Node.locked: bool` with `LockState` enum internally, with JSON transformation layer for backwards compatibility
- **Domain terms**:
  - `Node` - A diagram element with position, size, and properties
  - `NodeKind` - Enum with variants: `Node`, `Subgraph`, `Text`
  - `locked` - External JSON field (bool) for backwards compatibility
  - `LockState` - Internal Rust enum to replace boolean, makes illegal states unrepresentable
  - `lock_state` - Internal Rust field name (serialized as `locked` in JSON)
- **Assumptions**:
  - Subgraphs (NodeKind::Subgraph) are always movable regardless of lock state
  - The 70+ references span UI, models, layout, and core transform modules
  - Serialization/deserialization must maintain backwards compatibility with existing JSON files
- **Open questions**: None - domain fully understood from grep analysis

## Preconditions

- **P1**: Any code that reads `node.locked` must use `node.lock_state.is_locked()` instead (internal code)
- **P2**: Any code that writes `node.locked = <bool>` must use `node.lock_state = LockState::Locked/Unlocked` instead
- **P3**: All comparisons `node.locked == true` become `node.lock_state.is_locked()`
- **P4**: All comparisons `node.locked == false` become `!node.lock_state.is_locked()`
- **P5**: The pattern `!node.locked || node.kind == NodeKind::Subgraph` (movable check) must be replaced with `node.lock_state.is_movable(&node.kind)`
- **P6**: JSON serialization uses `locked: bool` format for backwards compatibility
- **P7**: JSON deserialization accepts both `locked: bool` (legacy) and `lock_state: String` (new format)

## Postconditions

- **Q1**: `Node.lock_state` field exists internally and is of type `LockState`
- **Q2**: `Node.locked` field does NOT exist in Rust struct (external JSON uses `locked`)
- **Q3**: `LockState` enum has variants `Unlocked` and `Locked`
- **Q4**: `LockState` implements `is_locked() -> bool` method
- **Q5**: `LockState` implements `is_movable(node_kind: &NodeKind) -> bool` that encapsulates Subgraph exception
- **Q6**: All 70+ references updated throughout codebase
- **Q7**: Serialization outputs `locked: bool` in JSON (backwards compatible)
- **Q8**: Deserialization accepts `locked: bool` (legacy format) and converts to `LockState`
- **Q9**: Hashing behavior preserved (LockState must implement Hash)
- **Q10**: Default for Node produces `lock_state: LockState::Unlocked`

## Invariants

- **I1**: For any Node with `NodeKind::Subgraph`, `is_movable()` always returns `true` regardless of lock state
- **I2**: For any Node with `NodeKind::Node` or `NodeKind::Text`, `is_movable()` returns `true` iff lock_state is `Unlocked`
- **I3**: `Default` for `Node` produces `lock_state: LockState::Unlocked`
- **I4**: JSON round-trip `serialize → deserialize → serialize` produces identical output for same lock state

## Error Taxonomy

This is a refactoring with no runtime errors - all errors are compile-time:
- **CompileError::FieldNotFound** - If `node.locked` is accessed after migration in Rust code
- **CompileError::WrongType** - If `LockState` methods not used correctly

Runtime deserialization errors (handled gracefully):
- **DeserializationError::InvalidLockValue** - If JSON contains invalid `locked` value (not bool)
- **DeserializationError::MissingLockField** - If neither `locked` nor `lock_state` present (defaults to Unlocked)

## Contract Signatures

```rust
// New enum definition
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub enum LockState {
    #[default]
    Unlocked,
    Locked,
}

impl LockState {
    /// Returns true if the node is in a locked state
    pub fn is_locked(&self) -> bool {
        matches!(self, LockState::Locked)
    }
    
    /// Returns true if the node can be moved/edited
    /// Subgraphs are always movable regardless of lock state
    pub fn is_movable(&self, node_kind: &NodeKind) -> bool {
        match node_kind {
            NodeKind::Subgraph => true,
            _ => !self.is_locked(),
        }
    }
}

// Node struct modification with serde transformation
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Hash)]
pub struct Node {
    pub kind: NodeKind,
    pub position: Position,
    pub size: Size,
    #[serde(default)]
    pub lock_state: LockState,  // Internal field - serialized as "locked" in JSON
    // ... rest unchanged ...
}

// Custom serializer/deserializer for backwards compatibility
// JSON format: "locked": true/false (not "lock_state")
mod lock_state_serde {
    use super::LockState;
    use serde::{Deserialize, Deserializer, Serializer, Serialize};
    
    /// Serializes LockState to JSON as "locked": bool
    pub fn serialize<S>(lock_state: &LockState, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let locked = lock_state.is_locked();
        locked.serialize(serializer)
    }
    
    /// Deserializes from JSON - accepts both "locked": bool (legacy) and new format
    pub fn deserialize<'de, D>(deserializer: D) -> Result<LockState, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Try to deserialize as bool first (legacy format)
        let result: Result<bool, _> = Deserialize::deserialize(deserializer);
        match result {
            Ok(true) => Ok(LockState::Locked),
            Ok(false) => Ok(LockState::Unlocked),
            Err(_) => Ok(LockState::Unlocked), // Default if missing/invalid
        }
    }
}
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Use lock_state.is_locked() | Compile-time | Compiler error if `.locked` field accessed in Rust |
| P2: Use LockState variants | Compile-time | Compiler error if boolean assigned to lock_state |
| P3: Replace boolean comparisons | Compile-time | Compiler error if `.locked == true/false` used in Rust |
| P5: Use is_movable() | Compile-time | Compiler error if old pattern `!locked \|\| kind == Subgraph` remains |
| P6/P7: JSON format | Runtime | Serde transformation handles conversion |

**Enforcement Strategy**: The migration removes the `locked` field from the Rust struct. Any remaining references to `.locked` will cause:
1. **Compile error**: Field does not exist on Node
2. **JSON transformation**: Serde handles backwards compatible serialization/deserialization

## Violation Examples (REQUIRED)

- **VIOLATES Q1**: Code still accesses `node.locked` in Rust -- produces compile error: `field locked not found in struct Node`
- **VIOLATES Q2**: Code still uses `node.locked = true` -- produces compile error: `field locked not found in struct Node`
- **VIOLATES Q3**: Code still checks `node.locked == true` -- produces compile error: `field locked not found in struct Node`
- **VIOLATES Q5**: Code still uses `!node.locked || node.kind == NodeKind::Subgraph` -- should use `node.lock_state.is_movable(&node.kind)` instead
- **VIOLATES Q7**: Serialized JSON contains `"lock_state"` instead of `"locked"` -- violates backwards compatibility
- **VIOLATES I4**: JSON round-trip produces different output for same lock state -- violates invariant

## Ownership Contracts

- **Not applicable**: This is a structural change (field replacement), not a function signature change
- **Clone policy**: `LockState` derives Clone (via derive macro), matching previous `bool` behavior

## Non-goals

- [ ] Adding new lock behaviors (e.g., lock by user, lock expiration)
- [ ] Changing serialization format beyond backwards-compatible `locked: bool`
- [ ] Adding runtime validation that was not previously present (the boolean was already trusted)

---

## Reference: Affected Files (70+ references)

Based on grep analysis, these modules contain references to `.locked`:

1. **Models** (document.rs, hashing.rs, selection.rs, multi_select.rs, projection/policy.rs)
2. **Core** (transform.rs, nudge.rs, z_order.rs, grouping/)
3. **Layout** (grid.rs, dag.rs)
4. **UI Canvas** (canvas.rs, interaction_reducer.rs, drag_math.rs, selection_geometry.rs)
5. **UI Commands** (selection.rs, distribution.rs, alignment.rs)
6. **UI Properties** (properties.rs)
7. **Tests** (transform_tests.rs, selection_tests.rs, multi_select_tests.rs, etc.)
8. **Test Harness** (test_harness.rs)

Each reference must be updated according to the contract above.

(End of file - total 200 lines)
