# Contract Specification

## Context
- Feature: Edge Port Anchors (EDG-001 to EDG-005)
- Domain terms:
  - **Edge**: A connection between a source Node and a target Node.
  - **Port Anchor**: A specific attachment point on a node (e.g., Top, Bottom, Left, Right, Center, or a Custom relative offset) where an edge endpoint binds.
- Assumptions:
  - If an edge has no port anchor specified (or it's `None`), it falls back to the default dynamic routing (e.g., bounding-box intersection).
  - Custom port anchors use normalized coordinates (0.0 to 1.0) relative to the node's bounding box.
- Open questions:
  - Should edges bound to a custom port anchor adjust their attachment point if the node resizes significantly? (Assuming normalized 0.0-1.0 coords handle this gracefully).
  - Can subgraphs have port anchors? (Assuming yes, they are treated as nodes).

## Preconditions
- [P1] When specifying a Custom port anchor, the normalized coordinates (x, y) must be finite.
- [P2] When specifying a Custom port anchor, the normalized coordinates (x, y) must be within the valid range [0.0, 1.0].
- [P3] Edge endpoints (source and target) must resolve to valid nodes in the document to compute the absolute anchor position.

## Postconditions
- [Q1] Binding an edge to a port updates the edge's `source_port` or `target_port` fields in the document state.
- [Q2] Edges with explicitly defined port anchors route their path starting/ending exactly at the computed absolute coordinate of the port on the bound node.
- [Q3] Moving a node updates the absolute position of its connected edges' port anchors.
- [Q4] Serializing and deserializing an edge preserves its port anchor configurations.

## Invariants
- [I1] An edge's port anchor (if defined) is always relative to its connected node's current bounds.

## Error Taxonomy
- `Error::InvalidPortOffset` - when a custom port anchor has coordinates outside [0.0, 1.0] or non-finite.
- `Error::NodeNotFound` - when trying to attach an edge to a port on a non-existent node.

## Contract Signatures
- `fn set_edge_source_port(edge_id: &EdgeId, port: Option<PortAnchor>) -> Result<(), Error>`
- `fn set_edge_target_port(edge_id: &EdgeId, port: Option<PortAnchor>) -> Result<(), Error>`
- `fn compute_port_absolute_position(node: &Node, port: &PortAnchor) -> Point`

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Custom port coords finite | Compile-time | `OrderedFloat` for x, y fields |
| P2: Custom port coords [0.0, 1.0] | Runtime-checked constructor | `NormalizedOffset::new(x, y) -> Result<Self, Error>` |
| P3: Nodes exist | Error variant | `Result<T, Error::NodeNotFound>` |

## Violation Examples (REQUIRED -- one per precondition and postcondition)
- VIOLATES P1: `NormalizedOffset::new(f64::NAN, 0.0)` -- should produce `Err(Error::InvalidPortOffset)` or fail to compile if using strict `OrderedFloat` constructors.
- VIOLATES P2: `NormalizedOffset::new(OrderedFloat(1.5), OrderedFloat(0.5))` -- should produce `Err(Error::InvalidPortOffset)`.
- VIOLATES P3: `document.set_edge_source_port(&EdgeId("edge_1"), Some(PortAnchor::Top))` where "edge_1" connects to a deleted node -- should produce `Err(Error::NodeNotFound)`.
- VIOLATES Q1: After `set_edge_source_port(&edge_1, Some(PortAnchor::Bottom))`, `document.get_edge(&edge_1).source_port == None` -- should produce test failure.
- VIOLATES Q2: After setting source port to `Top`, the routed edge's start point is at the node's `Center` -- should produce test failure.
- VIOLATES Q3: Moving node by (10, 10) does not translate the edge's computed start point by (10, 10) -- should produce test failure.
- VIOLATES Q4: Deserializing JSON with `"source_port": "top"` results in `None` -- should produce test failure.

## Ownership Contracts (Rust-specific)
- Shared borrow: `fn compute_port_absolute_position(node: &Node, port: &PortAnchor)` -- reads node geometry, no mutation, returns a new `Point`.
- Exclusive borrow: `fn set_edge_source_port(&mut self, edge_id: &EdgeId, port: Option<PortAnchor>)` -- mutates the `Edge` in the document, updating its `source_port` field.

## Non-goals
- Advanced routing algorithms around obstacles (handled by other routing features).
- Creating custom visually-rendered port "handles" (this is strictly about the logic/model of attachment, UI handles are separate).