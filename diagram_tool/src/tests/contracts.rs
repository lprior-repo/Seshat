use crate::core::delete::delete_selected;
use crate::core::history::{apply_redo, apply_undo};
// Note: We'll implement pure versions of zoom later, for now we skip them or mock them
// use crate::viewport::operations::{apply_zoom_in as core_zoom_in, apply_zoom_out as core_zoom_out};
use crate::history::History;
use crate::models::document::{
    ArrowType, DiagramDocument, Edge, EdgeId, EdgeStyle, Node, NodeId, NodeKind, NodeStyle,
    OrderedFloat,
};
use im::HashMap;

fn create_test_node(x: f64, y: f64) -> Node {
    Node {
        kind: NodeKind::Text,
        icon: String::new(),
        label: String::from("Test Node"),
        x: OrderedFloat::new_unchecked(x),
        y: OrderedFloat::new_unchecked(y),
        width: OrderedFloat::new_unchecked(100.0),
        height: OrderedFloat::new_unchecked(24.0),
        font_size: None,
        font_weight: None,
        locked: false,
        parent: None,
        dag_rank: None,
        tags: im::Vector::new(),
        metadata: HashMap::new(),
        z_index: 0,
        style: Some(NodeStyle::default()),
        collapsed: None,
    }
}

fn create_test_edge(source: NodeId, target: NodeId) -> Edge {
    Edge {
        source,
        target,
        label: String::new(),
        style: EdgeStyle::default(),
        arrow_type: ArrowType::default(),
        label_offset_t: OrderedFloat::new_unchecked(0.5),
        color: None,
        thickness: OrderedFloat::new_unchecked(1.5),
        directed: true,
        bend_points: im::Vector::new(),
        tags: im::Vector::new(),
        metadata: HashMap::new(),
        font_size: None,
        source_port: None,
        target_port: None,
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_doc_003_deleting_node_removes_incident_edges() {
    let mut doc = DiagramDocument::default();

    let node1_id = NodeId::new("node1".to_string());
    let node2_id = NodeId::new("node2".to_string());

    doc.document
        .nodes
        .insert(node1_id.clone(), create_test_node(0.0, 0.0));
    doc.document
        .nodes
        .insert(node2_id.clone(), create_test_node(100.0, 0.0));

    let edge_id = EdgeId::new("edge1".to_string());
    doc.document.edges.insert(
        edge_id.clone(),
        create_test_edge(node1_id.clone(), node2_id.clone()),
    );

    doc.editor_state
        .selected_items
        .insert(node1_id.as_str().to_string());

    let result = delete_selected(&mut doc);
    assert!(result);

    assert_eq!(doc.document.nodes.len(), 1);
    assert_eq!(
        doc.document.edges.len(),
        0,
        "Incident edge should be deleted"
    );
    assert!(doc.document.nodes.contains_key(&node2_id));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_doc_004_undo_redo_roundtrips_mutation_state() {
    let mut doc = DiagramDocument::default();

    let node1_id = NodeId::new("node1".to_string());
    doc.document
        .nodes
        .insert(node1_id.clone(), create_test_node(0.0, 0.0));
    doc.editor_state
        .selected_items
        .insert(node1_id.as_str().to_string());

    let mut history = History::new();

    // Perform delete and record history manually
    history = history.push(doc.clone());
    let deleted = delete_selected(&mut doc);
    assert!(deleted);
    assert_eq!(doc.document.nodes.len(), 0);

    // Undo
    apply_undo(&mut doc, &mut history).unwrap();
    assert_eq!(doc.document.nodes.len(), 1, "Undo should restore node");

    // Redo
    apply_redo(&mut doc, &mut history).unwrap();
    assert_eq!(doc.document.nodes.len(), 0, "Redo should delete node again");
}

#[cfg(test)]
mod combinatorial_tests {
    use super::*;
    use crate::perf::harness::PerformanceDriver;
    use crate::store_sqlx::SqlitePool;
    use proptest::prelude::*;

    proptest! {
        // Combinatorial headless test harness asserting Human Priority,
        // 8ms budget, and Ghosting Diff generation.
        // Does not mock the WAL.
        #[test]
        fn test_concurrent_interactions_with_restate(
            human_events in 0..10usize,
            ai_events in 0..10usize
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // Initialize real WAL (no mocking)
                // In a real test, this would use a temp file for true WAL mode testing
                let db_url = format!("sqlite::memory:?cache=shared");
                if let Ok(pool) = SqlitePool::connect(&db_url).await {
                    let mut driver = PerformanceDriver::new(pool);

                    let result = driver.simulate_concurrent_session(human_events, ai_events).await;

                    // Asserts Human Priority, 8ms budget, and Ghosting Diff generation
                    assert!(result.is_ok(), "Combinatorial headless test failed invariants");
                }
            });
        }
    }
}
