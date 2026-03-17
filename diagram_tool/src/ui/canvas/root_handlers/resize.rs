use canvas_domain::perf::{normalize_viewport, viewport_changed};
use dioxus::prelude::*;

pub fn use_resize_handler(
    mut canvas_origin: Signal<(f64, f64)>,
    mut viewport_size: Signal<(f64, f64)>,
) {
    use_effect(move || {
        let mut eval = document::eval(
            r"
                if (window.__seshat_canvas_resize_cleanup) {
                    window.__seshat_canvas_resize_cleanup();
                }

                const target = document.querySelector('.canvas-container');
                if (target) {
                    let rafId = 0;
                    let lastLeft = Number.NaN;
                    let lastTop = Number.NaN;
                    let lastWidth = Number.NaN;
                    let lastHeight = Number.NaN;
                    const notify = (left, top, width, height) => {
                        if (
                            Math.abs(left - lastLeft) < 0.5 &&
                            Math.abs(top - lastTop) < 0.5 &&
                            Math.abs(width - lastWidth) < 0.5 &&
                            Math.abs(height - lastHeight) < 0.5
                        ) {
                            return;
                        }
                        lastLeft = left;
                        lastTop = top;
                        lastWidth = width;
                        lastHeight = height;
                        dioxus.send({ type: 'resize', left, top, width, height });
                    };

                    const scheduleNotify = () => {
                        if (rafId !== 0) {
                            return;
                        }
                        rafId = window.requestAnimationFrame(() => {
                            rafId = 0;
                            const r = target.getBoundingClientRect();
                            notify(r.left, r.top, r.width, r.height);
                        });
                    };

                    const ro = new ResizeObserver(() => scheduleNotify());
                    ro.observe(target);

                    const pollOrigin = () => {
                        const rect = target.getBoundingClientRect();
                        dioxus.send({ type: 'resize', left: rect.left, top: rect.top, width: rect.width, height: rect.height });
                        rafId = window.requestAnimationFrame(pollOrigin);
                    };
                    rafId = window.requestAnimationFrame(pollOrigin);

                    const onScroll = () => {
                        const rect = target.getBoundingClientRect();
                        dioxus.send({ type: 'resize', left: rect.left, top: rect.top, width: rect.width, height: rect.height });
                    };
                    window.addEventListener('scroll', onScroll, { passive: true, capture: true });
                    document.addEventListener('scroll', onScroll, { passive: true, capture: true });

                    window.addEventListener('resize', scheduleNotify, { passive: true });
                    window.addEventListener('scroll', scheduleNotify, { passive: true, capture: true });
                    document.addEventListener('scroll', scheduleNotify, { passive: true, capture: true });
                    window.__seshat_canvas_resize_cleanup = () => {
                        ro.disconnect();
                        if (rafId !== 0) {
                            window.cancelAnimationFrame(rafId);
                        }
                        window.removeEventListener('scroll', onScroll, true);
                        document.removeEventListener('scroll', onScroll, true);
                        window.removeEventListener('resize', scheduleNotify);
                        window.removeEventListener('scroll', scheduleNotify, true);
                        document.removeEventListener('scroll', scheduleNotify, true);
                    };

                    scheduleNotify();
                }
            ",
        );

        spawn(async move {
            while let Ok(json) = eval.recv::<serde_json::Value>().await {
                if json["type"].as_str() == Some("resize") {
                    canvas_origin.set((
                        json["left"].as_f64().map_or(0.0, |v| v),
                        json["top"].as_f64().map_or(0.0, |v| v),
                    ));
                    let next = normalize_viewport(
                        json["width"].as_f64().map_or(1200.0, |v| v),
                        json["height"].as_f64().map_or(800.0, |v| v),
                    );
                    let current = *viewport_size.read();
                    if viewport_changed(current, next) {
                        viewport_size.set(next);
                    }
                }
            }
        });
    });
}
