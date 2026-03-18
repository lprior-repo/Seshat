//! Test scene generation utilities for benchmarking.

use diagram_models::document::DiagramDocument;

/// Generates a test scene with the specified number of nodes.
#[must_use]
pub fn generate_test_scene(node_count: u32, seed: u64) -> DiagramDocument {
    use im::HashMap as ImHashMap;

    use diagram_models::document::{
        DocumentData, Edge, EdgeId, LockState, Node, NodeId, NodeKind, OrderedFloat,
    };

    let mut nodes = ImHashMap::new();
    let mut edges = ImHashMap::new();

    // Simple LCG for deterministic generation
    let mut rng = seed;
    let next_random = |r: &mut u64| -> f64 {
        *r = r.wrapping_mul(1_103_515_245).wrapping_add(12345);
        f64::from(u16::try_from((*r >> 16) & 0xFFFF).unwrap_or(0)) / 65535.0
    };

    // Generate nodes in a grid pattern
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let grid_size = f64::from(node_count).sqrt().ceil() as u32;
    for i in 0..node_count {
        let row = i / grid_size;
        let col = i % grid_size;

        let x = f64::from(col).mul_add(120.0, next_random(&mut rng) * 20.0);
        let y = f64::from(row).mul_add(80.0, next_random(&mut rng) * 20.0);

        let node = Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: format!("Node {i}"),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(100.0),
            height: OrderedFloat(50.0),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::vector![],
            metadata: ImHashMap::new(),
            z_index: i64::from(i),
            style: None,
            collapsed: None,
        };

        nodes.insert(NodeId::new(format!("node-{i}")), node);
    }

    // Generate some edges (about 50% of nodes have edges)
    for i in 0..(node_count / 2) {
        let source_idx = i;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let target_idx = (i + 1 + (next_random(&mut rng) * 10.0) as u32) % node_count;

        if source_idx != target_idx {
            let edge = Edge {
                source: NodeId::new(format!("node-{source_idx}")),
                target: NodeId::new(format!("node-{target_idx}")),
                label: String::new(),
                style: diagram_models::document::EdgeStyle::default(),
                arrow_type: diagram_models::document::ArrowType::default(),
                label_offset_t: OrderedFloat(0.5),
                color: None,
                thickness: OrderedFloat(1.5),
                directed: true,
                bend_points: im::vector![],
                tags: im::vector![],
                metadata: ImHashMap::new(),
                font_size: None,
                source_port: None,
                target_port: None,
            };

            edges.insert(EdgeId::new(format!("edge-{i}")), edge);
        }
    }

    DiagramDocument {
        version: 2,
        revision: diagram_models::document::Revision::INITIAL,
        document: DocumentData { nodes, edges },
        editor_state: diagram_models::document::EditorState::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_generate_test_scene() {
        let doc = generate_test_scene(100, 42);

        assert_eq!(doc.document.nodes.len(), 100);
        // Should have about 50 edges
        assert!(doc.document.edges.len() > 30 && doc.document.edges.len() < 60);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_generate_test_scene_deterministic() {
        let doc1 = generate_test_scene(100, 42);
        let doc2 = generate_test_scene(100, 42);

        assert_eq!(doc1.document.nodes.len(), doc2.document.nodes.len());
    }
}
