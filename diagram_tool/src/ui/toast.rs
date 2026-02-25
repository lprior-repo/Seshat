#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::ui::theme::{
    BG_ELEVATED, BG_SURFACE, BORDER, ERROR, SUCCESS, TEXT_MAIN, TEXT_MUTED, WARNING,
};
use dioxus::prelude::*;
use std::collections::HashSet;

const MAX_TOASTS: usize = 1;
const DISMISS_REMOVE_DELAY_MS: u64 = 1_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ToastId(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastIntent {
    Info,
    Success,
    #[allow(dead_code)]
    Warning,
    Error,
}

impl ToastIntent {
    #[must_use]
    const fn stripe_color(self) -> &'static str {
        match self {
            Self::Info => "var(--accent)",
            Self::Success => SUCCESS,
            Self::Warning => WARNING,
            Self::Error => ERROR,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Toast {
    pub id: ToastId,
    pub intent: ToastIntent,
    pub title: String,
    pub detail: Option<String>,
    pub action: Option<ToastAction>,
    pub dismissed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToastAction {
    pub label: String,
    pub dismiss_all: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[allow(clippy::option_option)]
pub struct ToastUpdate {
    pub title: Option<String>,
    pub detail: Option<Option<String>>,
    pub intent: Option<ToastIntent>,
    pub action: Option<Option<ToastAction>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ToastQueue {
    next_id: u64,
    items: Vec<Toast>,
}

#[derive(Clone, Copy)]
pub struct ToastApi {
    queue: Signal<ToastQueue>,
}

impl ToastApi {
    #[must_use]
    pub fn show(
        mut self,
        intent: ToastIntent,
        title: impl Into<String>,
        detail: Option<String>,
    ) -> ToastId {
        let mut result = ToastId(0);
        self.queue.with_mut(|queue| {
            result = queue.add(intent, title, detail);
        });
        result
    }

    #[must_use]
    pub fn error(self, title: impl Into<String>, detail: Option<String>) -> ToastId {
        self.show(ToastIntent::Error, title, detail)
    }
}

#[must_use]
pub fn use_toast() -> ToastApi {
    ToastApi {
        queue: use_context::<Signal<ToastQueue>>(),
    }
}

impl ToastQueue {
    #[must_use]
    pub fn add(
        &mut self,
        intent: ToastIntent,
        title: impl Into<String>,
        detail: Option<String>,
    ) -> ToastId {
        self.add_with_action(intent, title, detail, None)
    }

    #[must_use]
    pub fn add_with_action(
        &mut self,
        intent: ToastIntent,
        title: impl Into<String>,
        detail: Option<String>,
        action: Option<ToastAction>,
    ) -> ToastId {
        let id = ToastId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        self.items.push(Toast {
            id,
            intent,
            title: title.into(),
            detail,
            action,
            dismissed: false,
        });
        while self.items.len() > MAX_TOASTS {
            if let Some(idx) = self.items.iter().position(|item| item.dismissed) {
                let _ = self.items.remove(idx);
            } else {
                let _ = self.items.remove(0);
            }
        }
        id
    }

    pub fn update(&mut self, id: ToastId, patch: ToastUpdate) -> bool {
        if let Some(item) = self.items.iter_mut().find(|item| item.id == id) {
            if let Some(title) = patch.title {
                item.title = title;
            }
            if let Some(detail) = patch.detail {
                item.detail = detail;
            }
            if let Some(intent) = patch.intent {
                item.intent = intent;
            }
            if let Some(action) = patch.action {
                item.action = action;
            }
            true
        } else {
            false
        }
    }

    pub fn dismiss(&mut self, id: ToastId) -> bool {
        if let Some(item) = self.items.iter_mut().find(|item| item.id == id) {
            item.dismissed = true;
            true
        } else {
            false
        }
    }

    pub fn dismiss_target(&mut self, id: Option<ToastId>) -> bool {
        if let Some(toast_id) = id {
            self.dismiss(toast_id)
        } else {
            self.dismiss_all()
        }
    }

    pub fn dismiss_all(&mut self) -> bool {
        let mut changed = false;
        for item in &mut self.items {
            if !item.dismissed {
                item.dismissed = true;
                changed = true;
            }
        }
        changed
    }

    pub fn remove(&mut self, id: ToastId) -> bool {
        if let Some(idx) = self.items.iter().position(|item| item.id == id) {
            let _ = self.items.remove(idx);
            true
        } else {
            false
        }
    }

    #[must_use]
    pub fn items(&self) -> &[Toast] {
        &self.items
    }
}

#[component]
pub fn Toaster() -> Element {
    let mut toasts = use_context::<Signal<ToastQueue>>();
    let items = toasts.read().items().to_vec();
    let mut pending_remove: Signal<HashSet<ToastId>> = use_signal(HashSet::new);
    let effect_items = items.clone();

    use_effect(move || {
        let to_schedule: Vec<ToastId> = effect_items
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
            let mut eval = document::eval(&format!(
                "setTimeout(() => dioxus.send({{ kind: 'remove-toast', id: {} }}), {});",
                id.0, DISMISS_REMOVE_DELAY_MS
            ));
            spawn(async move {
                if eval.recv::<serde_json::Value>().await.is_ok() {
                    toasts_signal.with_mut(|queue| {
                        let _ = queue.remove(id);
                    });
                    let _ = pending_signal.write().remove(&id);
                }
            });
        }
    });

    if items.is_empty() {
        return rsx! {};
    }

    rsx! {
        div {
            style: "position: fixed; right: 14px; top: 66px; z-index: 60; display: flex; flex-direction: column; gap: 8px; width: min(380px, calc(100vw - 24px)); pointer-events: none;",
            for toast in items {
                {
                    let id = toast.id;
                    let stripe = toast.intent.stripe_color();
                    let card_shadow = "0 10px 24px color-mix(in oklch, black 32%, transparent)";
                    let card_opacity = if toast.dismissed { "0" } else { "1" };
                    let card_transform = if toast.dismissed {
                        "translateY(-6px) scale(0.98)"
                    } else {
                        "translateY(0px) scale(1)"
                    };
                    let card_pointer_events = if toast.dismissed { "none" } else { "auto" };

                    rsx! {
                        article {
                            key: "{id:?}",
                            style: "pointer-events: {card_pointer_events}; position: relative; overflow: hidden; border: 1px solid {BORDER}; border-radius: 10px; background: linear-gradient(180deg, {BG_ELEVATED} 0%, {BG_SURFACE} 100%); color: {TEXT_MAIN}; box-shadow: {card_shadow}; transition: opacity 180ms ease, transform 180ms ease; opacity: {card_opacity}; transform: {card_transform};",

                            div {
                                style: "position: absolute; left: 0; top: 0; bottom: 0; width: 4px; background: {stripe};"
                            }

                            div {
                                style: "padding: 8px 10px 8px 12px; display: flex; gap: 10px; align-items: flex-start;",
                                div {
                                    style: "flex: 1; min-width: 0;",
                                    p {
                                        style: "margin: 0; font-size: 12px; font-weight: 700; color: {TEXT_MAIN};",
                                        "{toast.title}"
                                    }
                                    if let Some(detail) = toast.detail {
                                        p {
                                            style: "margin: 2px 0 0; font-size: 11px; color: {TEXT_MUTED}; white-space: pre-wrap;",
                                            "{detail}"
                                        }
                                    }
                                }
                                if let Some(action) = toast.action {
                                    button {
                                        style: "flex-shrink: 0; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_SURFACE}; color: {TEXT_MAIN}; font-size: 11px; line-height: 1; cursor: pointer; padding: 0 8px; height: 22px;",
                                        onclick: move |_| {
                                            toasts.with_mut(|queue| {
                                                let target = if action.dismiss_all {
                                                    None
                                                } else {
                                                    Some(id)
                                                };
                                                let _ = queue.dismiss_target(target);
                                            });
                                        },
                                        "{action.label}"
                                    }
                                }
                                button {
                                    style: "flex-shrink: 0; width: 22px; height: 22px; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_SURFACE}; color: {TEXT_MUTED}; font-size: 12px; line-height: 1; cursor: pointer;",
                                    onclick: move |_| {
                                        toasts.with_mut(|queue| {
                                            let _ = queue.dismiss_target(Some(id));
                                        });
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
