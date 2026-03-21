use diagram_models::document::{DiagramDocument, NodeId};
use im::HashMap;
use itertools::Itertools;
use petgraph::graph::{DiGraph, NodeIndex};

/// Build a `petgraph::DiGraph` from the document's unlocked root nodes and
/// edges.  Returns the graph plus two maps:
///  - `NodeId` → `NodeIndex` (for edge insertion)
///  - a sorted `Vec<NodeId>` of the nodes that participate in layout
pub fn build_graph(
    doc: &DiagramDocument,
) -> (DiGraph<NodeId, ()>, HashMap<NodeId, NodeIndex>, Vec<NodeId>) {
    let layout_ids: Vec<NodeId> = doc
        .document
        .nodes
        .iter()
        .filter(|(_, n)| !n.lock_state.is_locked() && n.parent.is_none())
        .map(|(id, _)| id.clone())
        .sorted()
        .collect();

    // pre-compute index map from sorted position
    let id_to_idx: HashMap<NodeId, NodeIndex> = layout_ids
        .iter()
        .enumerate()
        .map(|(i, id)| (id.clone(), NodeIndex::new(i)))
        .collect();

    let graph = layout_ids
        .iter()
        .fold(DiGraph::<NodeId, ()>::new(), |mut g, id| {
            g.add_node(id.clone());
            g
        });

    let graph = doc.document.edges.values().fold(graph, |mut g, edge| {
        if let (Some(&src), Some(&tgt)) = (id_to_idx.get(&edge.source), id_to_idx.get(&edge.target))
        {
            g.add_edge(src, tgt, ());
        }
        g
    });

    (graph, id_to_idx, layout_ids)
}
