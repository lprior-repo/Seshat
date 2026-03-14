# Contract Bundle for seshat-1m9: Z-index Ordering

## Scope
- **What**: Strict z-index handling when nodes overlap or are brought to front (DOC-016 to DOC-020)
- **Where**: `diagram_tool/src/core/z_order.rs`, `diagram_tool/src/models/projection/ops/z_order.rs`
- **Bead ID**: seshat-1m9
- **Priority**: 2
- **Issue Type**: task

## Contract Clauses

### Preconditions
- P1: ValidZIndexRange - z_index values must be within i64::MIN to i64::MAX
- P2: NoOverflow - Z-order operations must not cause integer overflow
- P3: NodesExist - All node IDs in selection must exist in document
- P4: LayerSeparation - Z-order ops must respect subgraph vs node layer separation
- P5: LockedNodeHandling - Locked nodes should be filtered from selection

### Postconditions
- Q1: UniqueZIndexes - All nodes in same layer have unique z_index values
- Q2: SequentialIndexes - z-indexes are sequential (no gaps) within each layer
- Q3: RelativeOrderPreserved - Relative order of selected nodes is preserved
- Q4: BringForwardSwapCount - Each selected node swaps at most once
- Q5: SendBackwardSwapCount - Each selected node swaps at most once
- Q6: ZIndexAssignment - min_z + index assigned to each node in sorted order
- Q7: SelectionNotEmpty - No valid nodes selected returns false
- Q8: LockedNodesExcluded - Only unlocked nodes affected by z-order ops

### Invariants
- I1: ZIndexUniqueness - No two nodes of same kind have identical z_index
- I2: LayerIntegrity - Subgraphs and regular nodes maintain separate z-order sequences
- I3: BoundedZIndex - All z_index values remain within i64 bounds

## Error Taxonomy
- ZOrderError::NoNodesSpecified - input node ID slice is empty
- ZOrderError::AllNodesInvalid - none of specified node IDs exist
- ZOrderError::ZIndexOverflow - node count exceeds i64 capacity
- ZOrderError::NoChange - operation would not change any z-indexes

## Traceability

| Requirement | DOC-ID | Contract Clause | Test Coverage |
|-------------|--------|-----------------|---------------|
| Unique z-indexes | DOC-016 | Q1 | test_contract_postcondition_unique_z_indexes |
| Sequential without gaps | DOC-017 | Q2 | test_contract_postcondition_sequential_indexes |
| Relative order preserved | DOC-018 | Q3 | test_contract_postcondition_relative_order_preserved |
| Locked node handling | DOC-019 | Q8 | test_handles_locked_nodes_as_noops |
| Layer separation | DOC-020 | Q4, I2 | test_contract_precondition_layer_separation |

## Evaluation Protocol

1. Run existing z_order_tests.rs - all tests must pass
2. Run new contract verification tests
3. Run property-based tests to verify invariants
4. Verify no duplicate z-indexes: `cargo test z_index_uniqueness`
5. Verify sequential z-indexes: `cargo test sequential`
6. Verify relative order: `cargo test relative_order`
