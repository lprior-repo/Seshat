use crate::ui::canvas::canvas_view::find_edge_at;
use crate::ui::canvas::document_ops::sync_canvas_origin;
use crate::ui::canvas::state::CanvasState;
use crate::ui::editor::ToolMode;
use crate::ui::grid::snap_point;
use crate::ui::interaction::select_single;
use canvas_domain::perf::to_canvas_coords;
use diagram_models::document::{LockState, Node, NodeId, NodeKind, NodeStyle, OrderedFloat};
use dioxus::prelude::*;
use im::HashMap;
use uuid::Uuid;

pub fn handle_double_click(state: CanvasState, evt: Event<dioxus::prelude::MouseData>) {
    let mut doc_signal = state.doc_signal;
    let mut history_signal = state.history_signal;
    let tool_signal = state.tool_signal;
    let mut editor_state = state.editor_state;
    let mut edit_value = state.edit_value;
    let ordered_node_cache = state.ordered_node_cache;
    let canvas_origin = state.canvas_origin;

    let coords = evt.data.coordinates().client();
    let origin = sync_canvas_origin().unwrap_or_else(|| *canvas_origin.read());
    let local_x = coords.x - origin.0;
    let local_y = coords.y - origin.1;
    let doc = doc_signal.read().clone();
    let pos = to_canvas_coords(
        canvas_domain::ScreenCoord(local_x, local_y),
        canvas_domain::CanvasCoord(doc.editor_state.camera_x.0, doc.editor_state.camera_y.0),
        doc.editor_state.zoom.0,
    );

    // Extend hit area 20px below the node body to cover the label rendered at bottom:-18px
    let hit_node = ordered_node_cache.read().iter().rev().find_map(|id| {
        doc.document.nodes.get(id).and_then(|node| {
            (pos.0 >= node.x.0
                && pos.0 <= node.x.0 + node.width.0
                && pos.1 >= node.y.0
                && pos.1 <= node.y.0 + node.height.0 + 20.0)
                .then(|| (id.clone(), node.label.clone()))
        })
    });

    if let Some((nid, label)) = hit_node {
        editor_state.set(crate::ui::canvas::state::EditorState::EditingNode(nid));
        edit_value.set(label);
        return;
    }

    if let Some(eid) = find_edge_at(&doc, pos.0, pos.1) {
        let label = doc
            .document
            .edges
            .get(&eid)
            .map_or_else(String::new, |e| e.label.clone());
        doc_signal.with_mut(|d| {
            d.editor_state.selected_items = select_single(eid.to_string());
        });
        editor_state.set(crate::ui::canvas::state::EditorState::EditingEdge(eid));
        edit_value.set(label);
        return;
    }

    // Double-click on empty canvas creates a new node in Select mode
    let tool = *tool_signal.read();
    if tool == ToolMode::Select {
        let id = NodeId::new(Uuid::new_v4().to_string());
        let current = doc_signal.read().clone();
        let history = history_signal.read().clone();
        *history_signal.write() = history.push(current);
        doc_signal.with_mut(|d| {
            let (x, y) = snap_point(
                (pos.0, pos.1),
                d.editor_state.snap_to_grid,
                d.editor_state.grid_size,
            );
            let _ = d.document.nodes.insert(
                id.clone(),
                Node {
                    kind: NodeKind::Node,
                    icon: String::new(),
                    label: String::from("Node"),
                    x: OrderedFloat(x - 32.0),
                    y: OrderedFloat(y - 32.0),
                    width: OrderedFloat(64.0),
                    height: OrderedFloat(64.0),
                    font_size: None,
                    font_weight: None,
                    lock_state: LockState::Unlocked,
                    parent: None,
                    dag_rank: None,
                    tags: im::Vector::new(),
                    metadata: HashMap::new(),
                    z_index: 0,
                    style: Some(NodeStyle::default()),
                    collapsed: None,
                },
            );
            d.editor_state.selected_items.clear();
            let _ = d.editor_state.selected_items.insert(id.to_string());
            d.revision = d.revision.increment();
        });
        editor_state.set(crate::ui::canvas::state::EditorState::Idle);
        edit_value.set(String::new());
    }
}
