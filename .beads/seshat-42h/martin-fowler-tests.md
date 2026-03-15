# Martin Fowler Tests: Node Hit Testing

## Given a document with multiple nodes
- **When** I click on the canvas where no node exists
- **Then** the selection is cleared

- **When** I click on a specific node
- **Then** that node becomes the only selected node

- **When** I click on an already selected node
- **Then** it remains selected

- **When** I click on a different node
- **Then** the old selection is cleared and the new node is selected