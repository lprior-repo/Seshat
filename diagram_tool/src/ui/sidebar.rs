#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use dioxus::prelude::*;
use crate::icons::ICONS;
use base64::{Engine as _, engine::general_purpose};

#[component]
pub fn Sidebar() -> Element {
    let mut search = use_signal(String::new);
    let mut dragging_icon = use_context::<Signal<Option<String>>>();

    rsx! {
        div {
            class: "sidebar",
            style: "width: 250px; background: #f0f0f0; padding: 10px; display: flex; flex-direction: column; gap: 10px; overflow-y: auto;",

            input {
                placeholder: "Search icons...",
                style: "padding: 5px; width: 100%;",
                oninput: move |evt| search.set(evt.value())
            }
            div {
                class: "icon-grid",
                style: "display: grid; grid-template-columns: repeat(3, 1fr); gap: 5px;",
                
                {
                    ICONS.find("**/*.png").map_or_else(|_| Vec::new(), |iter| {
                        iter.filter_map(|entry| entry.as_file().zip(entry.path().to_str()))
                            .filter(|(_, path)| search.read().is_empty() || path.to_lowercase().contains(&search.read().to_lowercase()))
                            .map(|(file, path)| {
                                let src = format!("data:image/png;base64,{}", general_purpose::STANDARD.encode(file.contents()));
                                let drag_path = path.to_string();
                                
                                rsx! {
                                    div {
                                        key: "{path}",
                                        class: "icon-item",
                                        title: "{path}",
                                        draggable: "true",
                                        onmousedown: move |_| dragging_icon.set(Some(drag_path.clone())),
                                        onmouseup: move |_| dragging_icon.set(None),
                                        style: "cursor: grab; border: 1px solid #ccc; padding: 5px; display: flex; justify-content: center; align-items: center;",
                                        img {
                                            src: "{src}",
                                            width: "40px",
                                            height: "40px",
                                            draggable: "false"
                                        }
                                    }
                                }
                            }).collect::<Vec<_>>()
                    }).into_iter()
                }
            }
        }
    }
}
