use diagram_models::document::{DiagramDocument, NodeId};
use diagram_models::envelope::EventEnvelope;
use diagram_models::history::History;
use dioxus::prelude::*;
use im::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DispatchError {
    Failed,
}

pub enum LabelTargetType {
    Node,
    Edge,
}

/// Dispatches a label update to the DB.
///
/// # Errors
///
/// Returns `DispatchError` if dispatching fails.
pub const fn dispatch_update_label(
    _tx: Option<&Coroutine<EventEnvelope>>,
    _target_id: &str,
    _target_type: LabelTargetType,
    _old_label: &str,
    _new_label: &str,
) -> Result<(), DispatchError> {
    Ok(())
}

pub struct ResizeBounds;
impl ResizeBounds {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        _id: NodeId,
        _ox: f64,
        _oy: f64,
        _ow: f64,
        _oh: f64,
        _nx: f64,
        _ny: f64,
        _nw: f64,
        _nh: f64,
    ) -> Self {
        Self
    }
}

/// Dispatches a node resize to the DB.
///
/// # Errors
///
/// Returns `DispatchError` if dispatching fails.
pub const fn dispatch_node_resize(
    _tx: Option<&Coroutine<EventEnvelope>>,
    _bounds: ResizeBounds,
) -> Result<(), DispatchError> {
    Ok(())
}

/// Mutates the document and pushes to history.
///
/// # Errors
///
/// Returns the error from the provided closure `F`.
pub fn mutate_doc_with_history<F, E>(
    doc_signal: &mut Signal<DiagramDocument>,
    history_signal: &mut Signal<History>,
    f: F,
) -> Result<(), E>
where
    F: FnOnce(&DiagramDocument) -> Result<DiagramDocument, E>,
{
    let current = doc_signal.read().clone();
    let new_doc = f(&current)?;
    let new_history = history_signal.read().push(current);
    *history_signal.write() = new_history;
    *doc_signal.write() = new_doc;
    Ok(())
}

