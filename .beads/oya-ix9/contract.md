# Contract: Freehand Drawing with Path Simplification (GEO-027)

## Meta
- **Date:** 2026-03-09
- **Bead:** oya-ix9
- **Test ID:** GEO-027
- **Quality Score:** 85%
- **Status:** Ready for Implementation
- **Scope Level:** Feature Extension

---

## 1. Problem Statement

Currently, the diagram tool has no freehand/draw capability. Users cannot sketch freeform paths on the canvas. We need to add a Draw tool that captures pointer input, creates a path shape, and applies Ramer-Douglas-Peucker path simplification to reduce point count while preserving visual fidelity and avoiding self-intersection artifacts.

### Context
- **Who:** Diagram tool users who want to sketch ideas quickly
- **What:** No freehand drawing tool exists - users must use pre-defined shapes
- **Impact:** Cannot sketch annotations, diagrams, or rough concepts

### Scope

**IN scope:**
- New "Draw" tool mode in toolbar
- Pointer capture during freehand drawing
- Path shape type (new node kind)
- Ramer-Douglas-Peucker simplification algorithm
- Endpoints preservation invariant
- Self-intersection spike prevention

**OUT of scope:**
- Smooth bezier curve fitting (future enhancement)
- Pressure sensitivity (tablet support)
- Path editing after creation (future enhancement)

---

## 2. EARS Requirements

### 2.1 Ubiquitous
- THE SYSTEM SHALL provide a Draw tool option in the toolbar
- THE SYSTEM SHALL render freehand paths as stroke shapes on the canvas

### 2.2 Event-Driven
- WHEN the user selects the Draw tool and clicks on the canvas, the system SHALL begin capturing pointer movement
- WHEN the user moves the pointer while Draw tool is active and pointer is captured, the system SHALL record point positions
- WHEN the user releases the pointer, the system SHALL simplify the captured path and create a node

### 2.3 State-Driven
- WHILE the Draw tool is active and pointer is captured, the system SHALL display a live preview of the path being drawn
- WHILE the path is being simplified, the system SHALL show a loading indicator if simplification takes >100ms

### 2.4 Optional
- WHERE the captured path has fewer than 3 points, the system SHALL NOT create a shape (treat as click, not draw)

### 2.5 Unwanted
- IF the simplified path introduces self-intersection spikes, THEN THE SYSTEM SHALL NOT create the shape and SHALL notify the user
- IF the simplified path loses both endpoints, THEN THE SYSTEM SHALL NOT create the shape
- IF the simplification results in a path with 0 or 1 points, THEN THE SYSTEM SHALL NOT create the shape

---

## 3. Domain Model

### 3.1 Entities

| Entity | Key Fields | Relationships |
|--------|-----------|---------------|
| PathNode | id, points: Vec<Point>, simplified_points: Vec<Point>, stroke_color, stroke_width | belongs_to Document |
| DrawToolState | tool_mode: ToolMode::Draw, is_capturing: bool, current_path: Vec<Point> | transient during draw |

### 3.2 Value Objects

| Value Object | Fields | Validation Rules |
|-------------|--------|-----------------|
| Point | x: f64, y: f64 | finite, not NaN |
| PathSimplificationConfig | epsilon: f64, min_points: usize | epsilon >= 0, min_points >= 2 |

### 3.3 States and Transitions

```
DrawTool States: Idle -> Capturing -> Simplifying -> Complete

Legal Transitions:
  Idle -> Capturing: pointer_down on canvas with Draw tool
  Capturing -> Simplifying: pointer_up event
  Simplifying -> Idle: simplification complete, node created OR rejected
  Capturing -> Idle: ESC key or tool switch
```

### 3.4 Illegal States

| Illegal State | Why Illegal | Prevention |
|--------------|-------------|-------------|
| Capturing without active Draw tool | State machine violation | Check tool mode before state transition |
| Simplifying with empty points | Invalid input | Precondition check before simplification |
| Simplified path loses both endpoints | UX violation - user loses anchor points | Postcondition verification |

---

## 4. KIRK Contracts

### Component: PathSimplifier

**Preconditions:**
| # | Condition | Enforcement | Violation Error |
|---|-----------|-------------|-----------------|
| P1 | Input points length >= 2 | Runtime check | Err(PathError::InsufficientPoints) |
| P2 | All points are finite (not NaN/Inf) | Runtime validation | Err(PathError::InvalidPoint) |
| P3 | epsilon >= 0 | Runtime validation | Err(PathError::InvalidEpsilon) |

**Postconditions:**
| # | Guarantee | Verification |
|---|-----------|-------------|
| Q1 | Output has >= 2 points OR simplification was rejected | Check output length |
| Q2 | First and last points match input first and last | Compare endpoints |
| Q3 | No self-intersection spikes introduced | Geometric validation |

