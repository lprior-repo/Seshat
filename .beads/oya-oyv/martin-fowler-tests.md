# GEO-026: Nested Container Bounds Propagation - Martin Fowler Tests

## Test Strategy
Using Given-When-Then format aligned with Martin Fowler's testing approach.

## Happy Path Tests

### GEO-026-H1: Single Child Move Updates Parent Container
**Given** a document with a container (subgraph) at position (50, 50) with size (200, 150) containing a child node at position (60, 60) with size (50, 40)
**When** the child node is moved to position (80, 80)
**Then** the parent container's bounds MUST be recomputed to encompass the new child position
**And** container x SHOULD be approximately 50 (min child x minus padding)
**And** container y SHOULD be approximately 50
**And** container width SHOULD encompass child at x=130 (80+50)
**And** container height SHOULD encompass child at y=120 (80+40)

### GEO-026-H2: Nested Container Bounds Update
**Given** a document with outer container containing inner container, containing a node
**When** the node is moved
**Then** inner container bounds MUST be updated first
**And** outer container bounds MUST then be updated to encompass inner container's new bounds

### GEO-026-H3: Multiple Children - Bounds Encompass All
**Given** a container with multiple children at different positions
**When** any child moves
**Then** the container bounds MUST encompass ALL children, not just the moved one

### GEO-026-H4: Child Moved Outside Container Expands Container
**Given** a container at (100, 100) with size (100, 100) containing a child at (110, 110)
**When** the child is moved to position (50, 50)
**Then** the container MUST expand to include the new child position at (50, 50)

## Unhappy Path Tests

### GEO-026-U1: Node Without Parent - No Propagation
**Given** a node at root level (no parent container)
**When** the node is moved
**Then** no bounds propagation occurs (no parent to update)

### GEO-026-U2: Container Without Children - Minimum Bounds
**Given** an empty container (no children)
**When** bounds are recomputed
**Then** the container SHOULD either maintain its current bounds or have minimum bounds

### GEO-026-U3: Deep Nesting - All Ancestors Updated
**Given** a 3-level nesting (A > B > C > node)
**When** the node moves
**Then** container C bounds MUST be updated
**And** container B bounds MUST be updated
**And** container A bounds MUST be updated

### GEO-026-U4: Move Child to Edge of Container
**Given** a container with a child
**When** child is moved to be just inside container edge
**Then** container bounds MUST correctly encompass the child

## Edge Cases

### GEO-026-E1: Container with Only Subgraph Children
**Given** a container that only contains subgraph children (no leaf nodes)
**When** a nested subgraph's bounds change
**Then** parent container MUST encompass the subgraph's bounds

### GEO-026-E2: Multiple Children Moving Simultaneously
**Given** multiple children being moved at once (e.g., multi-select drag)
**When** bounds are recomputed
**Then** the container MUST encompass ALL moved children positions

### GEO-026-E3: Zero-Size Child
**Given** a container with a zero-size child node
**When** bounds are computed
**Then** the zero-size child SHOULD be handled (point still contributes to bounds)

## Integration Tests

### GEO-026-I1: Bounds Propagation After Group Operation
**Given** nodes that are grouped into a new container
**When** the group operation completes
**Then** the new container's bounds MUST encompass all grouped nodes

### GEO-026-I2: Bounds Propagation After Ungroup Operation
**Given** a grouped container with children
**When** ungroup is executed
**Then** parent bounds (if any) MUST be recomputed to no longer include the removed container

### GEO-026-I3: Reparent Node - Bounds Update Both Containers
**Given** a node in container A
**When** the node is moved to container B
**Then** container A bounds MUST be recomputed (node removed)
**And** container B bounds MUST be recomputed (node added)
