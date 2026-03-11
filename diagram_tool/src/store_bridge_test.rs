use std::sync::Arc;
use dioxus::prelude::*;
use crate::store_bridge::StoreBridge;

pub fn TestApp() -> Element {
    let bridge = use_context::<Arc<StoreBridge>>();
    rsx! { div { "test" } }
}
