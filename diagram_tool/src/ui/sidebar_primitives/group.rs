use dioxus::prelude::*;

#[component]
pub fn SidebarHeader(
    title: String,
    action_label: String,
    onaction: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        div {
            class: "flex items-center justify-between gap-2 px-1 mb-2",
            h3 {
                class: "m-0 text-base font-semibold text-foreground tracking-wide",
                "{title}"
            }
            button {
                class: "bg-transparent text-muted-foreground hover:text-foreground border-none cursor-pointer text-xs",
                onclick: move |evt| onaction.call(evt),
                "{action_label}"
            }
        }
    }
}

#[component]
pub fn SidebarGroup(
    provider: String,
    expanded: bool,
    query_active: bool,
    total_count: usize,
    ontoggle: EventHandler<MouseEvent>,
    children: Element,
) -> Element {
    rsx! {
        section {
            key: "{provider}",
            class: "flex flex-col mb-1",

            button {
                class: "w-full bg-transparent border-none py-1.5 px-2 text-foreground flex justify-between items-center cursor-pointer text-[13px] hover:bg-[var(--bg-elevated)] rounded-md transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-[var(--accent)] focus-visible:outline-offset-2",
                "aria-expanded": "{expanded}",
                "aria-label": "{provider} provider, {total_count} icons",
                onclick: move |evt| ontoggle.call(evt),

                div {
                    class: "flex items-center gap-2",
                    crate::ui::icons::Icon {
                        kind: if expanded { crate::ui::icons::IconKind::ChevronDown } else { crate::ui::icons::IconKind::ChevronRight },
                        size: 14,
                        color: Some("currentColor")
                    }
                    crate::ui::icons::Icon {
                        kind: crate::ui::icons::IconKind::Cloud,
                        size: 16,
                        color: Some(crate::ui::theme::ACCENT) // We'll make it teal to match active, or let the caller pass an icon.
                    }
                    span {
                        class: "font-medium capitalize",
                        if query_active {
                            "{provider}"
                        } else {
                            "{provider}"
                        }
                    }
                }
                span { class: "text-muted-foreground text-[11px]", "{total_count}" }
            }

            if expanded {
                div {
                    class: "pl-2 flex flex-col gap-0.5 mt-0.5",
                    {children}
                }
            }
        }
    }
}
