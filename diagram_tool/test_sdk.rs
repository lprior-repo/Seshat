use dioxus::prelude::*;
fn test(evt: Event<DragData>) {
    if let Some(w) = evt.web_event() {
        // ok
    }
}
