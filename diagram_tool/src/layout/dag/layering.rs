use diagram_models::document::NodeId;
use petgraph::graph::{DiGraph, NodeIndex};

/// Longest-path layer assignment.  Returns layers as `Vec<Vec<NodeIndex>>`.
pub(crate) fn assign_layers(
    topo_order: &[NodeIndex],
    graph: &DiGraph<NodeId, ()>,
) -> Vec<Vec<NodeIndex>> {
    let layer_map: std::collections::HashMap<NodeIndex, usize> =
        topo_order
            .iter()
            .fold(std::collections::HashMap::new(), |mut m, &idx| {
                let layer = graph
                    .neighbors_directed(idx, petgraph::Direction::Incoming)
                    .filter_map(|p| m.get(&p).copied())
                    .max()
                    .map_or(0, |max_pred| max_pred + 1);
                m.insert(idx, layer);
                m
            });

    let max_layer = layer_map.values().copied().max().map_or(0, |x| x);

    (0..=max_layer)
        .map(|layer| {
            topo_order
                .iter()
                .filter(|&&idx| layer_map.get(&idx).copied().map_or(0, |x| x) == layer)
                .copied()
                .collect()
        })
        .collect()
}
