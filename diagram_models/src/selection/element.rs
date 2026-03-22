use crate::document::DiagramDocument;
use crate::selection::types::{ElementId, SelectModifiers, SelectionError};

fn is_element_visible(metadata: &im::HashMap<String, serde_json::Value>) -> bool {
    metadata
        .get("visibility")
        .and_then(serde_json::Value::as_str)
        != Some("hidden")
}

/// Selects an element.
///
/// # Errors
///
/// Returns `SelectionError` if element is not found, locked, hidden, or alt-selection fails.
pub fn select_element(
    state: &mut im::HashSet<String>,
    document: &DiagramDocument,
    id: &ElementId,
    modifiers: &SelectModifiers,
) -> Result<(), SelectionError> {
    match id {
        ElementId::Node(node_id) => {
            let node = document
                .document
                .nodes
                .get(node_id)
                .ok_or(SelectionError::ElementNotFound)?;

            if node.lock_state.is_locked() {
                return Err(SelectionError::ElementLocked);
            }

            if !is_element_visible(&node.metadata) {
                return Err(SelectionError::ElementHidden);
            }

            if modifiers.alt {
                if let Some(parent_id) = &node.parent {
                    state.clear();
                    state.insert(parent_id.to_string());
                } else {
                    return Err(SelectionError::NoParentContainer);
                }
            } else {
                state.clear();
                state.insert(node_id.to_string());
            }
        }
        ElementId::Edge(edge_id) => {
            let edge = document
                .document
                .edges
                .get(edge_id)
                .ok_or(SelectionError::ElementNotFound)?;

            if !is_element_visible(&edge.metadata) {
                return Err(SelectionError::ElementHidden);
            }

            state.clear();
            state.insert(edge_id.to_string());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::edge::{ArrowType, Edge, EdgeStyle};
    use crate::document::types::EdgeId;
    use crate::document::types::NodeId;
    use crate::document::{DocumentData, EditorState, LockState, Node, NodeKind, OrderedFloat};
    use im::HashMap;
    use serde_json::json;

    fn setup_doc() -> DiagramDocument {
        let mut nodes = HashMap::new();

        let p1_node = Node {
            kind: NodeKind::Subgraph,
            icon: String::new(),
            label: "p1".to_string(),
            x: OrderedFloat(0.0),
            y: OrderedFloat(0.0),
            width: OrderedFloat(100.0),
            height: OrderedFloat(100.0),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        };

        let c1_node = Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: "c1".to_string(),
            x: OrderedFloat(10.0),
            y: OrderedFloat(10.0),
            width: OrderedFloat(50.0),
            height: OrderedFloat(50.0),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: Some(NodeId::new("p1".to_string())),
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        };

        let mut locked_node = p1_node.clone();
        locked_node.lock_state = LockState::Locked;

        let mut hidden_node = p1_node.clone();
        hidden_node
            .metadata
            .insert("visibility".to_string(), json!("hidden"));

        nodes.insert(NodeId::new("p1".to_string()), p1_node);
        nodes.insert(NodeId::new("c1".to_string()), c1_node);
        nodes.insert(NodeId::new("locked".to_string()), locked_node);
        nodes.insert(NodeId::new("hidden".to_string()), hidden_node);

        let mut edges = HashMap::new();
        edges.insert(
            EdgeId::new("e1".to_string()),
            Edge {
                source: NodeId::new("p1".to_string()),
                target: NodeId::new("c1".to_string()),
                label: String::new(),
                style: EdgeStyle::Solid,
                arrow_type: ArrowType::Default,
                label_offset_t: OrderedFloat(0.5),
                color: None,
                thickness: OrderedFloat(1.0),
                directed: true,
                bend_points: im::Vector::new(),
                tags: im::Vector::new(),
                metadata: HashMap::new(),
                font_size: None,
                source_port: None,
                target_port: None,
            },
        );

        let mut hidden_edge = edges.get(&EdgeId::new("e1".to_string())).unwrap().clone();
        hidden_edge
            .metadata
            .insert("visibility".to_string(), json!("hidden"));
        edges.insert(EdgeId::new("hidden_e".to_string()), hidden_edge);

        DiagramDocument {
            version: 1,
            revision: crate::document::Revision::INITIAL,
            document: DocumentData { nodes, edges },
            editor_state: EditorState::default(),
        }
    }

    #[test]
    fn given_missing_node_when_selected_then_returns_element_not_found_error() {
        let doc = setup_doc();
        let mut state = im::HashSet::new();
        let mods = SelectModifiers {
            alt: false,
            ctrl: false,
            shift: false,
            right_click: false,
        };
        let res = select_element(
            &mut state,
            &doc,
            &ElementId::Node(NodeId::new("missing".to_string())),
            &mods,
        );
        assert_eq!(res, Err(SelectionError::ElementNotFound));
    }

    #[test]
    fn given_locked_node_when_selected_then_returns_element_locked_error() {
        let doc = setup_doc();
        let mut state = im::HashSet::new();
        let mods = SelectModifiers {
            alt: false,
            ctrl: false,
            shift: false,
            right_click: false,
        };
        let res = select_element(
            &mut state,
            &doc,
            &ElementId::Node(NodeId::new("locked".to_string())),
            &mods,
        );
        assert_eq!(res, Err(SelectionError::ElementLocked));
    }

    #[test]
    fn given_hidden_node_when_selected_then_returns_element_hidden_error() {
        let doc = setup_doc();
        let mut state = im::HashSet::new();
        let mods = SelectModifiers {
            alt: false,
            ctrl: false,
            shift: false,
            right_click: false,
        };
        let res = select_element(
            &mut state,
            &doc,
            &ElementId::Node(NodeId::new("hidden".to_string())),
            &mods,
        );
        assert_eq!(res, Err(SelectionError::ElementHidden));
    }

    #[test]
    fn given_valid_node_when_selected_then_adds_to_state() {
        let doc = setup_doc();
        let mut state = im::HashSet::new();
        let mods = SelectModifiers {
            alt: false,
            ctrl: false,
            shift: false,
            right_click: false,
        };
        let res = select_element(
            &mut state,
            &doc,
            &ElementId::Node(NodeId::new("c1".to_string())),
            &mods,
        );
        assert!(res.is_ok());
        assert!(state.contains("c1"));
    }

    #[test]
    fn given_valid_node_when_alt_selected_then_selects_parent() {
        let doc = setup_doc();
        let mut state = im::HashSet::new();
        let mods = SelectModifiers {
            alt: true,
            ctrl: false,
            shift: false,
            right_click: false,
        };
        let res = select_element(
            &mut state,
            &doc,
            &ElementId::Node(NodeId::new("c1".to_string())),
            &mods,
        );
        assert!(res.is_ok());
        assert!(state.contains("p1"));
    }

    #[test]
    fn given_node_without_parent_when_alt_selected_then_returns_no_parent_error() {
        let doc = setup_doc();
        let mut state = im::HashSet::new();
        let mods = SelectModifiers {
            alt: true,
            ctrl: false,
            shift: false,
            right_click: false,
        };
        let res = select_element(
            &mut state,
            &doc,
            &ElementId::Node(NodeId::new("p1".to_string())),
            &mods,
        );
        assert_eq!(res, Err(SelectionError::NoParentContainer));
    }

    #[test]
    fn given_missing_edge_when_selected_then_returns_element_not_found_error() {
        let doc = setup_doc();
        let mut state = im::HashSet::new();
        let mods = SelectModifiers {
            alt: false,
            ctrl: false,
            shift: false,
            right_click: false,
        };
        let res = select_element(
            &mut state,
            &doc,
            &ElementId::Edge(EdgeId::new("missing".to_string())),
            &mods,
        );
        assert_eq!(res, Err(SelectionError::ElementNotFound));
    }

    #[test]
    fn given_hidden_edge_when_selected_then_returns_element_hidden_error() {
        let doc = setup_doc();
        let mut state = im::HashSet::new();
        let mods = SelectModifiers {
            alt: false,
            ctrl: false,
            shift: false,
            right_click: false,
        };
        let res = select_element(
            &mut state,
            &doc,
            &ElementId::Edge(EdgeId::new("hidden_e".to_string())),
            &mods,
        );
        assert_eq!(res, Err(SelectionError::ElementHidden));
    }

    #[test]
    fn given_valid_edge_when_selected_then_adds_to_state() {
        let doc = setup_doc();
        let mut state = im::HashSet::new();
        let mods = SelectModifiers {
            alt: false,
            ctrl: false,
            shift: false,
            right_click: false,
        };
        let res = select_element(
            &mut state,
            &doc,
            &ElementId::Edge(EdgeId::new("e1".to_string())),
            &mods,
        );
        assert!(res.is_ok());
        assert!(state.contains("e1"));
    }
}
