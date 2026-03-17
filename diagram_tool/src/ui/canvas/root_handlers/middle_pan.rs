use dioxus::prelude::*;

pub fn use_middle_pan_handler() {
    use_effect(move || {
        let _ = document::eval(
            r"
                if (window.__seshat_canvas_middle_pan_cleanup) {
                    window.__seshat_canvas_middle_pan_cleanup();
                }

                const preventMiddleAutoScroll = (event) => {
                    const target = event.target;
                    const inCanvas = target && target.closest && target.closest('.canvas-container');
                    if (event.button === 1 && inCanvas) {
                        event.preventDefault();
                    }
                };

                window.addEventListener('mousedown', preventMiddleAutoScroll, { capture: true });
                window.__seshat_canvas_middle_pan_cleanup = () => {
                    window.removeEventListener('mousedown', preventMiddleAutoScroll, { capture: true });
                };
            ",
        );
    });
}
