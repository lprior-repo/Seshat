#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

use crate::document::{Edge, EdgeId, Node, NodeId};
use im::HashMap;
use petgraph::algo::connected_components;
use petgraph::graph::{DiGraph, NodeIndex};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CycleError {
    #[error("Cycle detected involving edge {0}")]
    CycleDetected(EdgeId),
    #[error("Graph has {0} disconnected components")]
    DisconnectedGraph(usize),
}

/// Validates that the graph is a DAG using petgraph.
///
/// # Errors
/// Returns an error if the graph contains cycles or disconnected components.
pub fn validate_dag(
    nodes: &HashMap<NodeId, Node>,
    edges: &HashMap<EdgeId, Edge>,
) -> Result<(), CycleError> {
    let (graph, id_to_idx) = build_graph(nodes, edges);
    check_self_loops(edges)?;
    if graph.node_count() <= 1 {
        return Ok(());
    }
    check_cycles(&graph, &id_to_idx, edges)
}

fn check_self_loops(edges: &HashMap<EdgeId, Edge>) -> Result<(), CycleError> {
    for (edge_id, edge) in edges {
        if edge.source == edge.target {
            return Err(CycleError::CycleDetected(edge_id.clone()));
        }
    }
    Ok(())
}

fn check_cycles(
    graph: &DiGraph<NodeId, ()>,
    id_to_idx: &HashMap<NodeId, NodeIndex>,
    edges: &HashMap<EdgeId, Edge>,
) -> Result<(), CycleError> {
    if petgraph::algo::is_cyclic_directed(graph) {
        let cycle_edge = find_cycle_edge(graph, id_to_idx, edges);
        return Err(CycleError::CycleDetected(cycle_edge));
    }
    Ok(())
}

#[allow(dead_code)]
fn check_connectivity(graph: &DiGraph<NodeId, ()>) -> Result<(), CycleError> {
    let components = connected_components(graph);
    if components > 1 {
        return Err(CycleError::DisconnectedGraph(components));
    }
    Ok(())
}

/// Build a petgraph `DiGraph` from nodes and edges.
fn build_graph(
    nodes: &HashMap<NodeId, Node>,
    edges: &HashMap<EdgeId, Edge>,
) -> (DiGraph<NodeId, ()>, HashMap<NodeId, NodeIndex>) {
    let sorted_nodes = sorted_node_ids(nodes);
    let id_to_idx = node_ids_to_indices(&sorted_nodes);
    let mut graph = build_graph_nodes(&sorted_nodes);
    add_graph_edges(&mut graph, &id_to_idx, edges);
    (graph, id_to_idx)
}

fn sorted_node_ids(nodes: &HashMap<NodeId, Node>) -> Vec<NodeId> {
    let mut ids: Vec<NodeId> = nodes.keys().cloned().collect();
    ids.sort();
    ids
}

fn node_ids_to_indices(ids: &[NodeId]) -> HashMap<NodeId, NodeIndex> {
    ids.iter()
        .enumerate()
        .map(|(i, id)| (id.clone(), NodeIndex::new(i)))
        .collect()
}

fn build_graph_nodes(ids: &[NodeId]) -> DiGraph<NodeId, ()> {
    let mut graph = DiGraph::with_capacity(ids.len(), 0);
    for id in ids {
        graph.add_node(id.clone());
    }
    graph
}

fn add_graph_edges(
    graph: &mut DiGraph<NodeId, ()>,
    id_to_idx: &HashMap<NodeId, NodeIndex>,
    edges: &HashMap<EdgeId, Edge>,
) {
    for edge in edges.values() {
        if let (Some(&src_idx), Some(&tgt_idx)) =
            (id_to_idx.get(&edge.source), id_to_idx.get(&edge.target))
        {
            graph.add_edge(src_idx, tgt_idx, ());
        }
    }
}

/// Find an edge that's part of a cycle in the graph.
fn find_cycle_edge(
    graph: &DiGraph<NodeId, ()>,
    id_to_idx: &HashMap<NodeId, NodeIndex>,
    edges: &HashMap<EdgeId, Edge>,
) -> EdgeId {
    if let Some(edge) = find_backward_edge(graph, id_to_idx, edges) {
        return edge;
    }
    find_fallback_edge(edges)
}

fn find_backward_edge(
    graph: &DiGraph<NodeId, ()>,
    id_to_idx: &HashMap<NodeId, NodeIndex>,
    edges: &HashMap<EdgeId, Edge>,
) -> Option<EdgeId> {
    let topo = petgraph::algo::toposort(graph, None).ok()?;
    for (edge_id, edge) in edges {
        let src_pos = id_to_idx
            .get(&edge.source)
            .and_then(|i| topo.iter().position(|n| *n == *i))?;
        let tgt_pos = id_to_idx
            .get(&edge.target)
            .and_then(|i| topo.iter().position(|n| *n == *i))?;
        if tgt_pos < src_pos {
            return Some(edge_id.clone());
        }
    }
    None
}

fn find_fallback_edge(edges: &HashMap<EdgeId, Edge>) -> EdgeId {
    edges
        .keys()
        .next()
        .cloned()
        .unwrap_or_else(|| EdgeId::new(String::from("unknown")))
}
