#![allow(clippy::unwrap_used)]

use super::dsl::GraphBuilder;
use crate::layout::dag::{dag_layout, DagLayoutSettings};
use crate::layout::grid::calculate_grid_layout;

// -----------------------------------------------------------------------------
// Executive Summaries (Dave Farley ATDD style)
// -----------------------------------------------------------------------------
// We express intent through the DSL. The layout process shouldn't mutate 
// the original graph but return a completely new projected state.

#[test]
fn given_a_simple_tree_when_layout_is_computed_then_ranks_are_assigned_topologically() {
    // Given
    let mut builder = GraphBuilder::new();
    //   0
    //  / \
    // 1   2
    builder.connect(0, 1).connect(0, 2);
    let doc = builder.build();
    let settings = DagLayoutSettings::default();

    // When
    let laid_out = dag_layout(&doc, &settings);

    // Then
    let node0 = laid_out.document.nodes.get(&builder.get_node_id(0)).unwrap();
    let node1 = laid_out.document.nodes.get(&builder.get_node_id(1)).unwrap();
    let node2 = laid_out.document.nodes.get(&builder.get_node_id(2)).unwrap();

    // Node 0 should be at a lower X (left) than nodes 1 and 2
    assert!(node0.x.0 < node1.x.0);
    assert!(node0.x.0 < node2.x.0);

    // Nodes 1 and 2 should be on the same layer (same X)
    assert_eq!(node1.x.0, node2.x.0);
    
    // Nodes 1 and 2 should have different Y coordinates
    assert!(node1.y.0 != node2.y.0);
}

#[test]
fn given_a_diamond_graph_when_layout_is_computed_then_paths_are_balanced() {
    // Given
    let mut builder = GraphBuilder::new();
    //   0
    //  / \
    // 1   2
    //  \ /
    //   3
    builder
        .connect(0, 1)
        .connect(0, 2)
        .connect(1, 3)
        .connect(2, 3);
        
    let doc = builder.build();
    let settings = DagLayoutSettings::default();

    // When
    let laid_out = dag_layout(&doc, &settings);

    // Then
    let node0 = laid_out.document.nodes.get(&builder.get_node_id(0)).unwrap();
    let node1 = laid_out.document.nodes.get(&builder.get_node_id(1)).unwrap();
    let node2 = laid_out.document.nodes.get(&builder.get_node_id(2)).unwrap();
    let node3 = laid_out.document.nodes.get(&builder.get_node_id(3)).unwrap();

    // Verify topological ordering (left-to-right layering)
    assert!(node0.x.0 < node1.x.0);
    assert!(node0.x.0 < node2.x.0);
    
    assert!(node1.x.0 < node3.x.0);
    assert!(node2.x.0 < node3.x.0);
    
    // Nodes 1 and 2 are on the same layer
    assert_eq!(node1.x.0, node2.x.0);
}

#[test]
fn given_an_empty_graph_when_layout_is_computed_then_an_empty_layout_is_returned() {
    // Given
    let builder = GraphBuilder::new();
    let doc = builder.build();
    let settings = DagLayoutSettings::default();

    // When
    let laid_out = dag_layout(&doc, &settings);

    // Then
    assert!(laid_out.document.nodes.is_empty());
}

#[test]
fn given_a_graph_with_cycles_when_layout_is_computed_then_it_falls_back_to_grid_layout() {
    // Given
    let mut builder = GraphBuilder::new();
    // 0 -> 1 -> 2 -> 0 (cycle)
    builder.connect(0, 1).connect(1, 2).connect(2, 0);
    let doc = builder.build();
    let settings = DagLayoutSettings::default();

    // When
    // DAG layout falls back to grid layout if toposort fails (which happens on cycles)
    let laid_out = dag_layout(&doc, &settings);
    let grid_layout = calculate_grid_layout(&doc, 200.0);

    // Then
    let node0_dag = laid_out.document.nodes.get(&builder.get_node_id(0)).unwrap();
    let node0_grid = grid_layout.document.nodes.get(&builder.get_node_id(0)).unwrap();
    
    // The positions should exactly match what the grid layout would produce
    assert_eq!(node0_dag.x.0, node0_grid.x.0);
    assert_eq!(node0_dag.y.0, node0_grid.y.0);
}

#[test]
fn given_unconnected_components_when_layout_is_computed_then_all_are_positioned() {
    // Given
    let mut builder = GraphBuilder::new();
    // 0 -> 1
    // 2 -> 3
    builder.connect(0, 1);
    builder.connect(2, 3);
    let doc = builder.build();
    let settings = DagLayoutSettings::default();

    // When
    let laid_out = dag_layout(&doc, &settings);

    // Then
    let node0 = laid_out.document.nodes.get(&builder.get_node_id(0)).unwrap();
    let node1 = laid_out.document.nodes.get(&builder.get_node_id(1)).unwrap();
    let node2 = laid_out.document.nodes.get(&builder.get_node_id(2)).unwrap();
    let node3 = laid_out.document.nodes.get(&builder.get_node_id(3)).unwrap();

    // 0 and 2 should be in the first layer
    assert_eq!(node0.x.0, node2.x.0);
    
    // 1 and 3 should be in the second layer
    assert_eq!(node1.x.0, node3.x.0);
    
    // Layers progress left to right
    assert!(node0.x.0 < node1.x.0);
}
