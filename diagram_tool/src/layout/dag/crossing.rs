use diagram_models::document::NodeId;
use petgraph::graph::{DiGraph, NodeIndex};

/// Barycentre of `node` neighbours found in `ref_pos`.
pub fn barycentre<S: ::std::hash::BuildHasher>(
    node: NodeIndex,
    ref_pos: &std::collections::HashMap<NodeIndex, f64, S>,
    graph: &DiGraph<NodeId, ()>,
    dir: petgraph::Direction,
) -> f64 {
    let neighbours: Vec<f64> = graph
        .neighbors_directed(node, dir)
        .filter_map(|n| ref_pos.get(&n).copied())
        .collect();

    if neighbours.is_empty() {
        f64::MAX
    } else {
        neighbours.iter().sum::<f64>() / neighbours.len() as f64
    }
}

/// 4-sweep barycentric crossing minimisation.
pub fn barycenter_sweep(
    layers: Vec<Vec<NodeIndex>>,
    graph: &DiGraph<NodeId, ()>,
) -> Vec<Vec<NodeIndex>> {
    [true, false, true, false]
        .into_iter()
        .fold(layers, |acc, is_forward| {
            let n = acc.len();
            if is_forward {
                // forward: fix layer l-1, reorder l
                (1..n).fold(acc, |mut ls, l| {
                    let ref_pos: std::collections::HashMap<NodeIndex, f64> = ls[l - 1]
                        .iter()
                        .enumerate()
                        .map(|(i, &node)| (node, i as f64))
                        .collect();
                    ls[l].sort_by(|&a, &b| {
                        barycentre(a, &ref_pos, graph, petgraph::Direction::Incoming)
                            .partial_cmp(&barycentre(
                                b,
                                &ref_pos,
                                graph,
                                petgraph::Direction::Incoming,
                            ))
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                    ls
                })
            } else {
                // backward: fix layer l+1, reorder l
                (0..n.saturating_sub(1)).rev().fold(acc, |mut ls, l| {
                    let ref_pos: std::collections::HashMap<NodeIndex, f64> = ls[l + 1]
                        .iter()
                        .enumerate()
                        .map(|(i, &node)| (node, i as f64))
                        .collect();
                    ls[l].sort_by(|&a, &b| {
                        barycentre(a, &ref_pos, graph, petgraph::Direction::Outgoing)
                            .partial_cmp(&barycentre(
                                b,
                                &ref_pos,
                                graph,
                                petgraph::Direction::Outgoing,
                            ))
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                    ls
                })
            }
        })
}
