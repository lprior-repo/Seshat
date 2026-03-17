use crate::layout::dag::DagLayoutSettings;
use petgraph::graph::NodeIndex;

pub const NODE_WIDTH: f64 = 220.0;
pub const NODE_HEIGHT: f64 = 68.0;
pub const LEFT_PADDING: f64 = 120.0;
pub const TOP_PADDING: f64 = 80.0;

/// Assign (x, y) coordinates from the layered, ordered structure.
pub(crate) fn assign_coordinates(
    layers: &[Vec<NodeIndex>],
    settings: &DagLayoutSettings,
) -> std::collections::HashMap<NodeIndex, (f64, f64)> {
    let max_layer_size = layers.iter().map(Vec::len).max().unwrap_or(1);
    let canvas_height = (max_layer_size as f64).mul_add(
        NODE_HEIGHT,
        max_layer_size.saturating_sub(1) as f64 * settings.node_spacing,
    );

    layers
        .iter()
        .enumerate()
        .flat_map(|(layer_idx, nodes)| {
            let x = (layer_idx as f64).mul_add(NODE_WIDTH + settings.layer_spacing, LEFT_PADDING);
            let layer_total_height = (nodes.len() as f64).mul_add(
                NODE_HEIGHT,
                nodes.len().saturating_sub(1) as f64 * settings.node_spacing,
            );
            let y_offset = TOP_PADDING + (canvas_height - layer_total_height) / 2.0;

            nodes
                .iter()
                .enumerate()
                .map(move |(pos, &idx)| {
                    let y = (pos as f64).mul_add(NODE_HEIGHT + settings.node_spacing, y_offset);
                    (idx, (x, y))
                })
                .collect::<Vec<_>>()
        })
        .collect()
}
