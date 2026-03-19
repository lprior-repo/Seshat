use base64::{engine::general_purpose, Engine as _};
use dioxus::prelude::*;

use crate::{
    app::DraggedIconPayload,
    icons::{IconMeta, ICONS},
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
fn IconImage(src: Option<String>) -> Element {
    rsx! {
        if let Some(src_str) = src {
            img {
                src: "{src_str}",
                class: "w-8 h-8 object-contain pointer-events-none",
                draggable: "false"
            }
        } else {
            div {
                class: "w-8 h-8 rounded bg-muted"
            }
        }
    }
}

#[component]
fn IconLabel(text: String) -> Element {
    rsx! {
        span {
            class: "text-[10px] text-muted-foreground text-center leading-tight line-clamp-2 break-words",
            "{text}"
        }
    }
}

fn handle_drag(
    icon: &IconMeta,
    data_url: &Option<String>,
    dragging_icon: &mut Signal<Option<DraggedIconPayload>>,
) {
    dragging_icon.set(Some(DraggedIconPayload {
        icon_key: icon.icon_key.clone(),
        label: Some(icon.display_name.clone()),
        image_data_url: data_url.clone(),
    }));
}

#[component]
pub fn IconTile(
    icon: ReadSignal<IconMeta>,
    mut dragging_icon: Signal<Option<DraggedIconPayload>>,
) -> Element {
    let current_icon = icon();
    let data_url_memo = use_memo(move || icon_data_url(&icon()));

    let category_for_title = if current_icon.category_path.is_empty() {
        String::from("General")
    } else {
        current_icon.category_path.join(" / ")
    };

    let title_display = current_icon.display_name.clone();
    let title_key = current_icon.icon_key.clone();
    let current_icon_mousedown = current_icon.clone();
    let current_icon_dragstart = current_icon.clone();

    rsx! {
        button {
            class: "icon-item flex flex-col justify-center items-center gap-1.5 p-2 rounded-md border border-border bg-gradient-to-b from-background to-card cursor-grab shadow-inner hover:bg-muted/50",
            "data-testid": "icon-item",
            title: "{title_display}\n{title_key}\n{category_for_title}",
            draggable: "true",
            onmousedown: move |_| handle_drag(&current_icon_mousedown, &data_url_memo.read(), &mut dragging_icon),
            ondragstart: move |_| handle_drag(&current_icon_dragstart, &data_url_memo.read(), &mut dragging_icon),
            IconImage { src: data_url_memo.read().clone() }
            IconLabel { text: current_icon.display_name.clone() }
        }
    }
}
