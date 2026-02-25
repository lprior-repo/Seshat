#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::ui::theme::{css_vars_for, ThemeMode, ThemeScheme, APP_FONT, BG_BASE, BG_SURFACE, TEXT_MAIN};
use dioxus::prelude::*;

#[component]
pub fn ThemeProvider(children: Element) -> Element {
    #[cfg_attr(not(target_arch = "wasm32"), allow(unused_mut))]
    let mut theme_mode = use_context_provider(|| Signal::new(ThemeMode::System));
    #[cfg_attr(not(target_arch = "wasm32"), allow(unused_mut))]
    let mut system_theme = use_context_provider(|| Signal::new(ThemeScheme::Dark));

    #[cfg(target_arch = "wasm32")]
    use_effect(move || {
        let mut eval = document::eval(
            r#"
                (() => {
                    const key = "diagram_tool.theme_mode";
                    let stored = "system";
                    try {
                        stored = localStorage.getItem(key) ?? "system";
                    } catch (_) {}
                    dioxus.send({ kind: "mode", value: stored });

                    const media = window.matchMedia("(prefers-color-scheme: dark)");
                    const emit = (matchesDark) => dioxus.send({ kind: "system", value: matchesDark ? "dark" : "light" });
                    emit(media.matches);

                    const onChange = (event) => emit(event.matches);
                    if (media.addEventListener) {
                        media.addEventListener("change", onChange);
                    } else if (media.addListener) {
                        media.addListener(onChange);
                    }
                })();
            "#,
        );

        spawn(async move {
            while let Ok(msg) = eval.recv::<serde_json::Value>().await {
                if msg["kind"].as_str() == Some("mode") {
                    if let Some(value) = msg["value"].as_str() {
                        if let Some(mode) = ThemeMode::from_persisted_key(value) {
                            theme_mode.set(mode);
                        }
                    }
                } else if msg["kind"].as_str() == Some("system") {
                    if let Some(value) = msg["value"].as_str() {
                        system_theme.set(ThemeScheme::from_str(value));
                    }
                }
            }
        });
    });

    #[cfg(target_arch = "wasm32")]
    use_effect(move || {
        let mode = theme_mode.read().persisted_key().to_string();
        let _eval = document::eval(&format!(
            "try {{ localStorage.setItem(\"diagram_tool.theme_mode\", \"{mode}\"); }} catch (_) {{}}"
        ));
    });

    let resolved_scheme = theme_mode.read().resolve(*system_theme.read());
    let css_vars = css_vars_for(resolved_scheme);

    rsx! {
        div {
            style: "{css_vars}display:flex; flex-direction:column; height:100vh; width:100vw; font-family:{APP_FONT}; margin:0; padding:0; overflow:hidden; background:radial-gradient(circle at 10% 20%, {BG_SURFACE} 0%, {BG_SURFACE} 55%, {BG_BASE} 100%); color:{TEXT_MAIN};",
            {children}
        }
    }
}
