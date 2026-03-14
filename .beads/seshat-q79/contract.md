# Contract Specification: Straight-Line Edge Routing (seshat-q79)

## Context
- Feature: Straight-line edge routing between port coordinates.
- Domain terms: PortAnchor, Point, Node, Edge, Straight-Line Route.
- Assumptions: Nodes have finite dimensions and coordinates.
- Open questions: Should we handle self-loops specially for straight lines? (Assuming they will just be zero-length or overlapping for now).

## Scope Map
- `diagram_tool/src/models/routing.rs` (Creation)
- `diagram_tool/src/models/mod.rs` (Registration)

## Preconditions
- [ ] P1: Source node referenced by `edge.source` must exist in `doc`.
- [ ] P2: Target node referenced by `edge.target` must exist in `doc`.

## Postconditions
- [ ] Q1: Start point corresponds to `edge.source_port` (or center) absolute position.
- [ ] Q2: End point corresponds to `edge.target_port` (or center) absolute position.
- [ ] Q3: The returned points must have finite coordinates (no NaN/Inf).

## Invariants
- [ ] I1: Node positions and dimensions remain unchanged.
- [ ] I2: Edge properties remain unchanged.

## Error Taxonomy
- `RoutingError::SourceNotFound(NodeId)` - when source node is missing from document.
- `RoutingError::TargetNotFound(NodeId)` - when target node is missing from document.

## Contract Signatures
- `pub fn compute_straight_line_route(doc: &DiagramDocument, edge: &Edge) -> Result<(Point, Point), RoutingError>`

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| Source exists | Error variant | `Result<..., RoutingError::SourceNotFound>` |
| Target exists | Error variant | `Result<..., RoutingError::TargetNotFound>` |

## Violation Examples
- VIOLATES <P1>: `compute_straight_line_route(doc_missing_n1, edge_from_n1)` -- should produce `Err(RoutingError::SourceNotFound("n1"))`
- VIOLATES <P2>: `compute_straight_line_route(doc_missing_n2, edge_to_n2)` -- should produce `Err(RoutingError::TargetNotFound("n2"))`

## Ownership Contracts
- `doc: &DiagramDocument`: Shared borrow, read-only.
- `edge: &Edge`: Shared borrow, read-only.
- Returns `(Point, Point)` which are owned values.

## Non-goals
- Multi-segment routing (Manhattan/orthogonal).
- Obstacle avoidance.
- Bend point handling (this bead focuses on basic straight lines).
