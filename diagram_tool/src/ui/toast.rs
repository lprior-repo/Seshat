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
const CONFLICT_TOAST_DISMISS_MS: u64 = 3_000;

/// AI conflict state representation for toast display
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiConflictState {
    /// The reason for the conflict rejection
    pub reason: Option<String>,
    /// Entities that were in conflict
    pub conflicting_entities: Vec<String>,
}

impl AiConflictState {
    #[must_use]
    pub const fn new(reason: Option<String>, conflicting_entities: Vec<String>) -> Self {
        Self {
            reason,
            conflicting_entities,
        }
    }

    #[must_use]
    pub fn has_valid_reason(&self) -> bool {
        self.reason
            .as_ref()
            .is_some_and(|r| !r.trim().is_empty())
    }
}

/// Error types for conflict toast operations - matches contract specification
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// No conflict state provided (P1 violation)
    NoConflictState,
    /// Toast queue is at capacity (P2 violation)
    QueueFull,
    /// Conflict reason is empty or missing (P3 violation)
    InvalidReason,
    /// The JavaScript setTimeout call failed or returned an error
    JsTimeoutFailure,
    /// The required Dioxus signal is not available in context
    SignalNotFound,
    /// Attempted to dismiss a toast that no longer exists in the queue
    ToastNotFound,
    /// The auto-dismiss timer was cancelled (e.g., manual dismiss before 3s)
    TimerCancelled,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoConflictState => write!(f, "No conflict state provided"),
            Self::QueueFull => write!(f, "Toast queue is at capacity"),
            Self::InvalidReason => write!(f, "Conflict reason is empty or missing"),
            Self::JsTimeoutFailure => write!(f, "JavaScript setTimeout call failed"),
            Self::SignalNotFound => write!(f, "Required Dioxus signal not available in context"),
            Self::ToastNotFound => write!(f, "Toast no longer exists in the queue"),
            Self::TimerCancelled => write!(f, "Auto-dismiss timer was cancelled"),
        }
    }
}

impl std::error::Error for Error {}

/// Validates that conflict state has meaningful content
fn validate_conflict_state(state: &AiConflictState) -> Result<(), Error> {
    let has_reason = state
        .reason
        .as_ref()
        .is_some_and(|r| !r.trim().is_empty());
    let has_entities = !state.conflicting_entities.is_empty();
    if !has_reason && !has_entities {
        Err(Error::NoConflictState)
    } else {
        Ok(())
    }
}

/// Extracts the reason text from conflict state, using fallback if empty
fn extract_reason_text(state: &AiConflictState) -> String {
    state
        .reason
        .as_ref()
        .filter(|r| !r.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| "Edit conflict".to_string())
}

/// Builds the detail text for the toast from conflict state
fn build_conflict_detail(reason_text: &str, entities: &[String]) -> Option<String> {
    if entities.is_empty() {
        Some(reason_text.to_string())
    } else {
        let entity_list = entities.join(", ");
        Some(format!("{reason_text}: {entity_list}"))
    }
}

/// Validates that the created toast has a valid ID
fn validate_toast_id(handle: &ToastHandle) -> Result<(), Error> {
    if handle.id().0 == 0 {
        Err(Error::QueueFull)
    } else {
        Ok(())
    }
}

/// Creates toast options for a conflict notification
fn create_conflict_toast_options(state: &AiConflictState) -> ToastOptions {
    let reason = extract_reason_text(state);
    let detail = build_conflict_detail(&reason, &state.conflicting_entities);
    ToastOptions::new(ToastIntent::Warning, "Edit Conflict").with_optional_detail(detail)
}

/// Clears the ai_conflict_state signal by setting it to None
pub fn clear_ai_conflict_state(state: &mut Signal<Option<AiConflictState>>) {
    state.set(None);
}

/// Display toast for AI conflict state
/// Returns: Result<ToastHandle, Error>
pub fn show_conflict_toast(
    conflict_state: &AiConflictState,
    toast_api: ToastApi,
) -> Result<ToastHandle, Error> {
    // P1: Must have valid conflict state
    validate_conflict_state(conflict_state)?;

    // Create toast options and display
    let options = create_conflict_toast_options(conflict_state);
    let handle = toast_api.toast(options);

    // Q1: Verify toast has valid non-zero ID
    validate_toast_id(&handle)?;

    Ok(handle)
}

/// Check if toast should be displayed for conflict
/// Returns: Result<bool, Error>
pub fn should_show_conflict_toast(
    conflict_state: Option<&AiConflictState>,
) -> Result<bool, Error> {
    match conflict_state {
        Some(state) => {
            // P1: Must have valid conflict state
            // P3: Rejection reason must be present
            Ok(state.has_valid_reason() || !state.conflicting_entities.is_empty())
        }
        None => Ok(false),
    }
}

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

