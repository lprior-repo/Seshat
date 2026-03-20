# Graph Layout & Traversal (`petgraph`)

Diagrams are fundamentally Directed Acyclic Graphs (DAGs). While our React-like UI manages the visual representation of nodes and edges, the mathematical structure of the diagram is analyzed and computed using `petgraph`.

## Why We Use `petgraph`
We need to rapidly answer complex architectural queries that humans take for granted, but computers find difficult:
1. **Cycle Detection:** If I link "Database" to "API", does it create an infinite loop?
2. **Auto-Arrange:** How can I lay out these 50 disconnected nodes so they look like a clean top-down architecture diagram?
3. **Sub-graph Selection:** If I click "Select All Dependencies," what nodes are downstream of this one?

## The Adapter Pattern
Our `DiagramDocument` stores data in an `im::HashMap<NodeId, Node>`. `petgraph` requires data in a `GraphMap` or `DiGraph` using internal numerical indices (`NodeIndex`).

We use the **Adapter Pattern** to build a transient `petgraph` representation on the fly during calculations. 

```rust
use petgraph::graph::DiGraph;
use std::collections::HashMap;

// Transient calculation function in `core/`
pub fn build_petgraph(doc: &DiagramDocument) -> (DiGraph<NodeId, EdgeId>, HashMap<NodeId, NodeIndex>) {
    let mut graph = DiGraph::new();
    let mut id_to_index = HashMap::new();
    
    for (id, _node) in &doc.nodes {
        let idx = graph.add_node(id.clone());
        id_to_index.insert(id.clone(), idx);
    }
    
    for (edge_id, edge) in &doc.edges {
        if let (Some(&src), Some(&dst)) = (id_to_index.get(&edge.source), id_to_index.get(&edge.target)) {
            graph.add_edge(src, dst, edge_id.clone());
        }
    }
    
    (graph, id_to_index)
}
```

## Cycle Detection (Preventing Illegal States)
Before allowing an AI or human to create a new connection, we validate it using `petgraph::algo::is_cyclic_directed`.
If adding the edge would cause a cycle, the pure calculation layer returns an explicit error (`ValidationError::DagCycle`), and the action is rejected before it ever hits the Undo stack or the Database.

## DAG Layout Algorithms
For Auto-Arrange, we extract the `petgraph`, run a topological sort (`petgraph::algo::toposort`), and assign horizontal/vertical bands based on depth. We then map those logical coordinates back into our `OrderedFloat` domain types and return a `DiagramDocument` with the updated X/Y coordinates. This is a pure Calc step, completely isolated from Dioxus rendering.