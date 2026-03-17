use super::{ToastAction, ToastId, ToastIntent, ToastQueue, ToastUpdate};
use dioxus::prelude::*;
#[derive(Clone, Copy)]
pub struct ToastApi {
    queue: Signal<ToastQueue>,
}
#[derive(Clone, Copy)]
pub struct ToastHandle {
    id: ToastId,
    api: ToastApi,
}
pub struct ToastOptions {
    pub intent: ToastIntent,
    pub title: String,
    pub detail: Option<String>,
    pub action: Option<ToastAction>,
}
impl ToastOptions {
    #[must_use]
    pub fn new(i: ToastIntent, t: impl Into<String>) -> Self {
        Self {
            intent: i,
            title: t.into(),
            detail: None,
            action: None,
        }
    }
    #[must_use]
    pub fn with_detail(self, d: impl Into<String>) -> Self {
        Self {
            detail: Some(d.into()),
            ..self
        }
    }
    #[must_use]
    pub fn with_optional_detail(self, d: Option<String>) -> Self {
        Self { detail: d, ..self }
    }
    #[must_use]
    #[allow(dead_code)]
    pub fn with_action(self, a: ToastAction) -> Self {
        Self {
            action: Some(a),
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
    pub fn update(self, p: ToastUpdate) -> bool {
        self.api.update(self.id, p)
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
    pub const fn from_signal(q: Signal<ToastQueue>) -> Self {
        Self { queue: q }
    }
    #[must_use]
    pub fn toast(self, o: ToastOptions) -> ToastHandle {
        let mut q = self.queue;
        let mut id = ToastId(0);
        q.with_mut(|s| {
            id = s.add_with_action(o.intent, o.title, o.detail, o.action);
        });
        ToastHandle { id, api: self }
    }
    #[must_use]
    pub fn show(self, i: ToastIntent, t: impl Into<String>, d: Option<String>) -> ToastId {
        self.toast(ToastOptions::new(i, t).with_optional_detail(d))
            .id()
    }
    #[must_use]
    pub fn update(self, id: ToastId, p: ToastUpdate) -> bool {
        let mut q = self.queue;
        let mut c = false;
        q.with_mut(|s| {
            c = s.update(id, p);
        });
        c
    }
    #[must_use]
    pub fn dismiss(self, id: Option<ToastId>) -> bool {
        let mut q = self.queue;
        let mut c = false;
        q.with_mut(|s| {
            c = s.dismiss_target(id);
        });
        c
    }
    #[allow(dead_code)]
    #[must_use]
    pub fn remove(self, id: ToastId) -> bool {
        let mut q = self.queue;
        let mut c = false;
        q.with_mut(|s| {
            c = s.remove(id);
        });
        c
    }
    #[must_use]
    pub fn error(self, t: impl Into<String>, d: Option<String>) -> ToastId {
        self.show(ToastIntent::Error, t, d)
    }
}
