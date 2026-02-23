use crate::ui::icons::{
    LayersIcon, MaximizeIcon, PlayIcon, RedoIcon, SaveIcon, SettingsIcon, UndoIcon, ZoomInIcon,
    ZoomOutIcon,
};
use dioxus::prelude::*;

#[component]
fn ToolbarButton(
    label: &'static str,
    disabled: bool,
    on_click: EventHandler<MouseEvent>,
    children: Element,
) -> Element {
    let disabled_classes = if disabled {
        "opacity-40 pointer-events-none"
    } else {
        ""
    };

    rsx! {
        button {
            class: "flex h-8 w-8 items-center justify-center rounded-md text-slate-500 transition-all duration-100 hover:bg-white hover:text-slate-900 {disabled_classes}",
            title: "{label}",
            disabled,
            onclick: move |evt| on_click.call(evt),
            {children}
        }
    }
}

#[component]
pub fn FlowToolbar(
    workflow_name: ReadSignal<String>,
    on_workflow_name_change: EventHandler<String>,
    node_count: ReadSignal<usize>,
    edge_count: ReadSignal<usize>,
    zoom_label: ReadSignal<String>,
    on_zoom_in: EventHandler<MouseEvent>,
    on_zoom_out: EventHandler<MouseEvent>,
    on_fit_view: EventHandler<MouseEvent>,
    on_layout: EventHandler<MouseEvent>,
    on_execute: EventHandler<MouseEvent>,
    on_undo: EventHandler<MouseEvent>,
    on_redo: EventHandler<MouseEvent>,
    on_save: EventHandler<MouseEvent>,
    on_settings: EventHandler<MouseEvent>,
    can_undo: ReadSignal<bool>,
    can_redo: ReadSignal<bool>,
) -> Element {
    rsx! {
        header { class: "flex h-14 items-center justify-between border-b border-slate-200 bg-white/95 px-4 backdrop-blur",
            div { class: "flex items-center gap-3",
                div { class: "flex items-center gap-2",
                    div { class: "flex h-8 w-8 items-center justify-center rounded-lg bg-blue-600/10",
                        PlayIcon { class: "ml-0.5 h-3.5 w-3.5 text-blue-600" }
                    }
                    input {
                        r#type: "text",
                        value: "{workflow_name.read()}",
                        class: "h-8 w-auto min-w-[140px] max-w-[280px] border-none bg-transparent text-[15px] font-semibold text-slate-900 outline-none",
                        spellcheck: false,
                        oninput: move |evt| on_workflow_name_change.call(evt.value())
                    }
                }
                div { class: "flex items-center gap-2 text-[11px] text-slate-500",
                    span { class: "rounded-full border border-blue-200 bg-blue-50 px-2 py-0.5 text-blue-700", "Canvas" }
                    span { class: "rounded-full border border-slate-200 bg-slate-50 px-2 py-0.5 font-mono", "{node_count.read()} nodes" }
                    span { class: "rounded-full border border-slate-200 bg-slate-50 px-2 py-0.5 font-mono", "{edge_count.read()} links" }
                }
            }

            div { class: "flex items-center gap-0.5 rounded-xl border border-slate-200 bg-slate-50 px-1 py-0.5",
                ToolbarButton {
                    label: "Zoom Out",
                    disabled: false,
                    on_click: move |evt| on_zoom_out.call(evt),
                    ZoomOutIcon { class: "h-4 w-4" }
                }
                span { class: "min-w-[3rem] text-center font-mono text-[11px] text-slate-600", "{zoom_label.read()}" }
                ToolbarButton {
                    label: "Zoom In",
                    disabled: false,
                    on_click: move |evt| on_zoom_in.call(evt),
                    ZoomInIcon { class: "h-4 w-4" }
                }
                div { class: "mx-1 h-4 w-px bg-slate-300" }
                ToolbarButton {
                    label: "Fit View",
                    disabled: false,
                    on_click: move |evt| on_fit_view.call(evt),
                    MaximizeIcon { class: "h-4 w-4" }
                }
                div { class: "mx-1 h-4 w-px bg-slate-300" }
                ToolbarButton {
                    label: "Auto Layout",
                    disabled: false,
                    on_click: move |evt| on_layout.call(evt),
                    LayersIcon { class: "h-4 w-4" }
                }
            }

            div { class: "flex items-center gap-1",
                ToolbarButton {
                    label: "Undo",
                    disabled: !*can_undo.read(),
                    on_click: move |evt| on_undo.call(evt),
                    UndoIcon { class: "h-4 w-4" }
                }
                ToolbarButton {
                    label: "Redo",
                    disabled: !*can_redo.read(),
                    on_click: move |evt| on_redo.call(evt),
                    RedoIcon { class: "h-4 w-4" }
                }
                div { class: "mx-1 h-5 w-px bg-slate-300" }
                ToolbarButton {
                    label: "Save Workflow",
                    disabled: false,
                    on_click: move |evt| on_save.call(evt),
                    SaveIcon { class: "h-4 w-4" }
                }
                ToolbarButton {
                    label: "Settings",
                    disabled: false,
                    on_click: move |evt| on_settings.call(evt),
                    SettingsIcon { class: "h-4 w-4" }
                }
                button {
                    class: "ml-1 flex h-8 items-center gap-1.5 rounded-lg bg-blue-600 px-3 text-[12px] font-semibold text-white transition-colors hover:bg-blue-700",
                    title: "Run this workflow",
                    onclick: move |evt| on_execute.call(evt),
                    PlayIcon { class: "h-3.5 w-3.5" }
                    "Execute"
                }
            }
        }
    }
}
