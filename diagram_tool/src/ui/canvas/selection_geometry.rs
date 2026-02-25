#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::models::document::{DiagramDocument, NodeId};

pub(super) fn selected_node_ids(doc: &DiagramDocument) -> Vec<NodeId> {
    doc.editor_state
        .selected_items
        .iter()
        .filter_map(|id| {
            let nid = NodeId::new(id.clone());
            doc.document.nodes.contains_key(&nid).then_some(nid)
        })
        .collect()
}

pub(super) fn selection_bounds(doc: &DiagramDocument) -> Option<(f64, f64, f64, f64)> {
    let ids = selected_node_ids(doc);
    if ids.is_empty() {
        return None;
    }

    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for id in ids {
        if let Some(n) = doc.document.nodes.get(&id) {
            min_x = min_x.min(n.x.0);
            min_y = min_y.min(n.y.0);
            max_x = max_x.max(n.x.0 + n.width.0);
            max_y = max_y.max(n.y.0 + n.height.0);
        }
    }

    Some((min_x, min_y, max_x - min_x, max_y - min_y))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::{selected_node_ids, selection_bounds};
    use crate::models::document::{
        DiagramDocument, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
    };

    #[test]
    fn given_selected_nodes_when_bounds_requested_then_bounds_cover_selection() {
        let mut doc = DiagramDocument::default();
        let id_a = NodeId::new(String::from("a"));
        let id_b = NodeId::new(String::from("b"));
        let _ = doc.document.nodes.insert(
            id_a.clone(),
            Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: String::from("A"),
                x: OrderedFloat(10.0),
                y: OrderedFloat(20.0),
                width: OrderedFloat(50.0),
                height: OrderedFloat(30.0),
                font_size: None,
                font_weight: None,
                locked: true,
                parent: None,
                dag_rank: None,
                tags: Vec::new(),
                metadata: im::HashMap::new(),
                z_index: 0,
                style: Some(NodeStyle::default()),
                collapsed: None,
            },
        );
        let _ = doc.document.nodes.insert(
            id_b.clone(),
            Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: String::from("B"),
                x: OrderedFloat(100.0),
                y: OrderedFloat(120.0),
                width: OrderedFloat(40.0),
                height: OrderedFloat(20.0),
                font_size: None,
                font_weight: None,
                locked: true,
                parent: None,
                dag_rank: None,
                tags: Vec::new(),
                metadata: im::HashMap::new(),
                z_index: 0,
                style: Some(NodeStyle::default()),
                collapsed: None,
            },
        );
        let _ = doc.editor_state.selected_items.insert(id_a.to_string());
        let _ = doc.editor_state.selected_items.insert(id_b.to_string());

        let ids = selected_node_ids(&doc);
        assert_eq!(ids.len(), 2);
        assert_eq!(selection_bounds(&doc), Some((10.0, 20.0, 130.0, 120.0)));
    }
}
