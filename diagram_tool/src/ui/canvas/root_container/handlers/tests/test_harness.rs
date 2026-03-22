use crate::ui::canvas::state::CanvasState;
use crate::app::AppState;
use dioxus::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

pub fn with_canvas_state<F>(f: F)
where
    F: FnOnce(CanvasState) + 'static,
{
    let f = Rc::new(RefCell::new(Some(f)));
    let mut vdom = VirtualDom::new_with_props(move |()| {
        use_context_provider(|| AppState::new());
        let state = crate::ui::canvas::state::use_canvas_state();
        if let Some(func) = f.borrow_mut().take() {
            func(state);
        }
        rsx! { div {} }
    });
    let _ = vdom.rebuild_in_place();
}
