use crate::ui::canvas::edge_types::{
    EdgeContext, EdgeEditorProps, EdgeLabelProps, EdgeLayerProps, EditEndReason,
};
use crate::ui::interaction::{select_single, toggle_selection};
use canvas_domain::interaction_reducer::commit_inline_edit;
use diagram_models::document::EdgeId;
use dioxus::html::input_data::keyboard_types::Key;
use dioxus::prelude::*;

#[allow(non_snake_case)]
pub fn EdgeLabel(props: EdgeLabelProps) -> Element {
    if !props.label.is_empty() && props.zoom >= 0.3 {
        rsx! { text { x: "{props.mid_x}", y: "{props.mid_y - 6.0}", text_anchor: "middle", class: "fill-[var(--text-muted)]", style: "font-size:{props.font_size * props.zoom}px;", "{props.label}" } }
    } else if props.is_selected && props.zoom >= 0.3 {
        rsx! { text { x: "{props.mid_x}", y: "{props.mid_y - 6.0}", text_anchor: "middle", class: "fill-[var(--text-muted)] opacity-60 text-[9px]", pointer_events: "none", "label" } }
    } else {
        rsx! {}
    }
}

#[allow(non_snake_case)]
pub fn EdgeEditor(props: EdgeEditorProps) -> Element {
    let mut val_sig = props.layer_props.edit_value;
    let layer_blur = props.layer_props.clone();
    let layer_key = props.layer_props.clone();
    rsx! {
        foreignObject {
            x: "{props.mid_x - 50.0}", y: "{props.mid_y - 12.0}", width: "100", height: "24",
            input {
                value: "{props.layer_props.edit_value}", class: "pointer-events-auto px-[6px] py-[2px] rounded border border-solid border-[var(--accent)] bg-[var(--bg-base)] text-[var(--text-main)] w-[100px] h-[22px] text-[11px]",
                onmousedown: move |evt| evt.stop_propagation(), oninput: move |evt| val_sig.set(evt.value()),
                onblur: move |_| end_edit(layer_blur.clone(), EditEndReason::Blur),
                onkeydown: move |evt| {
                    if evt.key() == Key::Enter {
                        end_edit(layer_key.clone(), EditEndReason::EnterPressed);
                    }
                }
            }
        }
    }
}

fn end_edit(mut layer: EdgeLayerProps, reason: EditEndReason) {
    if matches!(reason, EditEndReason::Blur) {
        layer
            .editor_state
            .set(crate::ui::canvas::state::EditorState::Idle);
        return;
    }
    let (n, e) = match *layer.editor_state.read() {
        crate::ui::canvas::state::EditorState::EditingNode(ref id) => (Some(id.clone()), None),
        crate::ui::canvas::state::EditorState::EditingEdge(ref id) => (None, Some(id.clone())),
        _ => (None, None),
    };
    commit_inline_edit(
        layer.doc_signal,
        layer.history_signal,
        n,
        e,
        layer.edit_value,
        layer.db_tx,
    )
    .ok();
    layer
        .editor_state
        .set(crate::ui::canvas::state::EditorState::Idle);
}

pub fn handle_edge_double_click(
    evt: Event<dioxus::prelude::MouseData>,
    id: EdgeId,
    ctx: EdgeContext,
    mut layer: EdgeLayerProps,
) {
    if !evt.data.modifiers().shift() {
        return;
    }

    evt.stop_propagation();
    let coords = evt.data().coordinates().client();
    let origin = match crate::ui::canvas::document_ops::sync_canvas_origin() {
        Some(o) => o,
        None => (ctx.canvas_origin.0, ctx.canvas_origin.1),
    };
    let raw = canvas_domain::perf::to_canvas_coords(
        canvas_domain::ScreenCoord(coords.x - origin.0, coords.y - origin.1),
        canvas_domain::CanvasCoord(ctx.camera_x, ctx.camera_y),
        ctx.zoom,
    );
    let click_point = match diagram_models::geometry::FinitePoint::new(raw.0, raw.1) {
        Some(p) => p,
        None => return,
    };
    let doc = layer.doc_signal.read().clone();
    if let Ok(new_doc) = diagram_models::document::routing_interactions::handle_bend_point_insertion(
        &doc,
        &id,
        click_point,
    ) {
        let current_hist = layer.history_signal.read().clone();
        *layer.history_signal.write() = current_hist.push(doc);
        layer.doc_signal.set(new_doc);
    }
}

pub fn handle_edge_mouse_down(
    evt: Event<dioxus::prelude::MouseData>,
    id: EdgeId,
    mut layer: EdgeLayerProps,
) {
    use dioxus::html::input_data::MouseButton;

    if evt.data.trigger_button() != Some(MouseButton::Primary) {
        return;
    }

    evt.stop_propagation();
    let (node_target, edge_target) = match *layer.editor_state.read() {
        crate::ui::canvas::state::EditorState::EditingNode(ref edit_id) => {
            (Some(edit_id.clone()), None)
        }
        crate::ui::canvas::state::EditorState::EditingEdge(ref edit_id) => {
            (None, Some(edit_id.clone()))
        }
        _ => (None, None),
    };
    if node_target.is_some() || edge_target.is_some() {
        commit_inline_edit(
            layer.doc_signal,
            layer.history_signal,
            node_target,
            edge_target,
            layer.edit_value,
            layer.db_tx,
        )
        .ok();
    }
    let modifiers = evt.data.modifiers();
    let additive = modifiers.shift() || modifiers.ctrl() || modifiers.meta();
    layer.doc_signal.with_mut(|doc| {
        doc.editor_state.selected_items = if additive {
            toggle_selection(&doc.editor_state.selected_items, id.to_string().as_str())
        } else if doc.editor_state.selected_items.contains(id.as_str()) {
            doc.editor_state.selected_items.clone()
        } else {
            select_single(id.to_string())
        };
    });
}
