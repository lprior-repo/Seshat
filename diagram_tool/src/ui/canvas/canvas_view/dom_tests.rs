#![cfg(test)]

use crate::ui::canvas::canvas_view::edge_preview::edge_preview_overlay;
use crate::ui::canvas::canvas_view::rubber_band::rubber_band_overlay;
use crate::ui::canvas::canvas_view::selection_handles::selection_handles_overlay;
use crate::ui::canvas::canvas_view::subgraph_preview::subgraph_preview_overlay;
use canvas_domain::interaction_reducer::InteractionMode;
use canvas_domain::{CanvasCoord, ScreenCoord};
use diagram_models::document::DiagramDocument;
use dioxus::prelude::*;

fn to_screen(c: CanvasCoord, _cam: CanvasCoord, zoom: f64) -> ScreenCoord {
    ScreenCoord(c.0 * zoom, c.1 * zoom)
}

#[test]
fn edge_preview_renders_empty_when_idle() {
    #[component]
    fn App() -> Element {
        let doc = DiagramDocument::default();
        let mode = InteractionMode::Select;
        edge_preview_overlay(&mode, &doc, to_screen)
    }

    let mut vdom = VirtualDom::new(App);
    vdom.rebuild_in_place();
    let html = dioxus_ssr::render(&vdom);
    assert_eq!(html, "");
}

#[test]
fn subgraph_preview_renders_empty_when_idle() {
    #[component]
    fn App() -> Element {
        let doc = DiagramDocument::default();
        let mode = InteractionMode::Select;
        subgraph_preview_overlay(&mode, &doc, to_screen)
    }

    let mut vdom = VirtualDom::new(App);
    vdom.rebuild_in_place();
    let html = dioxus_ssr::render(&vdom);
    assert_eq!(html, "");
}

#[test]
fn rubber_band_renders_empty_when_idle() {
    #[component]
    fn App() -> Element {
        let doc = DiagramDocument::default();
        let mode = InteractionMode::Select;
        rubber_band_overlay(&mode, &doc, to_screen)
    }

    let mut vdom = VirtualDom::new(App);
    vdom.rebuild_in_place();
    let html = dioxus_ssr::render(&vdom);
    assert_eq!(html, "");
}

#[test]
fn rubber_band_renders_rect_when_active() {
    #[component]
    fn App() -> Element {
        let doc = DiagramDocument::default();
        let mode = InteractionMode::RubberBand {
            start: (10.0, 10.0),
            current: (20.0, 20.0),
        };
        rubber_band_overlay(&mode, &doc, to_screen)
    }

    let mut vdom = VirtualDom::new(App);
    vdom.rebuild_in_place();
    let html = dioxus_ssr::render(&vdom);
    assert!(html.contains("<rect"));
    assert!(html.contains("width=\"10"));
    assert!(html.contains("height=\"10"));
}

#[test]
fn selection_handles_renders_empty_when_no_selection() {
    #[component]
    fn App() -> Element {
        let doc = DiagramDocument::default();
        let mode = use_signal(|| InteractionMode::Select);
        let doc_sig = use_signal(|| DiagramDocument::default());
        let origin = use_signal(|| (0.0, 0.0));
        selection_handles_overlay(&doc, mode, doc_sig, origin, to_screen)
    }

    let mut vdom = VirtualDom::new(App);
    vdom.rebuild_in_place();
    let html = dioxus_ssr::render(&vdom);
    assert_eq!(html, "");
}
