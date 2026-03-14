bead_id: seshat-6pa
bead_title: Implement subgraph parent validation (SUB-001)
phase: contract
updated_at: 2026-03-14T18:00:00Z

scope:
  what: Validate that nodes can only be reparented to Subgraph nodes
  where: diagram_tool/src/models/validation.rs, diagram_tool/src/models/subgraph/reparenting.rs

preconditions:
  - P1: Parent node must exist in the canvas
  - P2: Parent node must be of kind Subgraph
  - P3: Child node must exist in the canvas

postconditions:
  - Q1: After successful reparenting, child's parent reference points to the new Subgraph
  - Q2: Validation returns no errors when parent is a valid Subgraph

error_taxonomy:
  - Error::InvalidNodeType - when parent is not a Subgraph
  - Error::NodeNotFound - when child or parent node doesn't exist
  - Error::CircularDependency - when reparenting would create a cycle
  - ValidationIssue::invalid_parent - when document has non-subgraph parent

traceability:
  - SUB-001: Subgraph parent validation
  - validation.rs:87-99: validate_document_data parent check
  - reparenting.rs:31-38: validate_parent_is_subgraph function
  - reparenting.rs:102-103: validate_parent_is_subgraph called in set_node_parent_ext
