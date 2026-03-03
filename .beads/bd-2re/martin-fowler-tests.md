# Martin Fowler Test Catalog: bd-2re - Edge Binding Tests

**Bead ID**: bd-2re
**Test Strategy**: Edge-Correspondence, State-Verification, and Contract Tests

## Test Philosophy

Following Martin Fowler's test categories from "The Practical Test Pyramid":

1. **Contract Tests**: Verify edge model serialization/deserialization
2. **Edge Cases**: Boundary conditions, empty inputs, extreme values
3. **State Verification**: Document state changes after operations
4. **Correspondence Tests**: Visual rendering matches data model

## Test Catalog by Category

### 1. Contract Tests (Serialization)

#### Test: Edge Roundtrip Serialization
**ID**: EDG-007
**Purpose**: Verify edge serializes to JSON and deserializes back correctly
**Given**: An edge with all properties set
**When**: Edge is serialized to JSON and deserialized back
**Then**: All properties match original values
**Evidence**: JSON roundtrip preserves source, target, label, style, arrow_type

```typescript
test("EDG-007: edge serializes/deserializes correctly @baseline", async ({ page }) => {
  // Create edge with all properties
  // Serialize to JSON
  // Deserialize back
  // Assert all properties match
});
```

#### Test: Legacy Arrowhead Key Compatibility
**Purpose**: Verify edges with old "arrowhead" key deserialize correctly
**Given**: JSON edge with "arrowhead" key (legacy format)
**When**: JSON is deserialized
**Then**: arrow_type is set correctly from legacy key

### 2. State Verification Tests

#### Test: Edge Creation Updates Document
**ID**: EDG-004
**Purpose**: Verify creating an edge updates the document model
**Given**: A canvas with 2 nodes
**When**: User creates edge between nodes
**Then**: Document contains exactly 1 edge with correct source/target

```typescript
test("EDG-004: edge creation updates document state @baseline", async ({ page }) => {
  const beforeCount = await edgeCount(page);
  // Create edge
  const afterCount = await edgeCount(page);
  expect(afterCount).toBe(beforeCount + 1);
  // Verify source and target IDs in document
});
```

#### Test: Edge Deletion Removes from Document
**ID**: EDG-005
**Purpose**: Verify deleting an edge removes it from document
**Given**: A canvas with 1 edge
**When**: User deletes the edge
**Then**: Document contains 0 edges

### 3. Correspondence Tests (Visual ↔ Model)

#### Test: Edge Visual Position Matches Model
**Purpose**: Verify edge rendering matches source/target node positions
**Given**: Two nodes at known positions
**When**: Edge is created between nodes
**Then**: Edge visual connects node centers (or appropriate anchor points)
**Evidence**: Hit-testing at midpoint selects the edge

#### Test: Edge Binding Maintained During Node Drag
**ID**: EDG-015
**Purpose**: Verify edge follows node when node is dragged
**Given**: An edge connecting two nodes
**When**: Source node is dragged to new position
**Then**: Edge endpoint follows node, edge still selectable
**Evidence**: Edge count unchanged, hit-test works at new position

```typescript
test("EDG-015: edge endpoint follows node during drag @baseline", async ({ page }) => {
  // Create nodes and edge
  const edgeBefore = await edgeCount(page);
  // Drag source node
  // Verify edge still exists and is selectable
  expect(await edgeCount(page)).toBe(edgeBefore);
});
```

### 4. Edge Case Tests

#### Test: Self-Loop Rejection
**ID**: EDG-002
**Purpose**: Verify edge from node to itself is rejected in DAG mode
**Given**: A canvas with 1 node
**When**: User attempts to create self-loop edge
**Then**: Edge is not created, edge count remains 0
**Evidence**: No console errors, graceful rejection

```typescript
test("EDG-002: edge rejects self-loop in dag mode @baseline", async ({ page }) => {
  // Click same node twice
  // Assert edge count is 0
  await expectEdgeCount(page, 0);
});
```

#### Test: Cycle Formation Rejection
**ID**: EDG-003
**Purpose**: Verify cycle-forming edges are rejected in DAG mode
**Given**: A chain A → B → C
**When**: User attempts to create C → A
**Then**: Edge is not created
**Evidence**: Edge count unchanged, no cycles in document

