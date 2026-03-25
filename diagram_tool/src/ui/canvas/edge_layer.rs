use crate::ui::canvas::canvas_view::markers::EdgeMarkers;
use crate::ui::canvas::canvas_view::{edge_label_position, edge_path};
use crate::ui::canvas::edge_types::{
    BendPointProps, EdgeContext, EdgeEditorProps, EdgeItemProps, EdgeLabelProps, EdgeLayerProps,
    EdgePathProps, EditEndReason,
};
use crate::ui::theme::{EDGE_DEFAULT, EDGE_SELECTED};
use canvas_domain::interaction_reducer::commit_inline_edit;
use diagram_models::document::{DiagramDocument, Edge, EdgeId, EdgeStyle, Node};
use dioxus::html::input_data::keyboard_types::Key;
use dioxus::prelude::*;
use serde_json::Value;

#[allow(non_snake_case)]
#[allow(clippy::needless_collect)]
pub fn EdgeLayer(props: EdgeLayerProps) -> Element {
    let doc = props.doc_signal.read();
    let ctx = get_edge_context(&doc, &props);
    let culling_rect = get_culling_rect(&ctx, &props);

    let edges: Vec<_> = get_visible_edges(&doc, culling_rect)
        .map(|(id, e, s, t)| (id.clone(), e.clone(), s.clone(), t.clone()))
        .collect();

    rsx! {
        for (id, edge, src, tgt) in edges {
            EdgeItem {
                id: id.clone(), edge, src, tgt, ctx: ctx.clone(),
                is_selected: doc.editor_state.selected_items.contains(id.as_str()),
                layer_props: props.clone()
            }
        }
    }
}

fn get_edge_context(doc: &DiagramDocument, props: &EdgeLayerProps) -> EdgeContext {
    EdgeContext {
        camera_x: doc.editor_state.camera_x.0,
        camera_y: doc.editor_state.camera_y.0,
        zoom: doc.editor_state.zoom.0,
        canvas_origin: canvas_domain::CanvasCoord::from(*props.canvas_origin.read()),
    }
}

fn get_culling_rect(ctx: &EdgeContext, props: &EdgeLayerProps) -> (f64, f64, f64, f64) {
    let (vw, vh) = *props.viewport_size.read();
    (
        ctx.camera_x - vw,
        ctx.camera_y - vh,
        ctx.camera_x + (vw / ctx.zoom) * 2.0,
        ctx.camera_y + (vh / ctx.zoom) * 2.0,
    )
}

#[allow(non_snake_case)]
#[allow(clippy::redundant_clone)]
pub fn EdgeItem(props: EdgeItemProps) -> Element {
    let (s, t) = get_edge_screen_coords(&props);
    let d = edge_path(s.0, s.1, t.0, t.1, &props.edge);
    let (mid_x, mid_y) = edge_label_position(s.0, s.1, t.0, t.1, &props.edge);
    let markers = EdgeMarkers::for_edge(
        props.edge.directed,
        is_bidir(&props.edge),
        props.is_selected,
    );

    rsx! {
        g {
            EdgePath { id: props.id.clone(), d, ctx: props.ctx.clone(), markers, style: props.edge.style, is_selected: props.is_selected, layer_props: props.layer_props.clone() }
            for (i, bp) in props.edge.bend_points.iter().enumerate() {
                BendPoint { id: props.id.clone(), bp: bp.clone(), index: i, ctx: props.ctx.clone(), layer_props: props.layer_props.clone() }
            }
            if is_edge_editing(&props) {
                EdgeEditor { id: props.id.clone(), mid_x, mid_y, layer_props: props.layer_props.clone() }
            } else {
                EdgeLabel { label: props.edge.label.clone(), mid_x, mid_y, zoom: props.ctx.zoom, font_size: get_font_size(&props.edge), is_selected: props.is_selected }
            }
        }
    }
}

fn get_edge_screen_coords(props: &EdgeItemProps) -> ((f64, f64), (f64, f64)) {
    let src_pt = get_port_pos(&props.edge.source_port, &props.src);
    let tgt_pt = get_port_pos(&props.edge.target_port, &props.tgt);

    let s = canvas_domain::perf::to_screen_coords(
        canvas_domain::CanvasCoord(src_pt.0, src_pt.1),
        canvas_domain::CanvasCoord(props.ctx.camera_x, props.ctx.camera_y),
        props.ctx.zoom,
    );
    let t = canvas_domain::perf::to_screen_coords(
        canvas_domain::CanvasCoord(tgt_pt.0, tgt_pt.1),
        canvas_domain::CanvasCoord(props.ctx.camera_x, props.ctx.camera_y),
        props.ctx.zoom,
    );
    ((s.0, s.1), (t.0, t.1))
}

fn is_edge_editing(props: &EdgeItemProps) -> bool {
    matches!(*props.layer_props.editor_state.read(), crate::ui::canvas::state::EditorState::EditingEdge(ref eid) if eid == &props.id)
}

