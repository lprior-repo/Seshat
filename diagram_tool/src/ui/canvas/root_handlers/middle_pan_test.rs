#[cfg(test)]
mod tests {
    use crate::ui::canvas::root_handlers::middle_pan::use_middle_pan_handler;
    use dioxus::prelude::*;

    #[test]
    fn test_middle_pan_eval() {
        let mut vdom = VirtualDom::new(|| {
            use_middle_pan_handler();
            rsx! { div {} }
        });
        let () = vdom.rebuild_in_place();
    }
}