#### Test: Empty/Invalid Node IDs
**Purpose**: Verify edge handles empty node IDs gracefully
**Given**: Edge with empty source/target
**When**: Edge is accessed or rendered
**Then**: No crashes, graceful handling

### 5. Determinism Tests

#### Test: Overlapping Edge Hit-Selection
**ID**: EDG-016, EDG-024, EDG-025
**Purpose**: Verify hit-selection is deterministic across clicks
**Given**: Two overlapping edges
**When**: User clicks overlap point multiple times
**Then**: Same edge is selected each time (consistent ordering)
**Evidence**: Delete reveals same edge each time, undo/redo preserves ordering

```typescript
test("EDG-024: horizontal edge overlap hit-selection is deterministic @baseline", async ({ page }) => {
  // Create overlapping edges
  const selectedIds = [];
  for (let i = 0; i < 3; i++) {
    // Click overlap point
    // Delete edge and record which remains
    selectedIds.push(remainingEdgeId);
    // Undo
  }
  // All selections should be same edge
  expect(selectedIds[0]).toBe(selectedIds[1]);
  expect(selectedIds[1]).toBe(selectedIds[2]);
});
```

#### Test: Zoom-Level Hit-Testing
**ID**: EDG-018, EDG-019
**Purpose**: Verify thin edges remain selectable across zoom levels
**Given**: A thin horizontal/vertical edge
**When**: User zooms in/out and attempts to select edge
**Then**: Edge is selectable at all zoom levels (50%, 100%, 200%, 300%)
**Evidence**: Hit-test succeeds at midpoint for all zoom levels

```typescript
test("EDG-018: thin horizontal edge remains selectable across zoom levels @baseline", async ({ page }) => {
  // Create horizontal edge
  const zoomLevels = [50, 100, 200, 300];
  for (const zoom of zoomLevels) {
    await setZoom(page, zoom);
    await clickEdgeAtMidpoint(page);
    expect(await selectedCount(page)).toBe(1);
  }
});
```

### 6. Routing Tests

#### Test: Curved Edge Bezier Path Hit-Testing
**ID**: EDG-026
**Purpose**: Verify curved edges are hittable along bezier curve
**Given**: A curved edge between horizontally aligned nodes
**When**: User clicks along bezier path (not just straight line)
**Then**: Edge is selected at curve peak and midpoint
**Evidence**: Hit-test at calculated control point offset

```typescript
test("EDG-026: curved edge is hittable along quadratic bezier path @baseline", async ({ page }) => {
  // Create curved edge
  // Calculate bezier control point
  const curvePeakY = midY - dx * 0.25;
  // Click at curve peak
  await clickAt(page, midX, curvePeakY);
  expect(await selectedCount(page)).toBe(1);
});
```

#### Test: Step-Routed Edge Segment Hit-Testing
**ID**: EDG-027
**Purpose**: Verify step edges are hittable on all segments
**Given**: A step-routed edge (horizontal → vertical → horizontal)
**When**: User clicks on vertical segment or corners
**Then**: Edge is selected on all segments
**Evidence**: Hit-test at midpoint of vertical segment succeeds

### 7. Container Tests

#### Test: Edge Within Container
**ID**: EDG-021
**Purpose**: Verify edges work when both nodes are in same container
**Given**: Two nodes inside a subgraph container
**When**: User creates edge between nodes
**Then**: Edge is created and renders correctly
**Evidence**: Edge count = 1, edge is selectable

#### Test: Edge Crossing Container Boundary
**ID**: EDG-022
**Purpose**: Verify edges work when nodes are in different containers
**Given**: Node A inside container, Node B outside
**When**: User creates edge from A to B
**Then**: Edge is created and renders correctly crossing boundary
**Evidence**: Edge count = 1, edge is selectable

#### Test: Reparent Node With Connected Edge
**ID**: EDG-023
**Purpose**: Verify moving node out of container maintains edge
**Given**: Node A inside container with edge to Node B
**When**: User drags Node A outside container
**Then**: Edge remains connected, edge is not orphaned
**Evidence**: Edge count = 1 after reparent, edge is selectable

### 8. Transformation Tests

