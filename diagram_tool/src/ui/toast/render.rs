use crate::app::AppState;
use crate::ui::toast::{ToastId, ToastIntent};
use dioxus::prelude::*;
use std::collections::HashSet;
const DISMISS_REMOVE_DELAY_MS: u64 = 1_000_000;
const CONFLICT_TOAST_DISMISS_MS: u64 = 3_000;
#[component]
pub fn Toaster() -> Element {
    let app_state = use_context::<AppState>();
    let mut toasts = app_state.toasts;
    let mut ai_conflict_state = app_state.ai_conflict;
    let items = toasts.read().items().to_vec();
    let mut pending_remove: Signal<HashSet<ToastId>> = use_signal(HashSet::new);
    let mut pending_dismiss: Signal<HashSet<ToastId>> = use_signal(HashSet::new);
    let effect_items_dismiss = items.clone();
    use_effect(move || {
        let to_dismiss: Vec<ToastId> = effect_items_dismiss
            .iter()
            .filter_map(|item| {
                let is_conflict = matches!(item.intent, ToastIntent::Warning | ToastIntent::Error);
                let not_yet_dismissed = !item.dismissed;
                let not_scheduled = !pending_dismiss.read().contains(&item.id);
                if is_conflict && not_yet_dismissed && not_scheduled {
                    Some(item.id)
                } else {
                    None
                }
            })
            .collect();
        for id in to_dismiss {
            let _ = pending_dismiss.write().insert(id);
            let mut toasts_signal = toasts;
            let mut pending_signal = pending_dismiss;
            let mut conflict_state_clone = ai_conflict_state;
            spawn(async move {
                gloo_timers::future::sleep(std::time::Duration::from_millis(
                    CONFLICT_TOAST_DISMISS_MS,
                ))
                .await;
                toasts_signal.with_mut(|queue| {
                    let _ = queue.dismiss(id);
                });
                conflict_state_clone.write().take();
                let _ = pending_signal.write().remove(&id);
            });
        }
    });
    let effect_items_remove = items.clone();
    use_effect(move || {
        let to_schedule: Vec<ToastId> = effect_items_remove
            .iter()
            .filter_map(|item| {
                if item.dismissed && !pending_remove.read().contains(&item.id) {
                    Some(item.id)
                } else {
                    None
                }
            })
            .collect();
        for id in to_schedule {
            let _ = pending_remove.write().insert(id);
            let mut toasts_signal = toasts;
            let mut pending_signal = pending_remove;
            spawn(async move {
                gloo_timers::future::sleep(std::time::Duration::from_millis(
                    DISMISS_REMOVE_DELAY_MS,
                ))
                .await;
                toasts_signal.with_mut(|queue| {
                    let _ = queue.remove(id);
                });
                let _ = pending_signal.write().remove(&id);
            });
        }
    });
    if items.is_empty() {
        return rsx! {};
    }
    rsx! {
        div {
            class: "fixed right-[14px] top-[66px] z-[60] flex flex-col gap-2 w-[min(380px,calc(100vw-24px))] pointer-events-none",
            for toast in items {
                {
                    let id = toast.id;
                    let stripe = toast.intent.stripe_color();
                    let card_shadow = "0 10px 24px color-mix(in oklch, black 32%, transparent)";
                    let card_opacity = if toast.dismissed { "opacity-0" } else { "opacity-100" };
                    let card_transform = if toast.dismissed { "-translate-y-1.5 scale-98" } else { "translate-y-0 scale-100" };
                    let card_pointer_events = if toast.dismissed { "pointer-events-none" } else { "pointer-events-auto" };
                    rsx! {
                        article {
                            key: "{id:?}",
                            class: "relative overflow-hidden border border-border rounded-[10px] bg-gradient-to-b from-[var(--bg-elevated)] to-[var(--bg-surface)] text-foreground shadow-lg transition-all duration-180 ease-out {card_opacity} {card_transform} {card_pointer_events}",
                            style: "box-shadow: {card_shadow};",
                            div {
                                class: "absolute left-0 top-0 bottom-0 w-1",
                                style: "background: {stripe};"
                            }
                            div {
                                class: "py-2 pr-2.5 pl-3 flex gap-2.5 items-start",
                                div {
                                    class: "flex-1 min-w-0",
                                    p {
                                        class: "m-0 text-xs font-bold text-foreground",
                                        "{toast.title}"
                                    }
                                    if let Some(detail) = toast.detail {
                                        p {
                                            class: "mt-0.5 mb-0 text-[11px] text-muted-foreground whitespace-pre-wrap",
                                            "{detail}"
                                        }
                                    }
                                }
                                if let Some(action) = toast.action {
                                    button {
                                        class: "shrink-0 rounded-md border border-border bg-surface text-foreground text-[11px] leading-none cursor-pointer px-2 h-[22px]",
                                        onclick: move |_| {
                                            toasts.with_mut(|queue| {
                                                let target = if action.dismiss_all { None } else { Some(id) };
                                                let _ = queue.dismiss_target(target);
                                            });
                                        },
                                        "{action.label}"
                                    }
                                }
                                button {
                                    class: "shrink-0 w-[22px] h-[22px] rounded-md border border-border bg-surface text-muted-foreground text-xs leading-none cursor-pointer flex items-center justify-center",
                                    onclick: move |_| {
                                        toasts.with_mut(|queue| {
                                            let _ = queue.dismiss_target(Some(id));
                                        });
                                        ai_conflict_state.set(None);
                                    },
                                    "x"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
