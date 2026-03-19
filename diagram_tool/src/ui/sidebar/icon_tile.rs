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
fn IconImage(src: Option<String>) -> Element {
    rsx! {
        if let Some(src_str) = src {
            img {
                src: "{src_str}",
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

#[component]
fn IconLabel(text: String) -> Element {
    rsx! {
        span {
            style: "font-size: 10px; color: color-mix(in oklch, white 60%, transparent); text-align: center; line-height: 1.1; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; text-overflow: ellipsis; word-break: break-word;",
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
            class: "icon-item",
            "data-testid": "icon-item",
            title: "{title_display}\n{title_key}\n{category_for_title}",
            draggable: "true",
            onmousedown: move |_| handle_drag(&current_icon_mousedown, &data_url_memo.read(), &mut dragging_icon),
            ondragstart: move |_| handle_drag(&current_icon_dragstart, &data_url_memo.read(), &mut dragging_icon),
            style: "cursor: grab; border: 1px solid {BORDER}; border-radius: 6px; padding: 8px 4px; display: flex; flex-direction: column; justify-content: center; align-items: center; gap: 6px; background: linear-gradient(180deg, {BG_BASE} 0%, {BG_ELEVATED} 100%); box-shadow: inset 0 0 0 1px color-mix(in oklch, {BORDER} 60%, transparent);",
            IconImage { src: data_url_memo.read().clone() }
            IconLabel { text: current_icon.display_name.clone() }
        }
    }
}
