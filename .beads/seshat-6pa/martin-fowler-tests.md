# Martin Fowler Test Plan

## Happy Path Tests
- test_sub001_reparent_to_subgraph_succeeds
  Given: A canvas with a Subgraph node and a child Node
  When: set_node_parent is called with the Subgraph as parent
  Then: Operation succeeds and child's parent is updated

- test_sub001_validate_document_with_subgraph_parent_passes
  Given: A document where a node's parent is a Subgraph
  When: validate_document_data is called
  Then: No validation issues are returned

## Error Path Tests
- test_sub001_reparent_to_non_subgraph_fails
  Given: A canvas with a regular Node (not Subgraph) and another Node
  When: set_node_parent is called with the regular Node as parent
  Then: Returns Err(Error::InvalidNodeType)

- test_sub001_validate_document_with_non_subgraph_parent_fails
  Given: A document where a node's parent is NOT a Subgraph
  When: validate_document_data is called
  Then: Returns ValidationIssue with code "invalid-parent"

- test_sub001_reparent_to_nonexistent_parent_fails
  Given: A canvas with a child Node but no parent node
  When: set_node_parent is called with nonexistent parent ID
  Then: Returns Err(Error::NodeNotFound)

## Edge Case Tests
- test_sub001_reparent_nested_subgraph_to_subgraph_succeeds
  Given: A canvas with a nested Subgraph and another Subgraph
  When: set_node_parent is called to reparent one Subgraph to another
  Then: Operation succeeds (Subgraph-to-Subgraph is valid)

## Contract Verification Tests
- test_p2_parent_must_be_subgraph_violation
  Given: set_node_parent with non-subgraph parent
  When: Operation is attempted
  Then: Returns Err(Error::InvalidNodeType) -- NOT a panic

- test_p1_parent_must_exist_violation
  Given: set_node_parent with nonexistent parent
  When: Operation is attempted
  Then: Returns Err(Error::NodeNotFound)

- test_validation_invalid_parent_issue
  Given: Document with node having non-subgraph parent
  When: validate_document_data is called
  Then: Returns ValidationIssue with code "invalid-parent" and severity Error

## Given-When-Then Scenarios
### Scenario 1: Valid reparenting to Subgraph
Given: Canvas contains Subgraph "SG1" at (0,0) and Node "N1" at (10,10)
When: User calls set_node_parent("N1", "SG1", canvas)
Then: N1.parent becomes Some("SG1"), no errors returned

### Scenario 2: Invalid reparenting to regular Node
Given: Canvas contains Node "N1" at (0,0) and Node "N2" at (10,10)
When: User calls set_node_parent("N2", "N1", canvas)
Then: Returns Err(Error::InvalidNodeType), N2.parent unchanged

### Scenario 3: Document validation catches invalid parent
Given: Document with Node "N1" having parent "N2" where N2.kind = Node
When: validate_document_data is called
Then: Returns ValidationIssue { code: "invalid-parent", message: "Node N1 parent N2 is not a Subgraph" }
