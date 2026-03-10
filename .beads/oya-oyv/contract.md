# GEO-026: Nested Container Bounds Propagation - Contract

## EARS Requirements

### E1: Core Requirement
**When** a child node within a subgraph container is moved, **then** the parent container's bounds MUST be recomputed to encompass all children, **and** this propagation MUST continue up the entire parent chain for nested containers.

### E2: Nested Containers
**Given** a nested container hierarchy (container A contains container B, container B contains node C), **when** node C moves, **then** container B's bounds MUST be updated first, **then** container A's bounds MUST be updated to encompass the new position of B.

### E3: Bounds Computation
**When** recomputing container bounds, **then** the new bounds MUST equal the minimum bounding box that encompasses ALL descendant nodes (including nested containers), **with** optional padding if the container has visual padding.

### E4: Boundary Conditions
**When** a child is moved outside the current container bounds, **then** the container MUST expand to include the child's new position, **and** this expansion MUST trigger recalculation for all ancestors.

## KIRK Contracts

### K1: Parent Chain Traversal
```
traverse_parent_chain(node_id):
  current = node.parent
  while current is not None:
    yield current
    current = current.parent
```

### K2: Recompute Bounds for Single Container
```
recompute_container_bounds(container_id):
  children = find_all_descendants(container_id)
  bounds = compute_bounding_box(children)
  container.x = bounds.min_x - padding
  container.y = bounds.min_y - padding
  container.width = bounds.width + 2*padding
  container.height = bounds.height + 2*padding
```

### K3: Propagate Bounds Up Chain
```
propagate_bounds_to_ancestors(node_id):
  for parent_id in traverse_parent_chain(node_id):
    recompute_container_bounds(parent_id)
```

### K4: Find All Descendants
```
find_all_descendants(container_id):
  direct_children = nodes.filter(n => n.parent == container_id)
  descendants = direct_children
  for child in direct_children:
    if child.kind == Subgraph:
      descendants += find_all_descendants(child.id)
  return descendants
```

### K5: Update on Node Move
```
on_node_moved(node_id, old_position, new_position):
  propagate_bounds_to_ancestors(node_id)
```

## Preconditions
- The node exists and is part of the document
- The node has a parent (is inside a container), or has ancestors that are containers

## Postconditions
- All ancestor containers have bounds that encompass their descendants
- The container's x, y, width, height fields are updated
- The bounds include appropriate padding

## Invariants
- Container bounds MUST always contain all child nodes
- Nested containers MUST propagate bounds changes up the chain
- Empty containers SHOULD have minimum bounds (or be handled appropriately)