#### Test: Resize Selection With Edges
**ID**: EDG-013
**Purpose**: Verify resizing nodes maintains edge bindings
**Given**: Two connected nodes in a selection
**When**: User resizes the selection
**Then**: Edge remains connected to nodes
**Evidence**: Edge count unchanged, edge is selectable after resize

#### Test: Edge Selection Without Node Selection
**ID**: EDG-014
**Purpose**: Verify clicking edge selects only the edge
**Given**: An edge connecting two nodes
**When**: User clicks on edge midpoint
**Then**: Only edge is selected, nodes are not selected
**Evidence**: selectedCount = 1, edge is selected

## Test Smells to Avoid

### 1. Fragile Tests
- ❌ Hardcoded coordinates that break with layout changes
- ✅ Use nodeCenters() to calculate positions dynamically

### 2. Brittle Timing
- ❌ Fixed timeouts that may fail on slow machines
- ✅ Use waitForUiReady() and waitForNoRebuildOverlay()

### 3. Test Interdependence
- ❌ Tests that depend on execution order
- ✅ Each test calls freshStart() for isolation

### 4. Overly Specific Assertions
- ❌ Asserting exact pixel values
- ✅ Assert ranges and relationships (e.g., "within 30px")

## Coverage Matrix

| EDG ID | Category | Implemented | Test File |
|--------|----------|-------------|-----------|
| EDG-001 | Basic | ✅ | edges-and-routing.spec.ts |
| EDG-002 | Basic | ✅ | edges-and-routing.spec.ts |
| EDG-003 | Basic | ✅ | edges-and-routing.spec.ts |
| EDG-004 | State | ❌ | TBD |
| EDG-005 | State | ❌ | TBD |
| EDG-006 | State | ❌ | TBD |
| EDG-007 | Contract | ✅ | document.rs (Rust test) |
| EDG-008 | Basic | ❌ | TBD |
| EDG-009 | Basic | ❌ | TBD |
| EDG-010 | Basic | ❌ | TBD |
| EDG-011 | Transform | ⚠️ Skipped | edge-binding-2.spec.ts |
| EDG-012 | Transform | ⚠️ Skipped | edge-binding-2.spec.ts |
| EDG-013 | Transform | ✅ | edge-binding-2.spec.ts |
| EDG-014 | Selection | ✅ | edge-binding-2.spec.ts |
| EDG-015 | Binding | ✅ | edge-binding-2.spec.ts |
| EDG-016 | Determinism | ✅ | edges-and-routing.spec.ts |
| EDG-017 | Determinism | ✅ | edges-and-routing.spec.ts |
| EDG-018 | Zoom | ✅ | edges-and-routing.spec.ts |
| EDG-019 | Zoom | ✅ | edges-and-routing.spec.ts |
| EDG-020 | Selection | ✅ | edges-and-routing.spec.ts |
| EDG-021 | Container | ✅ | edges-and-routing.spec.ts |
| EDG-022 | Container | ✅ | edges-and-routing.spec.ts |
| EDG-023 | Container | ✅ | edges-and-routing.spec.ts |
| EDG-024 | Determinism | ✅ | edges-and-routing.spec.ts |
| EDG-025 | Determinism | ✅ | edges-and-routing.spec.ts |
| EDG-026 | Routing | ✅ | edges-and-routing.spec.ts |
| EDG-027 | Routing | ✅ | edges-and-routing.spec.ts |
| EDG-028 | Routing | ✅ | edges-and-routing.spec.ts |
| EDG-029 | Routing | ✅ | edges-and-routing.spec.ts |
| EDG-030 | Routing | ✅ | edges-and-routing.spec.ts |
| EDG-031 | Advanced | ❌ | TBD |
| EDG-032 | Advanced | ❌ | TBD |
| EDG-033 | Advanced | ❌ | TBD |
| EDG-034 | Advanced | ❌ | TBD |
| EDG-035 | Advanced | ❌ | TBD |

**Legend**: ✅ Implemented | ⚠️ Partial/Skipped | ❌ Missing

## Next Steps

1. Implement missing tests (EDG-004 to EDG-010, EDG-031 to EDG-035)
2. Fix rotation tests (EDG-011, EDG-012) when rotation controls are implemented
3. Run all tests and verify zero unwrap/panic
4. Create verification artifacts
