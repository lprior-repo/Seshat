#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]
// usize→f64 precision loss is unavoidable for coordinate maths; allow it explicitly.
#![allow(clippy::cast_precision_loss)]

pub mod apply;
pub mod crossing;
pub mod graph;
pub mod layering;
pub mod positioning;

use crate::layout::grid::calculate_grid_layout;
use diagram_models::document::{DiagramDocument, DocumentData, Node, NodeId};
use im::HashMap;
use petgraph::algo::toposort;
use petgraph::graph::NodeIndex;

pub use positioning::{LEFT_PADDING, NODE_HEIGHT, NODE_WIDTH, TOP_PADDING};

/// Settings for the Sugiyama-style DAG layout algorithm.
#[derive(Clone, Debug)]
pub struct DagLayoutSettings {
    pub layer_spacing: f64,
    pub node_spacing: f64,
}

impl Default for DagLayoutSettings {
    fn default() -> Self {
        Self {
            layer_spacing: 140.0,
            node_spacing: 60.0,
        }
    }
}

/// Pure function: returns a new `DiagramDocument` with node positions updated
/// according to the Sugiyama-style DAG layout.  Falls back to grid layout
/// silently if a cycle is detected.
///
/// Invariants:
/// - Locked nodes are **not** moved.
/// - Child nodes (`parent.is_some()`) move with their parent by the same delta.
/// - The input document is never mutated.
#[must_use]
pub fn dag_layout(doc: &DiagramDocument, settings: &DagLayoutSettings) -> DiagramDocument {
    let (graph, id_to_idx, layout_ids) = graph::build_graph(doc);

    let Ok(topo_order) = toposort(&graph, None) else {
        return calculate_grid_layout(doc, 200.0);
    };

    if layout_ids.is_empty() {
        return doc.clone();
    }

    let layers = layering::assign_layers(&topo_order, &graph);
    let ordered_layers = crossing::barycenter_sweep(layers, &graph);
    let coords = positioning::assign_coordinates(&ordered_layers, settings);

    // NodeIndex → NodeId reverse map
    let idx_to_id: std::collections::HashMap<NodeIndex, NodeId> = id_to_idx
        .iter()
        .map(|(id, &idx)| (idx, id.clone()))
        .collect();

    // NodeId → new (x, y)
    let new_positions: HashMap<NodeId, (f64, f64)> = coords
        .iter()
        .filter_map(|(&idx, &pos)| idx_to_id.get(&idx).map(|id| (id.clone(), pos)))
        .collect();

    // Deltas for child propagation: only for nodes that were moved
    let deltas: HashMap<NodeId, (f64, f64)> = new_positions
        .iter()
        .filter_map(|(id, &(nx, ny))| {
            doc.document
                .nodes
                .get(id)
                .map(|node| (id.clone(), (nx - node.x.0, ny - node.y.0)))
        })
        .collect();

    let next_nodes: HashMap<NodeId, Node> = doc
        .document
        .nodes
        .iter()
        .map(|(id, node)| {
            let next =
                apply::apply_position(id, node, &new_positions, &deltas, &doc.document.nodes);
            (id.clone(), next)
        })
        .collect();

    DiagramDocument {
        version: doc.version,
        revision: doc.revision.increment(),
        document: DocumentData {
            nodes: next_nodes,
            edges: doc.document.edges.clone(),
        },
        editor_state: doc.editor_state.clone(),
    }
}
