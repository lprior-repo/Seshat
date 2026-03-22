//! Canvas state module
//!
//! Splits the canvas state into:
//! - `editor_fsm`: Finite state machine for editor interactions
//! - `canvas_state`: Dioxus signal-based canvas state management
//! - `tests`: Property-based and unit tests for the state machine

pub mod canvas_state;
pub mod editor_fsm;

pub use canvas_state::{apply_transition, use_canvas_state, CanvasState};
pub use editor_fsm::{calculate_transition, EditorError, EditorEvent, EditorState};

#[cfg(test)]
mod tests {
    use super::editor_fsm::{calculate_transition, EditorError, EditorEvent, EditorState};
    use diagram_models::document::{
        DiagramDocument, DocumentData, Edge, EdgeId, EdgeStyle, Node, NodeId, NodeKind,
        OrderedFloat, Revision,
    };
    use im::HashMap;
    use proptest::prelude::*;

    #[derive(Debug)]
    struct FsmDriver {
        doc: DiagramDocument,
        state: EditorState,
    }

    impl FsmDriver {
        fn new() -> Self {
            let mut nodes = HashMap::new();
            let mut edges = HashMap::new();
            nodes.insert(
                NodeId::new("node_a".to_string()),
                Node {
                    kind: NodeKind::Node,
                    icon: String::new(),
                    label: "A".to_string(),
                    x: OrderedFloat(0.0),
                    y: OrderedFloat(0.0),
                    width: OrderedFloat(100.0),
                    height: OrderedFloat(100.0),
                    font_size: None,
                    font_weight: None,
                    lock_state: diagram_models::document::LockState::Unlocked,
                    parent: None,
                    dag_rank: None,
                    tags: im::Vector::new(),
                    metadata: im::HashMap::new(),
                    z_index: 0,
                    style: None,
                    collapsed: None,
                },
            );
            nodes.insert(
                NodeId::new("node_b".to_string()),
                Node {
                    kind: NodeKind::Node,
                    icon: String::new(),
                    label: "B".to_string(),
                    x: OrderedFloat(0.0),
                    y: OrderedFloat(0.0),
                    width: OrderedFloat(100.0),
                    height: OrderedFloat(100.0),
                    font_size: None,
                    font_weight: None,
                    lock_state: diagram_models::document::LockState::Unlocked,
                    parent: None,
                    dag_rank: None,
                    tags: im::Vector::new(),
                    metadata: im::HashMap::new(),
                    z_index: 0,
                    style: None,
                    collapsed: None,
                },
            );
            edges.insert(
                EdgeId::new("edge_1".to_string()),
                Edge {
                    source: NodeId::new("node_a".to_string()),
                    target: NodeId::new("node_b".to_string()),
                    label: String::new(),
                    style: EdgeStyle::Solid,
                    arrow_type: diagram_models::document::ArrowType::Default,
                    label_offset_t: OrderedFloat(0.5),
                    color: None,
                    thickness: OrderedFloat(1.0),
                    directed: true,
                    bend_points: im::Vector::new(),
                    tags: im::Vector::new(),
                    metadata: im::HashMap::new(),
                    font_size: None,
                    source_port: None,
                    target_port: None,
                },
            );
            Self {
                doc: DiagramDocument {
                    version: 1,
                    revision: Revision::INITIAL,
                    document: DocumentData { nodes, edges },
                    editor_state: diagram_models::document::EditorState::default(),
                },
                state: EditorState::Idle,
            }
        }

        fn given_idle_canvas(mut self) -> Self {
            self.state = EditorState::Idle;
            self
        }

        fn given_hovering_node(mut self, id: &str) -> Self {
            self.state = EditorState::HoveringNode(NodeId::new(id.to_string()));
            self
        }

        fn given_editing_node(mut self, id: &str) -> Self {
            self.state = EditorState::EditingNode(NodeId::new(id.to_string()));
            self
        }

        fn apply(&mut self, event: EditorEvent) -> Result<&mut Self, EditorError> {
            self.state = calculate_transition(&self.state, event, &self.doc)?;
            Ok(self)
        }

        fn when_mouse_enters_node(&mut self, id: &str) -> Result<&mut Self, EditorError> {
            self.apply(EditorEvent::HoverNode(NodeId::new(id.to_string())))
        }

        fn when_mouse_clicks_node(&mut self, id: &str) -> Result<&mut Self, EditorError> {
            self.apply(EditorEvent::EditNode(NodeId::new(id.to_string())))
        }

        fn when_escape_pressed(&mut self) -> Result<&mut Self, EditorError> {
            self.apply(EditorEvent::Escape)
        }

        fn then_state_is_hovering(&mut self, id: &str) -> &mut Self {
            assert_eq!(
                self.state,
                EditorState::HoveringNode(NodeId::new(id.to_string()))
            );
            self
        }

        fn then_state_is_editing(&mut self, id: &str) -> &mut Self {
            assert_eq!(
                self.state,
                EditorState::EditingNode(NodeId::new(id.to_string()))
            );
            self
        }

        fn then_state_is_idle(&mut self) -> &mut Self {
            assert_eq!(self.state, EditorState::Idle);
            self
        }
    }

    #[test]
    fn test_transitions_to_hovering_when_mouse_enters_node() {
        let mut driver = FsmDriver::new().given_idle_canvas();
        driver
            .when_mouse_enters_node("node_a")
            .unwrap()
            .then_state_is_hovering("node_a");
    }

    #[test]
    fn test_transitions_to_editing_when_node_clicked() {
        let mut driver = FsmDriver::new().given_hovering_node("node_a");
        driver
            .when_mouse_clicks_node("node_a")
            .unwrap()
            .then_state_is_editing("node_a");
    }

    #[test]
    fn test_commits_edit_and_returns_to_idle_on_escape() {
        let mut driver = FsmDriver::new().given_editing_node("node_a");
        driver.when_escape_pressed().unwrap().then_state_is_idle();
    }

    #[test]
    fn test_p2_violation_returns_invalid_transition() {
        let mut driver = FsmDriver::new().given_editing_node("node_a");
        let result = driver.apply(EditorEvent::EditEdge(EdgeId::new("edge_1".to_string())));
        assert!(matches!(result, Err(EditorError::InvalidTransition { .. })));
    }

    #[test]
    fn test_p1_violation_returns_element_not_found() {
        let mut driver = FsmDriver::new().given_idle_canvas();
        let result = driver.when_mouse_enters_node("non_existent_node");
        assert_eq!(
            result.unwrap_err(),
            EditorError::ElementNotFound("non_existent_node".to_string())
        );
    }

    proptest! {
        #[test]
        fn proptest_arbitrary_event_sequences_never_panic(events in prop::collection::vec(any_event(), 0..100)) {
            let mut driver = FsmDriver::new();
            for event in events {
                let _ = driver.apply(event);
            }
        }
    }

    fn any_event() -> impl Strategy<Value = EditorEvent> {
        prop_oneof![
            Just(EditorEvent::Escape),
            Just(EditorEvent::ClearHover),
            Just(EditorEvent::HoverNode(NodeId::new("node_a".to_string()))),
            Just(EditorEvent::HoverNode(NodeId::new("invalid".to_string()))),
            Just(EditorEvent::EditNode(NodeId::new("node_a".to_string()))),
            Just(EditorEvent::EditNode(NodeId::new("node_b".to_string()))),
            Just(EditorEvent::HoverEdge(EdgeId::new("edge_1".to_string()))),
            Just(EditorEvent::EditEdge(EdgeId::new("edge_1".to_string()))),
        ]
    }
}
