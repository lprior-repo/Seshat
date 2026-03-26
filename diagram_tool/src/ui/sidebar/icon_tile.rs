use dioxus::prelude::*;

use crate::{app::DraggedIconPayload, icons::IconMeta};

#[component]
fn IconImage(src: Option<String>) -> Element {
    rsx! {
        if let Some(src_str) = src {
            img {
                src: "{src_str}",
                class: "w-8 h-8 object-contain pointer-events-none",
                draggable: "false",
                loading: "lazy",
                decoding: "async"
            }
        } else {
            div {
                class: "w-8 h-8 rounded bg-muted",
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

fn handle_drag(icon: &IconMeta, dragging_icon: &mut Signal<Option<DraggedIconPayload>>) {
    dragging_icon.set(Some(DraggedIconPayload {
        icon_key: icon.icon_key.to_string(),
        label: Some(icon.display_name.to_string()),
        image_url: crate::icons::icon_src(icon),
    }));
}

#[component]
pub fn IconTile(
    icon: ReadSignal<IconMeta>,
    mut dragging_icon: Signal<Option<DraggedIconPayload>>,
) -> Element {
    let current_icon = icon();

    let category_for_title = if current_icon.category_path.is_empty() {
        String::from("General")
    } else {
        current_icon.category_path.join(" / ")
    };

    let title_display = current_icon.display_name.to_string();
    let title_key = current_icon.icon_key.to_string();
    let current_icon_mousedown = current_icon.clone();
    let current_icon_dragstart = current_icon.clone();

    rsx! {
        button {
            class: "flex flex-col justify-center items-center gap-1.5 p-2 rounded-md border border-border bg-surface cursor-grab w-full box-border",
            "data-testid": "icon-item",
            title: "{title_display}\n{title_key}\n{category_for_title}",
            draggable: "true",
            onmousedown: move |_| handle_drag(&current_icon_mousedown, &mut dragging_icon),
            ondragstart: move |evt| {
                let dt = evt.data().data_transfer();
                let _ = dt.set_data("text/plain", &current_icon_dragstart.icon_key);
                dt.set_effect_allowed("copy");
                handle_drag(&current_icon_dragstart, &mut dragging_icon);
            },
            IconImage { src: crate::icons::icon_src(&current_icon) }
            IconLabel { text: current_icon.display_name.to_string() }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use dioxus::prelude::*;

    #[test]
    fn test_icon_tile_rendering() {
        let mut vdom = VirtualDom::new(|| {
            let icon = use_signal(|| IconMeta {
                icon_key: std::sync::Arc::from("aws/analytics/athena"),
                provider: std::sync::Arc::from("aws"),
                category_path: vec![std::sync::Arc::from("Analytics")],
                file_relpath: std::sync::Arc::from("aws/Analytics/athena.svg"),
                display_name: std::sync::Arc::from("Athena"),
                search_terms: std::sync::Arc::from("athena aws analytics"),
            });
            let dragging_icon = use_signal(|| None);
            rsx! {
                IconTile { icon, dragging_icon }
            }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Athena"));
        assert!(html.contains("/assets/resources/aws/Analytics/athena.svg"));
        assert!(html.contains("data-testid=\"icon-item\""));
    }
}
