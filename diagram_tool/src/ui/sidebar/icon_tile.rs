use base64::{engine::general_purpose, Engine as _};
use dioxus::prelude::*;

use crate::{
    app::DraggedIconPayload,
    icons::{IconMeta, ICONS},
    ui::theme::{BG_BASE, BG_ELEVATED, BORDER},
};

pub fn icon_data_url(icon: &IconMeta) -> Option<String> {
    let file = ICONS.get_file(&icon.file_relpath)?;
    let mime = std::path::Path::new(&icon.file_relpath)
        .extension()
        .and_then(|ext| ext.to_str())
        .map_or("image/png", |ext| {
            if ext.eq_ignore_ascii_case("svg") {
                "image/svg+xml"
            } else {
                "image/png"
            }
        });

    Some(format!(
        "data:{mime};base64,{}",
        general_purpose::STANDARD.encode(file.contents())
    ))
}

#[component]
pub fn IconTile(icon: IconMeta, dragging_icon: Signal<Option<DraggedIconPayload>>) -> Element {
    let data_url = icon_data_url(&icon);
    let data_url_for_drag = data_url.clone();
    let data_url_for_drag_start = data_url.clone();
    let icon_key_for_drag = icon.icon_key.clone();
    let icon_key_for_title = icon.icon_key.clone();
    let category_for_title = if icon.category_path.is_empty() {
        String::from("General")
    } else {
        icon.category_path.join(" / ")
    };
    let icon_label_for_drag = icon.display_name.clone();

    rsx! {
        button {
            class: "icon-item",
            "data-testid": "icon-item",
            title: "{icon.display_name}\n{icon_key_for_title}\n{category_for_title}",
            draggable: "true",
            onmousedown: move |_| {
                dragging_icon.set(Some(DraggedIconPayload {
                    icon_key: icon_key_for_drag.clone(),
                    label: Some(icon_label_for_drag.clone()),
                    image_data_url: data_url_for_drag.clone(),
                }));
            },
            ondragstart: move |_| {
                // In Dioxus, we just need the drag to start successfully
                dragging_icon.set(Some(DraggedIconPayload {
                    icon_key: icon.icon_key.clone(),
                    label: Some(icon.display_name.clone()),
                    image_data_url: data_url_for_drag_start.clone(),
                }));
            },
            style: "cursor: grab; border: 1px solid {BORDER}; border-radius: 6px; padding: 5px; display: flex; justify-content: center; align-items: center; background: linear-gradient(180deg, {BG_BASE} 0%, {BG_ELEVATED} 100%); aspect-ratio: 1/1; box-shadow: inset 0 0 0 1px color-mix(in oklch, {BORDER} 60%, transparent);",

            if let Some(src) = data_url {
                img {
                    src: "{src}",
                    width: "32px",
                    height: "32px",
                    style: "object-fit: contain; pointer-events: none;",
                    draggable: "false"
                }
            } else {
                div {
                    style: "width: 32px; height: 32px; border-radius: 4px; background: #1f2937;"
                }
            }
        }
    }
}