#[derive(Clone, Copy)]
pub struct ToastHandle {
    id: ToastId,
    api: ToastApi,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToastOptions {
    pub intent: ToastIntent,
    pub title: String,
    pub detail: Option<String>,
    pub action: Option<ToastAction>,
}

impl ToastOptions {
    #[must_use]
    pub fn new(intent: ToastIntent, title: impl Into<String>) -> Self {
        Self {
            intent,
            title: title.into(),
            detail: None,
            action: None,
        }
    }

    #[must_use]
    pub fn with_detail(self, detail: impl Into<String>) -> Self {
        Self {
            detail: Some(detail.into()),
            ..self
        }
    }

    #[must_use]
    pub fn with_optional_detail(self, detail: Option<String>) -> Self {
        Self { detail, ..self }
    }

    #[must_use]
    #[allow(dead_code)]
    pub fn with_action(self, action: ToastAction) -> Self {
        Self {
            action: Some(action),
            ..self
        }
    }
}

impl ToastHandle {
    #[must_use]
    pub const fn id(self) -> ToastId {
        self.id
    }

    #[must_use]
    pub fn update(self, patch: ToastUpdate) -> bool {
        self.api.update(self.id, patch)
    }

    #[must_use]
    pub fn dismiss(self) -> bool {
        self.api.dismiss(Some(self.id))
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn remove(self) -> bool {
        self.api.remove(self.id)
    }
}

impl ToastApi {
    #[must_use]
    pub const fn from_signal(queue: Signal<ToastQueue>) -> Self {
        Self { queue }
    }

    #[must_use]
    pub fn toast(self, options: ToastOptions) -> ToastHandle {
        let mut queue = self.queue;
        let mut id = ToastId(0);
        queue.with_mut(|state| {
            id = state.add_with_action(
                options.intent,
                options.title,
                options.detail,
                options.action,
            );
        });
        ToastHandle { id, api: self }
    }

    #[must_use]
    pub fn show(
        self,
        intent: ToastIntent,
        title: impl Into<String>,
        detail: Option<String>,
    ) -> ToastId {
        self.toast(ToastOptions::new(intent, title).with_optional_detail(detail))
            .id()
    }

    #[must_use]
    pub fn update(self, id: ToastId, patch: ToastUpdate) -> bool {
        let mut queue = self.queue;
        let mut changed = false;
        queue.with_mut(|state| {
            changed = state.update(id, patch);
        });
        changed
    }

    #[must_use]
    pub fn dismiss(self, id: Option<ToastId>) -> bool {
        let mut queue = self.queue;
        let mut changed = false;
        queue.with_mut(|state| {
            changed = state.dismiss_target(id);
        });
        changed
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn remove(self, id: ToastId) -> bool {
        let mut queue = self.queue;
        let mut changed = false;
        queue.with_mut(|state| {
            changed = state.remove(id);
        });
        changed
    }

    #[must_use]
    pub fn error(self, title: impl Into<String>, detail: Option<String>) -> ToastId {
        self.show(ToastIntent::Error, title, detail)
    }
}

#[must_use]
pub fn use_toast() -> ToastApi {
    ToastApi::from_signal(use_context::<Signal<ToastQueue>>())
}

#[must_use]
#[allow(dead_code)]
pub fn toast(options: ToastOptions) -> ToastHandle {
    use_toast().toast(options)
}

impl ToastQueue {
    #[must_use]
    #[allow(dead_code)]
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
    let mut ai_conflict_state: Signal<Option<AiConflictState>> = use_context();
    let items = toasts.read().items().to_vec();
    let mut pending_remove: Signal<HashSet<ToastId>> = use_signal(HashSet::new);
    let mut pending_dismiss: Signal<HashSet<ToastId>> = use_signal(HashSet::new);

    // Auto-dismiss effect for conflict toasts (Warning and Error intents)
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
            let mut conflict_state_clone = ai_conflict_state.clone();
            let mut eval = document::eval(&format!(
                "setTimeout(() => dioxus.send({{ kind: 'dismiss-conflict', id: {} }}), {});",
                id.0, CONFLICT_TOAST_DISMISS_MS
            ));
            spawn(async move {
                if eval.recv::<serde_json::Value>().await.is_ok() {
                    toasts_signal.with_mut(|queue| {
                        let _ = queue.dismiss(id);
                    });
                    // Clear conflict state after auto-dismiss
                    conflict_state_clone.write().take();
                    let _ = pending_signal.write().remove(&id);
                }
            });
        }
    });

    // Existing removal effect for dismissed toasts
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
                                        // Clear conflict state on manual dismiss
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
