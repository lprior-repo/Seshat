use super::{Toast, ToastAction, ToastId, ToastIntent, ToastUpdate, MAX_TOASTS};
#[derive(Default)]
pub struct ToastQueue {
    next_id: u64,
    items: Vec<Toast>,
}
impl ToastQueue {
    #[must_use]
    #[allow(dead_code)]
    pub fn add(&mut self, i: ToastIntent, t: impl Into<String>, d: Option<String>) -> ToastId {
        self.add_with_action(i, t, d, None)
    }
    #[must_use]
    pub fn add_with_action(
        &mut self,
        i: ToastIntent,
        t: impl Into<String>,
        d: Option<String>,
        a: Option<ToastAction>,
    ) -> ToastId {
        let id = ToastId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        self.items.push(Toast {
            id,
            intent: i,
            title: t.into(),
            detail: d,
            action: a,
            dismissed: false,
        });
        while self.items.len() > MAX_TOASTS {
            self.items.remove(
                self.items
                    .iter()
                    .position(|x| x.dismissed)
                    .unwrap_or_default(),
            );
        }
        id
    }
    pub fn update(&mut self, id: ToastId, p: ToastUpdate) -> bool {
        if let Some(x) = self.items.iter_mut().find(|x| x.id == id) {
            if let Some(t) = p.title {
                x.title = t;
            }
            if let Some(d) = p.detail {
                x.detail = d;
            }
            if let Some(i) = p.intent {
                x.intent = i;
            }
            if let Some(a) = p.action {
                x.action = a;
            }
            true
        } else {
            false
        }
    }
    pub fn dismiss(&mut self, id: ToastId) -> bool {
        if let Some(x) = self.items.iter_mut().find(|x| x.id == id) {
            x.dismissed = true;
            true
        } else {
            false
        }
    }
    pub fn dismiss_target(&mut self, id: Option<ToastId>) -> bool {
        if let Some(x) = id {
            self.dismiss(x)
        } else {
            self.dismiss_all()
        }
    }
    pub fn dismiss_all(&mut self) -> bool {
        let mut c = false;
        for x in &mut self.items {
            if !x.dismissed {
                x.dismissed = true;
                c = true;
            }
        }
        c
    }
    pub fn remove(&mut self, id: ToastId) -> bool {
        if let Some(x) = self.items.iter().position(|x| x.id == id) {
            self.items.remove(x);
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
