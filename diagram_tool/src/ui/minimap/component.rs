#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::ui::minimap::models::{MinimapProjection, MinimapSnapshot, ProjectionKey};
use crate::ui::theme::{ACCENT, APP_FONT, BG_ELEVATED, BG_SURFACE, BORDER, TEXT_MUTED};
use diagram_models::document::{DiagramDocument, Revision};
use dioxus::prelude::*;

const PAD: f64 = 60.0;
const BASE_SIDE: f64 = 180.0;
const MIN_W: f64 = 120.0;
const MAX_W: f64 = 280.0;
const MIN_H: f64 = 80.0;
const MAX_H: f64 = 200.0;

#[component]
pub fn Minimap() -> Element {
    let mut doc_signal = use_context::<Signal<DiagramDocument>>();
    let viewport_size = use_context::<Signal<(f64, f64)>>();
    let mut dragging = use_signal(|| false);
    let mut cached_snapshot = use_signal(|| Option::<MinimapSnapshot>::None);
    let mut last_snapshot_revision = use_signal(|| Option::<Revision>::None);
    let mut cached_projection = use_signal(|| Option::<MinimapProjection>::None);
    let mut last_projection_key = use_signal(|| Option::<ProjectionKey>::None);

    let (cam_x, cam_y, zoom, revision) = {
        let doc = doc_signal.read();

        if doc.document.nodes.is_empty() {
            return rsx! {};
        }

        let needs_refresh = cached_snapshot.read().is_none()
            || last_snapshot_revision
                .read()
                .as_ref()
                .is_none_or(|cached| *cached != doc.revision);

        if needs_refresh {
            cached_snapshot.set(MinimapSnapshot::from_document(&doc.document));
            last_snapshot_revision.set(Some(doc.revision));
            last_projection_key.set(None);
        }

        (
            doc.editor_state.camera_x.0,
            doc.editor_state.camera_y.0,
            doc.editor_state.zoom.0,
            doc.revision,
        )
    };

    let snapshot = cached_snapshot.read();
    let Some(snapshot) = snapshot.as_ref() else {
        return rsx! {};
    };
    let doc_min_x = snapshot.min_x;
    let doc_min_y = snapshot.min_y;
    let doc_max_x = snapshot.max_x;
    let doc_max_y = snapshot.max_y;

    let (viewport_w, viewport_h) = *viewport_size.read();
    let vp_w = viewport_w.max(1.0) / zoom;
    let vp_h = viewport_h.max(1.0) / zoom;
    let vp_left = cam_x;
    let vp_top = cam_y;

    let min_x = doc_min_x.min(vp_left) - PAD;
    let min_y = doc_min_y.min(vp_top) - PAD;
    let max_x = doc_max_x.max(vp_left + vp_w) + PAD;
    let max_y = doc_max_y.max(vp_top + vp_h) + PAD;

    let world_w = (max_x - min_x).max(1.0);
    let world_h = (max_y - min_y).max(1.0);
    let aspect = world_w / world_h;
    let (mut view_w, mut view_h) = if aspect > 1.0 {
        let width = BASE_SIDE.round();
        (width, (width / aspect).round())
    } else {
        let height = BASE_SIDE.round();
        ((height * aspect).round(), height)
    };
    view_w = view_w.clamp(MIN_W, MAX_W);
    view_h = view_h.clamp(MIN_H, MAX_H);

    let scale = (view_w / world_w).min(view_h / world_h);

    let projection_key = ProjectionKey::from_state(revision, min_x, min_y, scale);
    if last_projection_key
        .read()
        .as_ref()
        .is_none_or(|cached| *cached != projection_key)
    {
        cached_projection.set(Some(snapshot.project(min_x, min_y, scale)));
        last_projection_key.set(Some(projection_key));
    }

    let projection = cached_projection.read();
    let Some(projection) = projection.as_ref() else {
        return rsx! {};
    };
    let vp_x = (vp_left - min_x) * scale;
    let vp_y = (vp_top - min_y) * scale;

    let mut nav_to = move |screen_x: f64, screen_y: f64| {
        let center_x = (screen_x / scale) + min_x;
        let center_y = (screen_y / scale) + min_y;
        let doc = doc_signal.read();
        let zoom = doc.editor_state.zoom.0;
        let viewport = *viewport_size.read();
        let vp_w = viewport.0.max(1.0) / zoom;
        let vp_h = viewport.1.max(1.0) / zoom;
        let left = center_x - (vp_w / 2.0);
        let top = center_y - (vp_h / 2.0);
        let next_camera_x = left;
        let next_camera_y = top;
        let changed = (doc.editor_state.camera_x.0 - next_camera_x).abs() > 0.25
            || (doc.editor_state.camera_y.0 - next_camera_y).abs() > 0.25;
        if changed {
            drop(doc);
            doc_signal.with_mut(|doc_mut| {
                doc_mut.editor_state.camera_x.0 = next_camera_x;
                doc_mut.editor_state.camera_y.0 = next_camera_y;
            });
        }
    };

    rsx! {
        div {
            "data-testid": "minimap-root",
            style: "position: absolute; right: 12px; bottom: 12px; width: {view_w}px; height: {view_h}px; border: 1px solid {BORDER}; border-radius: 10px; background: linear-gradient(180deg, {BG_ELEVATED}f2 0%, {BG_SURFACE}ea 100%); backdrop-filter: blur(8px); overflow: hidden; z-index: 20; user-select:none; box-shadow: 0 8px 20px color-mix(in oklch, black 28%, transparent);",
            onmousedown: move |evt| {
                evt.stop_propagation();
                dragging.set(true);
                let c = evt.data.coordinates().element();
                nav_to(c.x, c.y);
            },
            onmousemove: move |evt| {
                if *dragging.read() {
                    let c = evt.data.coordinates().element();
                    nav_to(c.x, c.y);
                }
            },
            onmouseup: move |_| dragging.set(false),
            onmouseleave: move |_| dragging.set(false),

            svg {
                width: "{view_w}",
                height: "{view_h}",
                for &(sx, sy, tx, ty) in projection.edge_segments.iter() {
                    line {
                        x1: "{sx}",
                        y1: "{sy}",
                        x2: "{tx}",
                        y2: "{ty}",
                        stroke: "color-mix(in oklch, {TEXT_MUTED} 78%, transparent)",
                        stroke_width: "0.7",
                        opacity: "0.7",
                    }
                }
                for &(is_subgraph, x, y, w, h, provider_color) in projection.node_rects.iter() {
                    rect {
                        x: "{x}",
                        y: "{y}",
                        width: "{w}",
                        height: "{h}",
                        rx: "1.5",
                        fill: if is_subgraph { "none" } else { provider_color },
                        stroke: if is_subgraph {
                            format!("color-mix(in oklch, {TEXT_MUTED} 55%, transparent)")
                        } else { String::from(ACCENT) },
                        stroke_width: "0.8",
                        opacity: "0.85",
                    }
                }
                rect {
                    "data-testid": "minimap-viewport",
                    x: "{vp_x}",
                    y: "{vp_y}",
                    width: "{(vp_w * scale).max(4.0)}",
                    height: "{(vp_h * scale).max(4.0)}",
                    fill: "color-mix(in oklch, {ACCENT} 20%, transparent)",
                    stroke: "{ACCENT}",
                    stroke_width: "1.2",
                    rx: "2",
                }
            }
            div {
                style: "position: absolute; top: 4px; right: 6px; color: {TEXT_MUTED}; font-size: 10px; font-family: {APP_FONT};",
                "{(zoom * 100.0).round()}%"
            }
        }
    }
}