**Invariants:**
| # | Condition | Enforcement | Broken During |
|---|-----------|-------------|---------------|
| I1 | Start and end points preserved | Postcondition check | Never |
| I2 | Path visual fidelity within epsilon | Algorithm guarantees | Never |

### Component: DrawTool

**Preconditions:**
| # | Condition | Enforcement | Violation Error |
|---|-----------|-------------|-----------------|
| P1 | ToolMode == Draw | Runtime state check | N/A - UI prevents |
| P2 | Canvas has focus | Event handler check | N/A - UI prevents |

**Postconditions:**
| # | Guarantee | Verification |
|---|-----------|-------------|
| Q1 | Node created iff path valid | Verify node in document |
| Q2 | Live preview shown during capture | UI verification |

---

## 5. Error Taxonomy

| Variant | When | User Message |
|---------|------|-------------|
| PathError::InsufficientPoints | < 3 points captured | "Path too short" |
| PathError::InvalidPoint | NaN/Inf in points | "Invalid path data" |
| PathError::SelfIntersection | Simplified path has spikes | "Path has invalid shape" |
| PathError::EpsilonTooSmall | epsilon <= 0 | Internal error |

---

## 6. Inversion Analysis

### 6.1 Security Inversions
- N/A - Drawing tool has no security implications

### 6.2 Usability Inversions

| Inversion | Applicable | Trigger | Response |
|-----------|------------|---------|----------|
| not-found | N | | |
| invalid-format | Y | NaN points in input | Reject with error |
| missing-required | Y | No points captured | Treat as click, not draw |
| duplicate | N | | |
| empty-result | Y | Path reduces to < 2 points | Reject with error |
| stale-data | N | | |
| invalid-transition | Y | Tool switch during capture | Cancel capture |

### 6.3 Integration Inversions

| Inversion | Applicable | Trigger | Response |
|-----------|------------|---------|----------|
| idempotency | N | | |
| timeout | Y | Simplification > 5s | Cancel and notify user |
| concurrent-modification | N | | |
| partial-failure | N | | |
| downstream-unavailable | N | | |

---

## 7. Second-Order Consequences

### Behavior: Path Simplification

**First Order:** Path points reduced, node created with simplified path

**Second Order:**
| # | Cascade Effect | Affected Component | Consequence Check |
|---|---------------|-------------------|-------------------|
| 1 | Node added to document | Document state | Verify node in document |
| 2 | Node appears on canvas | Canvas renderer | Visual verification |
| 3 | Undo operation must work | Undo system | Verify undo restores state |

---

## 8. Pre-Mortem

| # | Cause | Probability | Severity | Mitigation |
|---|-------|------------|----------|------------|
| 1 | Simplification creates ugly artifacts | MED | MED | Self-intersection detection |
| 2 | User can't draw at all | LOW | HIGH | Test on multiple browsers |
| 3 | Performance issues with many points | MED | LOW | Limit max points, use spatial sampling |

---

## 9. Acceptance Criteria

### 9.1 Happy Path
| # | Scenario | Given | When | Then |
|---|----------|-------|------|------|
| 1 | Basic draw | Draw tool selected | User draws path | Path shape created |
| 2 | Simplification | Path with 100+ points | On pointer up | Simplified path with fewer points |
| 3 | Endpoints | Any valid path | After simplification | First/last points unchanged |

### 9.2 Error Path
| # | Scenario | Given | When | Then |
|---|----------|-------|------|------|
| 1 | Too short | < 3 points | On pointer up | No shape created |
| 2 | Invalid points | NaN in points | During capture | Reject path |

### 9.3 Edge Cases
| # | Scenario | Given | When | Then |
|---|----------|-------|------|------|
| 1 | Self-intersection spike | Path that would spike | After simplification | Path rejected, user notified |
| 2 | Tool switch mid-draw | Capturing path | User switches to Select | Capture cancelled |

---

## 10. Implementation Notes

### Ramer-Douglas-Peucker Algorithm

The algorithm:
1. Connect first and last point with a line
2. Find point farthest from line (max distance)
3. If max distance > epsilon, recursively simplify sub-paths
4. If max distance <= epsilon, discard all intermediate points
5. Preserve first and last points always

### Self-Intersection Detection

After simplification, verify:
1. No segment crosses any other segment
2. No segment touches except at endpoints
3. If violation found, either reject or try alternative epsilon

---

## 11. Test Coverage Map

| Test ID | Coverage Area | Test Type |
|---------|--------------|-----------|
| GEO-027 | Path simplification | Unit |
| GEO-027-001 | Basic simplification | Unit |
| GEO-027-002 | Endpoint preservation | Unit |
| GEO-027-003 | No self-intersection spikes | Unit |
| GEO-027-004 | Too short path rejected | Unit |
| GEO-027-005 | Invalid points rejected | Unit |
