//! Type definitions for edge layer components

use crate::history::History;
use canvas_domain::CanvasCoord;
use diagram_models::document::{DiagramDocument, Edge, EdgeId, EdgeStyle, Node};
use dioxus::prelude::*;
use im::HashSet as ImHashSet;

#[derive(Clone, PartialEq)]
pub struct EdgeContext {
    pub camera_x: f64,
    pub camera_y: f64,
    pub zoom: f64,
    pub canvas_origin: CanvasCoord,
}

#[derive(Clone, PartialEq, Props)]
pub struct EdgeLayerProps {
    pub doc_signal: Signal<DiagramDocument>,
    pub history_signal: Signal<History>,
    pub geometry_render_tick: Signal<u64>,
    pub editor_state: Signal<crate::ui::canvas::state::EditorState>,
    pub edit_value: Signal<String>,
    pub viewport_size: Signal<(f64, f64)>,
    pub interaction_mode: Signal<canvas_domain::interaction_reducer::InteractionMode>,
    pub canvas_origin: Signal<(f64, f64)>,
    /// Lightweight trigger Memo for camera/selection data. `EdgeLayer` subscribes
    /// to this instead of the full `doc_signal` to avoid unnecessary re-renders.
    pub node_viewport_trigger: Memo<(u64, f64, f64, f64, ImHashSet<String>)>,
    pub db_tx: Option<Coroutine<diagram_models::envelope::EventEnvelope>>,
}

#[derive(Clone, PartialEq, Props)]
pub struct EdgeItemProps {
    pub id: EdgeId,
    pub edge: Edge,
    pub src: Node,
    pub tgt: Node,
    pub is_selected: bool,
    pub ctx: EdgeContext,
    pub layer_props: EdgeLayerProps,
}

#[derive(Clone, PartialEq, Props)]
pub struct EdgePathProps {
    pub id: EdgeId,
    pub d: String,
    pub ctx: EdgeContext,
    pub markers: crate::ui::canvas::canvas_view::markers::EdgeMarkers,
    pub style: EdgeStyle,
    pub is_selected: bool,
    pub layer_props: EdgeLayerProps,
}

#[derive(Clone, PartialEq, Props)]
pub struct BendPointProps {
    pub id: EdgeId,
    pub bp: diagram_models::document::edge::SerializedPoint,
    pub index: usize,
    pub ctx: EdgeContext,
    pub layer_props: EdgeLayerProps,
}

#[derive(Clone, PartialEq, Props)]
pub struct EdgeEditorProps {
    pub id: EdgeId,
    pub mid_x: f64,
    pub mid_y: f64,
    pub layer_props: EdgeLayerProps,
}

#[derive(Clone, PartialEq, Props)]
pub struct EdgeLabelProps {
    pub label: String,
    pub mid_x: f64,
    pub mid_y: f64,
    pub zoom: f64,
    pub font_size: f64,
    pub is_selected: bool,
}

#[derive(Clone, PartialEq)]
pub enum EditEndReason {
    EnterPressed,
    Blur,
}
