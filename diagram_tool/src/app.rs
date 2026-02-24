#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::history::History;
use crate::hooks::keyboard::use_global_keyboard;
use crate::models::document::DiagramDocument;
use crate::models::validation::validate_document;
use crate::ui::canvas::Canvas;
use crate::ui::properties::PropertiesPanel;
use crate::ui::sidebar::Sidebar;
use crate::ui::toolbar::Toolbar;
use crate::ui::IconNav;
use crate::ui::ValidationPanel;
use dioxus::prelude::*;

#[allow(non_snake_case)]
pub fn App() -> Element {
    use_context_provider(|| Signal::new(DiagramDocument::default()));
    let dragging_icon = use_context_provider(|| Signal::new(Option::<String>::None));
    use_context_provider(|| Signal::new(History::new()));
    // Shared counter that the Validate button can increment to force re-validation.
    use_context_provider(|| Signal::new(0_u64));

    use_global_keyboard();

    let doc_signal = use_context::<Signal<DiagramDocument>>();
    let validate_trigger = use_context::<Signal<u64>>();

    let selected_icon_key = use_signal(|| Option::<String>::None);

    let validation_issues = use_memo(move || {
        // Reading validate_trigger subscribes this memo to forced re-validation.
        let _t = *validate_trigger.read();
        validate_document(&*doc_signal.read())
    });

    rsx! {
        div {
            display: "flex",
            flex_direction: "column",
            height: "100vh",
            width: "100vw",
            font_family: "sans-serif",
            margin: "0",
            padding: "0",
            overflow: "hidden",

            Toolbar {}

            div {
                display: "flex",
                flex: "1",
                overflow: "hidden",

                Sidebar {}
                Canvas {}
                PropertiesPanel {}
                IconNav { selected_icon_key, dragging_icon }
            }

            ValidationPanel { issues: validation_issues }
        }
    }
}
