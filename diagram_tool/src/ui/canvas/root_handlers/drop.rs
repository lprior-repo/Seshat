use crate::app::DraggedIconPayload;
use crate::history::History;
use crate::ui::canvas::document_ops::{fallback_icon_label, icon_tags, sync_canvas_origin};
use crate::ui::grid::snap_point;
use canvas_domain::perf::to_canvas_coords;
use diagram_models::document::{DiagramDocument, Node, NodeId, NodeKind, NodeStyle, OrderedFloat};
use diagram_models::LockState;
use dioxus::prelude::*;
use im::HashMap;
use serde_json::Value;
use uuid::Uuid;

pub fn handle_drop(
    evt: DragEvent,
    mut drag_over: Signal<bool>,
    mut dragging_icon: Signal<Option<DraggedIconPayload>>,
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    canvas_origin: Signal<(f64, f64)>,
) {
    evt.prevent_default();
    drag_over.set(false);

    dragging_icon.with_mut(|dragging| {
        if let Some(payload) = dragging.take() {
            let icon_key = payload.icon_key;
            let image_data_url = payload.image_data_url;
            let current = doc_signal.read().clone();
            let history = history_signal.read().clone();
            *history_signal.write() = history.push(current);
            let derived_label = payload
                .label
                .filter(|label| !label.trim().is_empty())
                .unwrap_or_else(|| fallback_icon_label(&icon_key));
            let tags = icon_tags(&icon_key);

            doc_signal.with_mut(|doc| {
                let coords = evt.data.coordinates().client();
                let origin = sync_canvas_origin().unwrap_or_else(|| *canvas_origin.read());
                let local_x = coords.x - origin.0;
                let local_y = coords.y - origin.1;
                let canvas_domain::CanvasCoord(x, y) = to_canvas_coords(
                    canvas_domain::ScreenCoord(local_x, local_y),
                    canvas_domain::CanvasCoord(
                        doc.editor_state.camera_x.0,
                        doc.editor_state.camera_y.0,
                    ),
                    doc.editor_state.zoom.0,
                );
                let (x, y) = snap_point(
                    (x - 32.0, y - 32.0),
                    doc.editor_state.snap_to_grid,
                    doc.editor_state.grid_size,
                );
                let metadata = image_data_url.clone().map_or_else(HashMap::new, |image| {
                    HashMap::new().update("icon_data_url".to_string(), Value::String(image))
                });
                let id = NodeId::new(Uuid::new_v4().to_string());
                let _ = doc.document.nodes.insert(
                    id.clone(),
                    Node {
                        kind: NodeKind::Node,
                        icon: icon_key,
                        label: derived_label,
                        x: OrderedFloat(x),
                        y: OrderedFloat(y),
                        width: OrderedFloat(64.0),
                        height: OrderedFloat(64.0),
                        font_size: None,
                        font_weight: None,
                        lock_state: LockState::Unlocked,
                        parent: None,
                        dag_rank: None,
                        tags: tags.into(),
                        metadata,
                        z_index: 0,
                        style: Some(NodeStyle::default()),
                        collapsed: None,
                    },
                );
                doc.editor_state.selected_items.clear();
                let _ = doc.editor_state.selected_items.insert(id.to_string());
                doc.revision = doc.revision.increment();
            });
        }
    });
}