fn get_font_size(edge: &Edge) -> f64 {
    match edge.font_size {
        Some(f) => f.0,
        None => 10.0,
    }
}

fn is_bidir(e: &Edge) -> bool {
    e.metadata
        .get("bidirectional")
        .is_some_and(|v| v == &Value::Bool(true))
}

fn get_port_pos(port: &Option<diagram_models::port::PortAnchor>, node: &Node) -> (f64, f64) {
    let p = port.as_ref().map_or_else(
        || {
            diagram_models::geometry::Point::new(
                node.x.0 + node.width.0 / 2.0,
                node.y.0 + node.height.0 / 2.0,
            )
        },
        |p| diagram_models::port::compute_port_absolute_position(node, p),
    );
    (p.x, p.y)
}

#[allow(non_snake_case)]
pub fn EdgePath(props: EdgePathProps) -> Element {
    let dash = match props.style {
        EdgeStyle::Dashed => "8,4",
        EdgeStyle::Dotted => "2,4",
        EdgeStyle::Solid => "",
    };
    let stroke_color = if props.is_selected {
        EDGE_SELECTED
    } else {
        EDGE_DEFAULT
    };
    let stroke_width = if props.is_selected { 2.5 } else { 1.5 };
    #[allow(clippy::manual_unwrap_or_default, clippy::manual_unwrap_or)]
    let marker_start_str = match props.markers.marker_start.as_deref() {
        Some(s) => s,
        None => "",
    };

    rsx! {
        path {
            "data-node-kind": "edge", "data-testid": "edge-{props.id}", d: "{props.d}", fill: "none",
            stroke: "{stroke_color}", stroke_width: "{stroke_width}", stroke_dasharray: "{dash}",
            marker_end: "{props.markers.marker_end}", marker_start: marker_start_str,
            ondoubleclick: move |evt| handle_edge_double_click(evt, props.id.clone(), props.ctx.clone(), props.layer_props.clone())
        }
    }
}

#[allow(non_snake_case)]
pub fn BendPoint(mut props: BendPointProps) -> Element {
    let c = canvas_domain::perf::to_screen_coords(
        canvas_domain::CanvasCoord(props.bp.x.0, props.bp.y.0),
        canvas_domain::CanvasCoord(props.ctx.camera_x, props.ctx.camera_y),
        props.ctx.zoom,
    );
    rsx! {
        circle {
            cx: "{c.0}", cy: "{c.1}", r: "{4.0 * props.ctx.zoom.max(1.0)}", fill: "{EDGE_SELECTED}", cursor: "move",
            onmousedown: move |evt| {
                evt.stop_propagation();
                let current_doc = props.layer_props.doc_signal.read().clone();
                let current_hist = props.layer_props.history_signal.read().clone();
                *props.layer_props.history_signal.write() = current_hist.push(current_doc);
                props.layer_props.interaction_mode.set(canvas_domain::interaction_reducer::InteractionMode::DraggingBendPoint { edge_id: props.id.clone(), bend_index: props.index });
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn EdgeLabel(props: EdgeLabelProps) -> Element {
    if !props.label.is_empty() && props.zoom >= 0.3 {
        rsx! { text { x: "{props.mid_x}", y: "{props.mid_y - 6.0}", text_anchor: "middle", class: "fill-[var(--text-muted)]", style: "font-size:{props.font_size * props.zoom}px;", "{props.label}" } }
    } else if props.is_selected && props.zoom >= 0.3 {
        rsx! { text { x: "{props.mid_x}", y: "{props.mid_y - 6.0}", text_anchor: "middle", class: "fill-[var(--text-muted)] opacity-60 text-[9px]", "label" } }
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

#[must_use]
pub fn get_visible_edges(
    doc: &DiagramDocument,
    culling_rect: (f64, f64, f64, f64),
) -> impl Iterator<Item = (&EdgeId, &Edge, &Node, &Node)> + use<'_> {
    let (c_min_x, c_min_y, c_max_x, c_max_y) = culling_rect;
    doc.document.edges.iter().filter_map(move |(id, edge)| {
        let src = doc.document.nodes.get(&edge.source)?;
        let tgt = doc.document.nodes.get(&edge.target)?;

        let min_x = src.x.0.min(tgt.x.0);
        let min_y = src.y.0.min(tgt.y.0);
        let max_x = (src.x.0 + src.width.0).max(tgt.x.0 + tgt.width.0);
        let max_y = (src.y.0 + src.height.0).max(tgt.y.0 + tgt.height.0);

        let visible = max_x >= c_min_x && min_x <= c_max_x && max_y >= c_min_y && min_y <= c_max_y;
        visible.then_some((id, edge, src, tgt))
    })
}

fn handle_edge_double_click(
    evt: Event<dioxus::prelude::MouseData>,
    id: EdgeId,
    ctx: EdgeContext,
    mut layer: EdgeLayerProps,
) {
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
