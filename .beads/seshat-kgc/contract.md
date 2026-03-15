# Contract Specification: Marquee Performance (seshat-kgc)

## Context
- Feature: Optimized marquee selection for large diagrams (3000+ nodes).
- Domain terms: 
    - SpatialIndex: A data structure to accelerate rectangular queries.
    - Marquee: A rectangular selection box.
    - ATDD DSL: Testing domain actions instead of internal structures.
- Assumptions:
    - Node positions and dimensions are provided by the diagram document.
    - Nodes can be rotated.
    - Performance target: 3000 node selection completes successfully (latency benchmarks handled by dedicated criterion suite, not pass/fail tests).

## Preconditions
- P1: Marquee rectangle bounds (width and height) must be non-negative.

## Postconditions
- Q1: The diagram correctly reports all nodes satisfying the bounds as selected.
- Q2: Resulting selection correctly applies `Contain` vs `Intersect` modes (exhaustive permutations: inside, partial, outside, edge, corner).
- Q3: Rotated nodes are accurately evaluated (unrotated, 90 deg, arbitrary angles).
- Q4: The observable document state (aside from selection) remains equivalent to the start state.

## Invariants
- INV1: Property-Based Testing Invariant: Optimized spatial index result must be perfectly identical to the linear scan result for any arbitrary layout.

## Error Taxonomy
- `Error::InvalidMarqueeBounds` - when marquee width or height is negative.

## Contract Signatures
- `impl DiagramDocument { pub fn select_marquee(&mut self, bounds: ValidRect, mode: MarqueeMode) -> Result<(), Error> }`

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| width >= 0, height >= 0 | Compile-time / Constructor | `ValidRect::new(x, y, w, h) -> Result<ValidRect, Error::InvalidMarqueeBounds>` |
| mode | Compile-time | `enum MarqueeMode { Contain, Intersect }` |

## Violation Examples
- VIOLATES P1: `ValidRect::new(0.0, 0.0, -10.0, 10.0)` -- should produce `Err(Error::InvalidMarqueeBounds)`
- VIOLATES Q1: (Implementation Defect) The diagram fails to report an enclosed node as selected -- should panic (test assertion failure).
- VIOLATES Q2: (Implementation Defect) A partially intersecting node is included when mode is Contain -- should panic (test assertion failure).
- VIOLATES Q3: (Implementation Defect) A rotated node within bounds is ignored -- should panic (test assertion failure).
- VIOLATES Q4: (Implementation Defect) Observable node coordinates differ from start state after selection -- should panic (test assertion failure).
- VIOLATES INV1: (Implementation Defect) Spatial index result differs from linear scan -- should panic (test assertion failure).

## Ownership Contracts
- `DiagramDocument::select_marquee(self: &mut DiagramDocument, bounds: ValidRect, mode: MarqueeMode)`
  - Borrows: `&mut self` to update selection state.
  - Mutates: The selection aspect of the document. The observable state of nodes and metadata remains unchanged.

## Non-goals
- Full virtualization of rendering (only focused on selection logic).
- Incremental index updates (rebuilding for now is acceptable if fast).