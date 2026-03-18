#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]
pub mod data;
pub mod render;
pub mod state;
use crate::ui::theme::{ERROR, SUCCESS, WARNING};
pub use data::ToastQueue;
use dioxus::prelude::*;
pub use render::Toaster;
pub use state::{ToastApi, ToastHandle, ToastOptions};
pub const MAX_TOASTS: usize = 1;
pub const DISMISS_REMOVE_DELAY_MS: u64 = 1_000_000;
pub const CONFLICT_TOAST_DISMISS_MS: u64 = 3_000;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiConflictState {
    pub reason: Option<String>,
    pub conflicting_entities: Vec<String>,
}
impl AiConflictState {
    #[must_use]
    pub const fn new(r: Option<String>, c: Vec<String>) -> Self {
        Self {
            reason: r,
            conflicting_entities: c,
        }
    }
}
impl AiConflictState {
    #[must_use]
    pub fn has_valid_reason(&self) -> bool {
        self.reason.as_ref().is_some_and(|r| !r.trim().is_empty())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    NoConflictState,
    QueueFull,
    InvalidReason,
    JsTimeoutFailure,
    SignalNotFound,
    ToastNotFound,
    TimerCancelled,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoConflictState => write!(f, "No conflict state provided"),
            Self::QueueFull => write!(f, "Toast queue is at capacity"),
            Self::InvalidReason => write!(f, "Conflict reason is empty or missing"),
            Self::JsTimeoutFailure => write!(f, "JavaScript setTimeout call failed"),
            Self::SignalNotFound => write!(f, "Required Dioxus signal not available"),
            Self::ToastNotFound => write!(f, "Toast no longer exists in the queue"),
            Self::TimerCancelled => write!(f, "Auto-dismiss timer was cancelled"),
        }
    }
}
impl std::error::Error for Error {}
fn validate_conflict_state(s: &AiConflictState) -> Result<(), Error> {
    let hr = s.reason.as_ref().is_some_and(|r| !r.trim().is_empty());
    let he = !s.conflicting_entities.is_empty();
    if !hr && !he {
        Err(Error::NoConflictState)
    } else {
        Ok(())
    }
}
fn extract_reason_text(s: &AiConflictState) -> String {
    s.reason
        .as_ref()
        .filter(|r| !r.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| "Edit conflict".to_string())
}
fn build_conflict_detail(rt: &str, es: &[String]) -> Option<String> {
    if es.is_empty() {
        Some(rt.to_string())
    } else {
        Some(format!("{}: {}", rt, es.join(", ")))
    }
}
const fn validate_toast_id(h: &ToastHandle) -> Result<(), Error> {
    if h.id().0 == 0 {
        Err(Error::QueueFull)
    } else {
        Ok(())
    }
}
fn create_conflict_toast_options(s: &AiConflictState) -> ToastOptions {
    let r = extract_reason_text(s);
    let d = build_conflict_detail(&r, &s.conflicting_entities);
    ToastOptions::new(ToastIntent::Warning, "Edit Conflict").with_optional_detail(d)
}
pub fn clear_ai_conflict_state(s: &mut Signal<Option<AiConflictState>>) {
    s.set(None);
}
pub fn show_conflict_toast(cs: &AiConflictState, ta: ToastApi) -> Result<ToastHandle, Error> {
    validate_conflict_state(cs)?;
    let o = create_conflict_toast_options(cs);
    let h = ta.toast(o);
    validate_toast_id(&h)?;
    Ok(h)
}
pub fn should_show_conflict_toast(cs: Option<&AiConflictState>) -> Result<bool, Error> {
    match cs {
        Some(s) => Ok(s.has_valid_reason() || !s.conflicting_entities.is_empty()),
        None => Ok(false),
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ToastId(pub u64);
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
    pub const fn stripe_color(self) -> &'static str {
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
#[must_use]
pub fn use_toast() -> ToastApi {
    ToastApi::from_signal(use_context::<Signal<ToastQueue>>())
}
#[must_use]
#[allow(dead_code)]
pub fn toast(o: ToastOptions) -> ToastHandle {
    use_toast().toast(o)
}
