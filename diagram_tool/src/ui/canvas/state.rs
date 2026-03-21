use super::root_handlers::{
    use_keyboard_handler, use_middle_pan_handler, use_raf_handler, use_resize_handler,
    use_touch_handler,
};
use crate::app::DraggedIconPayload;
use crate::history::History;
use crate::ui::canvas::document_ops::{ordered_node_ids, WheelSample};
use crate::ui::editor::ToolMode;
use crate::ui::toast::use_toast;
use canvas_domain::interaction_reducer::InteractionMode;
use diagram_models::document::{ArrowType, DiagramDocument, EdgeId, EdgeStyle, NodeId};
use dioxus::prelude::*;
use std::collections::HashSet;

#[derive(Clone, PartialEq, Debug)]
pub enum EditorState {
    Idle,
    HoveringNode(NodeId),
    EditingNode(NodeId),
    HoveringEdge(EdgeId),
    EditingEdge(EdgeId),
}

#[derive(Clone, PartialEq, Debug)]
pub enum EditorEvent {
    HoverNode(NodeId),
    HoverEdge(EdgeId),
    EditNode(NodeId),
    EditEdge(EdgeId),
    ClearHover,
    Escape,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum EditorError {
    #[error("Element {0} not found")]
    ElementNotFound(String),
    #[error("Invalid transition from {from:?} with event {to_event:?}")]
    InvalidTransition {
        from: EditorState,
        to_event: EditorEvent,
    },
    #[error("Inconsistent state")]
    InconsistentState,
}

#[allow(clippy::needless_pass_by_value)]
pub fn calculate_transition(
    current: &EditorState,
    event: EditorEvent,
    doc: &DiagramDocument,
) -> Result<EditorState, EditorError> {
    match event {
        EditorEvent::Escape => Ok(EditorState::Idle),
        EditorEvent::ClearHover => match current {
            EditorState::HoveringNode(_) | EditorState::HoveringEdge(_) | EditorState::Idle => {
                Ok(EditorState::Idle)
            }
            _ => Err(EditorError::InvalidTransition {
                from: current.clone(),
                to_event: event,
            }),
        },
        EditorEvent::HoverNode(ref id) => {
            if !doc.document.nodes.contains_key(id) {
                return Err(EditorError::ElementNotFound(id.to_string()));
            }
            match current {
                EditorState::HoveringNode(current_id) if current_id == id => Ok(current.clone()),
                EditorState::Idle | EditorState::HoveringNode(_) | EditorState::HoveringEdge(_) => {
                    Ok(EditorState::HoveringNode(id.clone()))
                }
                _ => Err(EditorError::InvalidTransition {
                    from: current.clone(),
                    to_event: event.clone(),
                }),
            }
        }
        EditorEvent::HoverEdge(ref id) => {
            if !doc.document.edges.contains_key(id) {
                return Err(EditorError::ElementNotFound(id.to_string()));
            }
            match current {
                EditorState::HoveringEdge(current_id) if current_id == id => Ok(current.clone()),
                EditorState::Idle | EditorState::HoveringNode(_) | EditorState::HoveringEdge(_) => {
                    Ok(EditorState::HoveringEdge(id.clone()))
                }
                _ => Err(EditorError::InvalidTransition {
                    from: current.clone(),
                    to_event: event.clone(),
                }),
            }
        }
        EditorEvent::EditNode(ref id) => {
            if !doc.document.nodes.contains_key(id) {
                return Err(EditorError::ElementNotFound(id.to_string()));
            }
            match current {
                EditorState::HoveringNode(hover_id) if hover_id == id => {
                    Ok(EditorState::EditingNode(id.clone()))
                }
                EditorState::EditingNode(edit_id) if edit_id == id => Ok(current.clone()),
                EditorState::EditingNode(_) => Ok(EditorState::EditingNode(id.clone())),
                _ => Err(EditorError::InvalidTransition {
                    from: current.clone(),
                    to_event: event.clone(),
                }),
            }
        }
        EditorEvent::EditEdge(ref id) => {
            if !doc.document.edges.contains_key(id) {
                return Err(EditorError::ElementNotFound(id.to_string()));
            }
            match current {
                EditorState::HoveringEdge(hover_id) if hover_id == id => {
                    Ok(EditorState::EditingEdge(id.clone()))
                }
                EditorState::EditingEdge(edit_id) if edit_id == id => Ok(current.clone()),
                EditorState::EditingEdge(_) => Ok(EditorState::EditingEdge(id.clone())),
                _ => Err(EditorError::InvalidTransition {
                    from: current.clone(),
                    to_event: event.clone(),
                }),
            }
        }
    }
}

pub fn apply_transition(
    canvas_state: &mut CanvasState,
    next_state: EditorState,
) -> Result<(), EditorError> {
    if next_state == EditorState::Idle {
        canvas_state.edit_value.set(String::new());
    }

    canvas_state.editor_state.set(next_state.clone());

    if next_state == EditorState::Idle && !canvas_state.edit_value.read().is_empty() {
        return Err(EditorError::InconsistentState);
    }

    Ok(())
}

#[derive(Clone, PartialEq)]
pub struct CanvasState {
    pub doc_signal: Signal<DiagramDocument>,
    pub dragging_icon: Signal<Option<DraggedIconPayload>>,
    pub history_signal: Signal<History>,
    pub tool_signal: Signal<ToolMode>,
    pub edge_style_default: Signal<EdgeStyle>,
    pub arrow_type_default: Signal<ArrowType>,
    pub interaction_mode: Signal<InteractionMode>,
    pub space_pressed: Signal<bool>,
    pub shift_pressed: Signal<bool>,
    pub ctrl_pressed: Signal<bool>,
    pub meta_pressed: Signal<bool>,
    pub drag_over: Signal<bool>,
    pub editor_state: Signal<EditorState>,
    pub edit_value: Signal<String>,
    pub nudge_batch_active: Signal<bool>,
    pub space_pan_active: Signal<bool>,
    pub viewport_size: Signal<(f64, f64)>,
    pub pending_pointer_sample: Signal<Option<(f64, f64)>>,
    pub pending_wheel_sample: Signal<Option<WheelSample>>,
    pub multi_touch_active: Signal<bool>,
    pub captured_pointer: Signal<Option<u32>>,
    pub active_pointers: Signal<HashSet<u32>>,
    pub canvas_origin: Signal<(f64, f64)>,
    pub ordered_node_cache: Memo<Vec<NodeId>>,
    pub db_tx: Option<Coroutine<diagram_models::envelope::EventEnvelope>>,
}

pub fn use_canvas_state() -> CanvasState {
    let app_state = use_context::<crate::app::AppState>();
    let doc_signal = app_state.document;
    let dragging_icon = app_state.dragging_icon;
    let history_signal = app_state.history;
    let tool_signal = app_state.tool_mode;
    let edge_style_default = app_state.edge_style;
    let arrow_type_default = app_state.arrow_type;
    let _toast = use_toast();

    let interaction_mode = use_signal(|| InteractionMode::Select);
    let space_pressed = use_signal(|| false);
    let shift_pressed = use_signal(|| false);
    let ctrl_pressed = use_signal(|| false);
    let meta_pressed = use_signal(|| false);
    let drag_over = use_signal(|| false);
    let editor_state = use_signal(|| EditorState::Idle);
    let edit_value = use_signal(String::new);
    let nudge_batch_active = use_signal(|| false);
    let space_pan_active = use_signal(|| false);
    let viewport_size = app_state.viewport_size;
    let pending_pointer_sample = use_signal(|| Option::<(f64, f64)>::None);
    let pending_wheel_sample = use_signal(|| Option::<WheelSample>::None);
    let multi_touch_active = use_signal(|| false);
    let captured_pointer = use_signal(|| Option::<u32>::None);
    let active_pointers = use_signal(HashSet::<u32>::new);
    let canvas_origin = use_signal(|| (0.0_f64, 0.0_f64));
    let ordered_node_cache = use_memo(move || {
        let doc = doc_signal.read();
        ordered_node_ids(&doc)
    });
    let db_tx = use_context::<Option<Coroutine<diagram_models::envelope::EventEnvelope>>>();

    use_keyboard_handler(
        doc_signal,
        history_signal,
        interaction_mode,
        tool_signal,
        space_pressed,
        shift_pressed,
        ctrl_pressed,
        meta_pressed,
        nudge_batch_active,
        space_pan_active,
        editor_state,
        edit_value,
        viewport_size,
        db_tx,
    );

    use_touch_handler(
        multi_touch_active,
        pending_pointer_sample,
        pending_wheel_sample,
        space_pan_active,
        interaction_mode,
    );

    use_middle_pan_handler();

    use_raf_handler(
        doc_signal,
        history_signal,
        interaction_mode,
        pending_pointer_sample,
        pending_wheel_sample,
        db_tx,
    );

    use_resize_handler(canvas_origin, viewport_size);

    CanvasState {
        doc_signal,
        dragging_icon,
        history_signal,
        tool_signal,
        edge_style_default,
        arrow_type_default,
        interaction_mode,
        space_pressed,
        shift_pressed,
        ctrl_pressed,
        meta_pressed,
        drag_over,
        editor_state,
        edit_value,
        nudge_batch_active,
        space_pan_active,
        viewport_size,
        pending_pointer_sample,
        pending_wheel_sample,
        multi_touch_active,
        captured_pointer,
        active_pointers,
        canvas_origin,
        ordered_node_cache,
        db_tx,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use diagram_models::document::{DiagramDocument, DocumentData, Node, NodeKind};
    use indexmap::IndexMap;
    use proptest::prelude::*;
    use uuid::Uuid;

    struct FsmDriver {
        doc: DiagramDocument,
        state: EditorState,
    }

    impl FsmDriver {
        fn new() -> Self {
            let mut nodes = IndexMap::new();
            let mut edges = IndexMap::new();
            nodes.insert(
                "node_a".to_string(),
                Node {
                    id: "node_a".to_string(),
                    kind: NodeKind::Task,
                    label: "A".to_string(),
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 100.0,
                    color: None,
                    is_group: false,
                    parent_id: None,
                },
            );
            nodes.insert(
                "node_b".to_string(),
                Node {
                    id: "node_b".to_string(),
                    kind: NodeKind::Task,
                    label: "B".to_string(),
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 100.0,
                    color: None,
                    is_group: false,
                    parent_id: None,
                },
            );
            edges.insert(
                "edge_1".to_string(),
                diagram_models::document::Edge {
                    id: "edge_1".to_string(),
                    source: "node_a".to_string(),
                    target: "node_b".to_string(),
                    label: None,
                    style: EdgeStyle::Solid,
                    color: None,
                    thickness: 1.0,
                    arrow_type: ArrowType::Normal,
                    control_points: vec![],
                },
            );
            Self {
                doc: DiagramDocument {
                    version: 1,
                    revision: diagram_models::document::Revision::INITIAL,
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
            self.state = EditorState::HoveringNode(id.to_string());
            self
        }

        fn given_editing_node(mut self, id: &str) -> Self {
            self.state = EditorState::EditingNode(id.to_string());
            self
        }

        fn apply(mut self, event: EditorEvent) -> Result<Self, EditorError> {
            self.state = calculate_transition(&self.state, event, &self.doc)?;
            Ok(self)
        }

        fn when_mouse_enters_node(self, id: &str) -> Result<Self, EditorError> {
            self.apply(EditorEvent::HoverNode(id.to_string()))
        }

        fn when_mouse_clicks_node(self, id: &str) -> Result<Self, EditorError> {
            self.apply(EditorEvent::EditNode(id.to_string()))
        }

        fn when_escape_pressed(self) -> Result<Self, EditorError> {
            self.apply(EditorEvent::Escape)
        }

        fn then_state_is_hovering(self, id: &str) -> Self {
            assert_eq!(self.state, EditorState::HoveringNode(id.to_string()));
            self
        }

        fn then_state_is_editing(self, id: &str) -> Self {
            assert_eq!(self.state, EditorState::EditingNode(id.to_string()));
            self
        }

        fn then_state_is_idle(self) -> Self {
            assert_eq!(self.state, EditorState::Idle);
            self
        }
    }

    #[test]
    fn test_transitions_to_hovering_when_mouse_enters_node() {
        let driver = FsmDriver::new().given_idle_canvas();
        driver
            .when_mouse_enters_node("node_a")
            .unwrap()
            .then_state_is_hovering("node_a");
    }

    #[test]
    fn test_transitions_to_editing_when_node_clicked() {
        let driver = FsmDriver::new().given_hovering_node("node_a");
        driver
            .when_mouse_clicks_node("node_a")
            .unwrap()
            .then_state_is_editing("node_a");
    }

    #[test]
    fn test_commits_edit_and_returns_to_idle_on_escape() {
        let driver = FsmDriver::new().given_editing_node("node_a");
        driver.when_escape_pressed().unwrap().then_state_is_idle();
    }

    #[test]
    fn test_P2_violation_returns_invalid_transition() {
        let driver = FsmDriver::new().given_editing_node("node_a");
        let result = driver.apply(EditorEvent::EditEdge("edge_1".to_string()));
        assert!(matches!(result, Err(EditorError::InvalidTransition { .. })));
    }

    #[test]
    fn test_P1_violation_returns_element_not_found() {
        let driver = FsmDriver::new().given_idle_canvas();
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
            Just(EditorEvent::HoverNode("node_a".to_string())),
            Just(EditorEvent::HoverNode("invalid".to_string())),
            Just(EditorEvent::EditNode("node_a".to_string())),
            Just(EditorEvent::EditNode("node_b".to_string())),
            Just(EditorEvent::HoverEdge("edge_1".to_string())),
            Just(EditorEvent::EditEdge("edge_1".to_string())),
        ]
    }
}
