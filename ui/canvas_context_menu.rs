use dioxus::prelude::*;

#[component]
pub fn CanvasContextMenu(
    open: ReadSignal<bool>,
    x: ReadSignal<f32>,
    y: ReadSignal<f32>,
    on_close: EventHandler<MouseEvent>,
    on_add_node: EventHandler<MouseEvent>,
    on_fit_view: EventHandler<MouseEvent>,
    on_layout: EventHandler<MouseEvent>,
) -> Element {
    if !open() {
        return rsx! {};
    }

    let menu_style = format!("left: {}px; top: {}px;", x(), y());

    rsx! {
        div {
            class: "fixed inset-0 z-50",

            button {
                r#type: "button",
                class: "absolute inset-0 h-full w-full cursor-default bg-transparent",
                aria_label: "Close context menu",
                onclick: move |evt| on_close.call(evt),
            }

            div {
                class: "absolute w-56 overflow-hidden rounded-lg border border-slate-700/80 bg-slate-900/95 shadow-2xl shadow-slate-950/70 ring-1 ring-slate-700/70 backdrop-blur",
                style: "{menu_style}",

                button {
                    r#type: "button",
                    class: "block w-full px-3 py-2 text-left text-sm font-medium text-slate-200 transition-colors hover:bg-slate-800/90 hover:text-slate-50",
                    onclick: move |evt| on_add_node.call(evt),
                    "Add Node"
                }

                button {
                    r#type: "button",
                    class: "block w-full px-3 py-2 text-left text-sm font-medium text-slate-200 transition-colors hover:bg-slate-800/90 hover:text-slate-50",
                    onclick: move |evt| on_fit_view.call(evt),
                    "Fit View"
                }

                button {
                    r#type: "button",
                    class: "block w-full px-3 py-2 text-left text-sm font-medium text-slate-200 transition-colors hover:bg-slate-800/90 hover:text-slate-50",
                    onclick: move |evt| on_layout.call(evt),
                    "Auto Layout"
                }

                div {
                    class: "border-t border-slate-700 px-3 py-2 text-xs text-slate-400",
                    "Hint: Press Esc or click outside to close"
                }
            }
        }
    }
}
