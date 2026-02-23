#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use dioxus::prelude::*;
use crate::ui::sidebar::Sidebar;
use crate::ui::canvas::Canvas;
use crate::ui::toolbar::Toolbar;
use crate::ui::properties::PropertiesPanel;
use crate::models::document::DiagramDocument;
use crate::history::History;

#[allow(non_snake_case)]
pub fn App() -> Element {
    use_context_provider(|| Signal::new(DiagramDocument::default()));
    use_context_provider(|| Signal::new(Option::<String>::None)); 
    use_context_provider(|| Signal::new(History::new()));

    rsx! {
        div {
            display: "flex",
            flex_direction: "column",
            height: "100vh",
            width: "100vw",
            font_family: "sans-serif",
            
            Toolbar {}
            
            div {
                display: "flex",
                flex: "1",
                overflow: "hidden",
                
                Sidebar {}
                Canvas {}
                PropertiesPanel {}
            }
        }
    }
}
