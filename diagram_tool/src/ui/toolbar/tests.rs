#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]
use crate::app::AppState;
use crate::ui::editor::ToolMode;
use crate::ui::toolbar::Toolbar;
use dioxus::prelude::*;

#[test]
fn given_app_state_when_rendering_toolbar_then_nodes_and_edges_counts_are_displayed() {
    #[component]
    fn App() -> Element {
        let mut state = AppState::provide();
        {
            let mut stats = state.toolbar_stats.write();
            stats.node_count = 5;
            stats.edge_count = 3;
            stats.revision = 42;
        }

        rsx! {
            Toolbar {}
        }
    }

    let mut vdom = VirtualDom::new(App);
    vdom.rebuild_in_place();

    let html = dioxus_ssr::render(&vdom);
    assert!(
        html.contains("5 nodes"),
        "Expected '5 nodes' in html: {}",
        html
    );
    assert!(
        html.contains("3 edges"),
        "Expected '3 edges' in html: {}",
        html
    );
    assert!(
        html.contains("Rev 42"),
        "Expected 'Rev 42' in html: {}",
        html
    );
}

#[test]
fn given_app_state_with_tool_mode_when_rendering_toolbar_then_tool_is_active() {
    #[component]
    fn App() -> Element {
        let mut state = AppState::provide();
        *state.tool_mode.write() = ToolMode::Pan;

        rsx! {
            Toolbar {}
        }
    }

    let mut vdom = VirtualDom::new(App);
    vdom.rebuild_in_place();

    let html = dioxus_ssr::render(&vdom);
    assert!(
        html.contains("data-testid=\"tool-pan\""),
        "Expected tool-pan test id in html"
    );
}
