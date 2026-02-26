#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

pub const SIDEBAR_COOKIE_NAME: &str = "sidebar_state";
#[cfg(target_arch = "wasm32")]
pub const SIDEBAR_COOKIE_MAX_AGE_SECONDS: u32 = 60 * 60 * 24 * 7;
pub const SIDEBAR_LEGACY_LOCAL_STORAGE_KEY: &str = "diagram_tool.sidebar_open";

#[cfg(target_arch = "wasm32")]
pub fn persist_sidebar_open(open: bool) {
    let open_value = if open { "true" } else { "false" };
    let script = format!(
        r#"(() => {{
            try {{
                document.cookie = \"{SIDEBAR_COOKIE_NAME}={open_value}; path=/; max-age={SIDEBAR_COOKIE_MAX_AGE_SECONDS}; samesite=lax\";
            }} catch (_) {{}}
            try {{
                localStorage.setItem(\"{SIDEBAR_LEGACY_LOCAL_STORAGE_KEY}\", \"{open_value}\");
            }} catch (_) {{}}
        }})();"#
    );
    let _eval = dioxus::document::eval(&script);
}

#[cfg(not(target_arch = "wasm32"))]
pub const fn persist_sidebar_open(_open: bool) {}