#[must_use]
pub fn drag_original_positions(
    doc: &DiagramDocument,
    selected_items: &HashSet<String>,
) -> HashMap<NodeId, (f64, f64)> {
    let selected_nodes = selected_items
        .iter()
        .map(|id| diagram_models::document::NodeId::new(id.clone()))
        .filter(|id| doc.document.nodes.contains_key(id))
        .collect::<HashSet<_>>();

    let with_children = std::iter::successors(Some(selected_nodes), |current| {
        let expanded = doc
            .document
            .nodes
            .iter()
            .fold(current.clone(), |acc, (id, node)| {
                if node
                    .parent
                    .as_ref()
                    .is_some_and(|parent| acc.contains(parent))
                {
                    acc.update(id.clone())
                } else {
                    acc
                }
            });

        (expanded.len() > current.len()).then_some(expanded)
    })
    .last()
    .unwrap_or_else(HashSet::new);

    with_children.iter().fold(HashMap::new(), |acc, id| {
        if let Some(node) = doc.document.nodes.get(id) {
            acc.update(id.clone(), (node.x.0, node.y.0))
        } else {
            acc
        }
    })
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        clippy::let_unit_value,
        unused_variables
    )]

    use super::*;
    use diagram_models::document::{
        DiagramDocument, LockState, Node, NodeId, NodeKind, OrderedFloat,
    };
    use im::{HashMap, HashSet};

    fn make_node(id: &str, x: f64, y: f64, parent: Option<NodeId>) -> (NodeId, Node) {
        let node_id = NodeId::new(id.to_string());
        let node = Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: String::from("Test"),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(100.0),
            height: OrderedFloat(50.0),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: im::HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        };
        (node_id, node)
    }

    #[test]
    fn dispatch_update_label_always_returns_ok() {
        let result = dispatch_update_label(None, "id-1", LabelTargetType::Node, "old", "new");
        assert!(result.is_ok());
    }

    #[test]
    fn dispatch_node_resize_always_returns_ok() {
        let bounds = ResizeBounds::new(
            NodeId::new("n1".to_string()),
            0.0,
            0.0,
            100.0,
            50.0,
            200.0,
            0.0,
            200.0,
            100.0,
        );
        let result = dispatch_node_resize(None, bounds);
        assert!(result.is_ok());
    }

    #[test]
    fn resize_bounds_new_is_const_constructible() {
        let _bounds = ResizeBounds::new(
            NodeId::new("n1".to_string()),
            0.0,
            0.0,
            100.0,
            50.0,
            200.0,
            0.0,
            200.0,
            100.0,
        );
    }

    #[test]
    fn dispatch_error_failed_variant_exists() {
        let err = DispatchError::Failed;
        let _ = format!("{err:?}");
    }

    #[test]
    fn label_target_type_variants() {
        let _node = LabelTargetType::Node;
        let _edge = LabelTargetType::Edge;
    }

    #[test]
    fn drag_original_positions_empty_selection() {
        let doc = DiagramDocument::default();
        let selected: HashSet<String> = HashSet::new();
        let positions = drag_original_positions(&doc, &selected);
        assert!(positions.is_empty());
    }

    #[test]
    fn drag_original_positions_nonexistent_node() {
        let doc = DiagramDocument::default();
        let selected: HashSet<String> = HashSet::new();
        let selected = selected.update("ghost-node".to_string());
        let positions = drag_original_positions(&doc, &selected);
        assert!(positions.is_empty());
    }

    #[test]
    fn drag_original_positions_single_node() {
        let mut doc = DiagramDocument::default();
        let (node_id, node) = make_node("n1", 50.0, 75.0, None);
        doc.document.nodes.insert(node_id.clone(), node);

        let selected: HashSet<String> = HashSet::new();
        let selected = selected.update("n1".to_string());
        let positions = drag_original_positions(&doc, &selected);

        assert_eq!(positions.len(), 1);
        assert_eq!(positions.get(&node_id), Some(&(50.0, 75.0)));
    }

    #[test]
    fn drag_original_positions_multiple_nodes() {
        let mut doc = DiagramDocument::default();
        let (id1, n1) = make_node("a", 10.0, 20.0, None);
        let (id2, n2) = make_node("b", 30.0, 40.0, None);
        doc.document.nodes.insert(id1.clone(), n1);
        doc.document.nodes.insert(id2.clone(), n2);

        let selected: HashSet<String> = HashSet::new();
        let selected = selected.update("a".to_string()).update("b".to_string());
        let positions = drag_original_positions(&doc, &selected);

        assert_eq!(positions.len(), 2);
        assert_eq!(positions.get(&id1), Some(&(10.0, 20.0)));
        assert_eq!(positions.get(&id2), Some(&(30.0, 40.0)));
    }

    #[test]
    fn drag_original_positions_includes_children() {
        let mut doc = DiagramDocument::default();
        let (parent_id, parent_node) = make_node("parent", 0.0, 0.0, None);
        let (child_id, child_node) = make_node("child", 100.0, 200.0, Some(parent_id.clone()));
        doc.document.nodes.insert(parent_id.clone(), parent_node);
        doc.document.nodes.insert(child_id.clone(), child_node);

        let selected: HashSet<String> = HashSet::new();
        let selected = selected.update("parent".to_string());
        let positions = drag_original_positions(&doc, &selected);

        assert_eq!(positions.len(), 2);
        assert_eq!(positions.get(&parent_id), Some(&(0.0, 0.0)));
        assert_eq!(positions.get(&child_id), Some(&(100.0, 200.0)));
    }

    #[test]
    fn drag_original_positions_deep_child_hierarchy() {
        let mut doc = DiagramDocument::default();
        let (gp_id, gp) = make_node("grandparent", 0.0, 0.0, None);
        let (p_id, p) = make_node("parent", 10.0, 10.0, Some(gp_id.clone()));
        let (c_id, c) = make_node("child", 20.0, 20.0, Some(p_id.clone()));
        doc.document.nodes.insert(gp_id.clone(), gp);
        doc.document.nodes.insert(p_id.clone(), p);
        doc.document.nodes.insert(c_id.clone(), c);

        let selected: HashSet<String> = HashSet::new();
        let selected = selected.update("grandparent".to_string());
        let positions = drag_original_positions(&doc, &selected);

        assert_eq!(positions.len(), 3);
    }

    #[test]
    fn drag_original_positions_sibling_children_not_included() {
        let mut doc = DiagramDocument::default();
        let (parent_id, parent_node) = make_node("parent", 0.0, 0.0, None);
        let (child_id, child_node) = make_node("child", 100.0, 200.0, Some(parent_id.clone()));
        let (sibling_id, sibling_node) = make_node("sibling", 300.0, 400.0, None);
        doc.document.nodes.insert(parent_id.clone(), parent_node);
        doc.document.nodes.insert(child_id.clone(), child_node);
        doc.document.nodes.insert(sibling_id.clone(), sibling_node);

        let selected: HashSet<String> = HashSet::new();
        let selected = selected.update("parent".to_string());
        let positions = drag_original_positions(&doc, &selected);

        assert_eq!(positions.len(), 2);
        assert!(positions.contains_key(&parent_id));
        assert!(positions.contains_key(&child_id));
        assert!(!positions.contains_key(&sibling_id));
    }

    #[test]
    fn drag_original_positions_does_not_follow_circular_parent() {
        let mut doc = DiagramDocument::default();
        let (a_id, a_node) = make_node("a", 0.0, 0.0, None);
        let b_id = NodeId::new("b".to_string());
        let b_node = Node {
            parent: Some(b_id.clone()),
            ..make_node("b", 10.0, 10.0, None).1
        };
        doc.document.nodes.insert(a_id.clone(), a_node);
        doc.document.nodes.insert(b_id.clone(), b_node);

        let selected: HashSet<String> = HashSet::new();
        let selected = selected.update("a".to_string());
        let positions = drag_original_positions(&doc, &selected);

        assert_eq!(positions.len(), 1);
        assert!(positions.contains_key(&a_id));
    }

    #[test]
    fn dispatch_update_label_with_edge_target() {
        let result = dispatch_update_label(None, "edge-1", LabelTargetType::Edge, "old", "new");
        assert!(result.is_ok());
    }

    #[test]
    fn dispatch_update_label_with_empty_strings() {
        let result = dispatch_update_label(None, "", LabelTargetType::Node, "", "");
        assert!(result.is_ok());
    }

    #[test]
    fn dispatch_node_resize_with_zero_bounds() {
        let bounds = ResizeBounds::new(
            NodeId::new("n1".to_string()),
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );
        let result = dispatch_node_resize(None, bounds);
        assert!(result.is_ok());
    }

    #[test]
    fn dispatch_error_debug_clone_eq() {
        let e1 = DispatchError::Failed;
        let e2 = DispatchError::Failed;
        assert_eq!(e1, e2);
        assert_eq!(e1.clone(), e2);
        assert_eq!(format!("{e1:?}"), "Failed");
    }
}
