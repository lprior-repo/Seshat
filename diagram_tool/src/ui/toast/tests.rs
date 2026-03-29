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
use crate::ui::toast::{ToastApi, ToastIntent, ToastOptions, Toaster};
use dioxus::prelude::*;

#[test]
fn given_toast_in_state_when_rendering_toaster_then_toast_is_displayed() {
    #[component]
    fn App() -> Element {
        let state = AppState::provide();

        let toast_api = ToastApi::from_signal(state.toasts);
        let _ = toast_api.toast(
            ToastOptions::new(ToastIntent::Info, "Test Toast Title")
                .with_detail("Test Toast Detail"),
        );

        rsx! {
            Toaster {}
        }
    }

    let mut vdom = VirtualDom::new(App);
    vdom.rebuild_in_place();

    let html = dioxus_ssr::render(&vdom);
    assert!(
        html.contains("Test Toast Title"),
        "Expected title in html: {html}"
    );
    assert!(
        html.contains("Test Toast Detail"),
        "Expected detail in html: {html}"
    );
}
