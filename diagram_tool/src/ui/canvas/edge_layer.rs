mod interaction;

use self::interaction::{handle_edge_double_click, handle_edge_mouse_down, EdgeEditor, EdgeLabel};
use crate::ui::canvas::canvas_view::markers::EdgeMarkers;
use crate::ui::canvas::canvas_view::{edge_endpoints, edge_label_position, edge_path};
use crate::ui::canvas::edge_types::{
    BendPointProps, EdgeContext, EdgeItemProps, EdgeLayerProps, EdgePathProps,
};
use crate::ui::theme::{EDGE_DEFAULT, EDGE_SELECTED};
use diagram_models::document::{DiagramDocument, Edge, EdgeId, EdgeStyle, Node};
use dioxus::prelude::*;
use serde_json::Value;

#[allow(non_snake_case)]
#[allow(clippy::needless_collect)]
pub fn EdgeLayer(props: EdgeLayerProps) -> Element {
    // Subscribe to viewport/geometry triggers, plus document revision for edge-only
    // updates such as direction and arrow-type changes.
    let trigger = props.node_viewport_trigger.read();
    let _geometry_tick = *props.geometry_render_tick.read();
    let _document_revision = props.doc_signal.read().revision;
    let (camera_x, camera_y, zoom, selected_items) = (trigger.0, trigger.1, trigger.2, &trigger.3);

    // Peek at document for edge/node geometry after the revision subscription above.
    let doc = props.doc_signal.peek();
    let canvas_origin = canvas_domain::CanvasCoord::from(*props.canvas_origin.read());
    let ctx = EdgeContext {
        camera_x,
        camera_y,
        zoom,
        canvas_origin,
    };
    let culling_rect = get_culling_rect(&ctx, &props);

    let edges: Vec<_> = get_visible_edges(&doc, culling_rect)
        .map(|(id, e, s, t)| (id.clone(), e.clone(), s.clone(), t.clone()))
        .collect();

    rsx! {
        for (id, edge, src, tgt) in edges {
            EdgeItem {
                id: id.clone(), edge, src, tgt, ctx: ctx.clone(),
                is_selected: selected_items.contains(id.as_str()),
                layer_props: props.clone()
            }
        }
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
    let (src_pt, tgt_pt) = edge_endpoints(&props.edge, &props.src, &props.tgt);

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

#[allow(non_snake_case)]
pub fn EdgePath(props: EdgePathProps) -> Element {
    let edge_id = props.id.clone();
    let edge_id_for_mouse_down = edge_id.clone();
    let edge_id_for_double_click = edge_id.clone();
    let layer_props_for_mouse_down = props.layer_props.clone();
    let layer_props_for_double_click = props.layer_props.clone();
    let edge_ctx_for_double_click = props.ctx.clone();
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
    let hit_width = if props.is_selected { 22.0 } else { 18.0 };
    #[allow(clippy::manual_unwrap_or_default, clippy::manual_unwrap_or)]
    let marker_start_str = match props.markers.marker_start.as_deref() {
        Some(s) => s,
        None => "",
    };

    rsx! {
        g {
            path {
                "data-node-kind": "edge-hit-target",
                "data-testid": "edge-hit-{edge_id}",
                d: "{props.d}",
                fill: "none",
                stroke: "transparent",
                stroke_width: "{hit_width}",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                pointer_events: "stroke",
                cursor: "pointer",
                onmousedown: move |evt| handle_edge_mouse_down(evt, edge_id_for_mouse_down.clone(), layer_props_for_mouse_down.clone()),
                ondoubleclick: move |evt| handle_edge_double_click(evt, edge_id_for_double_click.clone(), edge_ctx_for_double_click.clone(), layer_props_for_double_click.clone())
            }
            path {
                "data-node-kind": "edge", "data-testid": "edge-{edge_id}", d: "{props.d}", fill: "none",
                stroke: "{stroke_color}", stroke_width: "{stroke_width}", stroke_dasharray: "{dash}",
                marker_end: "{props.markers.marker_end}", marker_start: marker_start_str,
                pointer_events: "none"
            }
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
            pointer_events: "all",
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
