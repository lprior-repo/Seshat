#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![allow(clippy::cast_precision_loss)]
#![forbid(unsafe_code)]

pub mod canvas_view;
pub mod document_ops;
pub mod domain;
pub mod edge_layer;
pub mod edge_types;
pub mod grid_layer;
pub mod node_layer;
pub mod root_container;
pub mod root_handlers;
pub mod state;
pub mod toolbar;

pub use canvas_view::{
    touch_handle_hit_test, touch_hit_radius, RESIZE_HANDLE_SIZE_PX, TOUCH_HIT_RADIUS_PX,
};

use dioxus::prelude::*;

#[component]
pub fn Canvas() -> Element {
    let state = state::use_canvas_state();
    rsx! {
        root_container::RootContainer { state: state }
    }
}
