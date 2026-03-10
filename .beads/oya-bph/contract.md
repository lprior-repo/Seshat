# Contract: Container Bounds Recomputation (GEO-025)

## EARS Requirements

### EARS-001: Container Bounds Must Track Children
**E** - When children of a subgraph container are transformed (moved, resized, rotated),
**A** - the container's bounds must be automatically recomputed to encompass all children,
**R** - to maintain geometric consistency and prevent visual drift,
**S** - so that container bounds remain stable as children move within them.

### EARS-002: Compute Function Required
**E** - Given a container node and its children nodes,
**A** - a `compute_subgraph_bounds()` function must compute the minimal bounding rectangle,
**R** - using the children's geometric extents (x, y, width, height),
**S** - to provide an authoritative bounds calculation for integration into transform operations.

### EARS-003: Integration with Child Transforms
**E** - When child nodes within a subgraph are transformed (move, resize),
**A** - the container bounds must be recomputed and updated,
**R** - immediately after the transform completes,
**S** - so that subsequent operations use accurate container bounds.

## KIRK Contracts

### KIRK-001: compute_subgraph_bounds Function Contract

```rust
/// Computes the bounding box of a subgraph container based on its children.
///
/// # Parameters
/// - `container_id`: The NodeId of the subgraph container
/// - `nodes`: Reference to the document nodes HashMap
///
/// # Returns
/// - `Some((x, y, width, height))`: The computed bounds if children exist
/// - `None`: If the container has no children or is not a container
///
/// # Preconditions
/// - container_id must reference a valid Node
/// - nodes must contain all child nodes referenced by container
///
/// # Postconditions
/// - Returns bounds that encompass ALL children geometrically
/// - Returns None if children list is empty
/// - Bounds are minimal (tight fit to children)
/// - All coordinate values are finite (not NaN/Infinity)
```

### KIRK-002: Transform Integration Contract

```rust
/// After any child transform within a container:
/// 1. Identify all containers that are ancestors of transformed nodes
/// 2. For each container, compute new bounds via compute_subgraph_bounds()
/// 3. Update container's x, y, width, height to match computed bounds
/// 4. Preserve container's position if it has explicit bounds (manual resize)
```

### KIRK-003: Empty Container Contract
- Container with no children: bounds remain unchanged or use default
- Container added to empty container: bounds initialized from child

## Error Taxonomy

| Error Code | Condition | Recovery |
|------------|-----------|----------|
| `NoChildren` | Container has no child nodes | Return None, leave bounds unchanged |
| `InvalidChildBounds` | Child has NaN/Infinity coordinates | Skip invalid child, warn |
| `ContainerNotFound` | Container ID not in nodes map | Return None |

## Illegal States

1. **Container smaller than child**: Container bounds MUST always encompass all children
2. **Orphaned children**: Children without valid parent reference
3. **Infinite bounds**: Container with infinite or NaN coordinates is invalid
