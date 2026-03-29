#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

use crate::ui::panels::PanelVisibility;
use crate::ui::sidebar_persistence::{
    persist_sidebar_open, SIDEBAR_COOKIE_NAME, SIDEBAR_LEGACY_LOCAL_STORAGE_KEY,
};
use dioxus::prelude::*;

pub const MOBILE_BREAKPOINT: u32 = 768;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SidebarUiState {
    pub is_mobile: bool,
    pub open: bool,
    pub open_mobile: bool,
}

impl Default for SidebarUiState {
    fn default() -> Self {
        Self {
            is_mobile: false,
            open: true,
            open_mobile: false,
        }
    }
}

#[must_use]
fn message_type(message: &serde_json::Value) -> &str {
    message
        .get("type")
        .and_then(serde_json::Value::as_str)
        .map_or("", |value| value)
}

#[must_use]
fn message_bool(message: &serde_json::Value, key: &str) -> Option<bool> {
    message.get(key).and_then(serde_json::Value::as_bool)
}

pub fn open_sidebar(mut sidebar_ui: Signal<SidebarUiState>) {
    sidebar_ui.with_mut(|state| {
        if state.is_mobile {
            state.open_mobile = true;
        } else {
            state.open = true;
        }
    });
    let should_persist = !sidebar_ui.read().is_mobile;
    if should_persist {
        persist_sidebar_open(true);
    }
}

pub fn close_sidebar(mut sidebar_ui: Signal<SidebarUiState>) {
    sidebar_ui.with_mut(|state| {
        if state.is_mobile {
            state.open_mobile = false;
        } else {
            state.open = false;
        }
    });
    let should_persist = !sidebar_ui.read().is_mobile;
    if should_persist {
        persist_sidebar_open(false);
    }
}

pub fn toggle_sidebar(mut sidebar_ui: Signal<SidebarUiState>, mut panels: Signal<PanelVisibility>) {
    if panels.read().sidebar {
        sidebar_ui.with_mut(|state| {
            if state.is_mobile {
                state.open_mobile = !state.open_mobile;
            } else {
                state.open = !state.open;
                persist_sidebar_open(state.open);
            }
        });
    } else {
        panels.with_mut(|panel_state| panel_state.sidebar = true);
        sidebar_ui.with_mut(|state| {
            if state.is_mobile {
                state.open_mobile = true;
            } else {
                state.open = true;
                persist_sidebar_open(true);
            }
        });
    }
}

pub fn use_sidebar_mobile_bridge(
    mut sidebar_ui: Signal<SidebarUiState>,
    panels: Signal<PanelVisibility>,
) {
    use_effect(move || {
        let mut eval = document::eval(&format!(
            r"
                const BREAKPOINT = {};
                const readSidebarPreference = () => {{
                    let open = true;
                    let foundCookie = false;
                    try {{
                        const cookieValue = document.cookie
                            .split('; ')
                            .find((value) => value.startsWith('{}='))
                            ?.split('=')
                            .slice(1)
                            .join('=');
                        if (cookieValue === 'true' || cookieValue === 'false') {{
                            open = cookieValue === 'true';
                            foundCookie = true;
                        }}
                    }} catch (_) {{}}
                    if (!foundCookie) {{
                        try {{
                            const stored = localStorage.getItem('{}');
                            if (stored === 'true' || stored === 'false') {{
                                open = stored === 'true';
                            }}
                        }} catch (_) {{}}
                    }}
                    dioxus.send({{ type: 'sidebar-open', open }});
                }};
                const emitViewport = () => {{
                    dioxus.send({{ type: 'viewport', isMobile: window.innerWidth < BREAKPOINT }});
                }};
                if (!window.__diagramToolViewportListenerInstalled) {{
                    window.__diagramToolViewportListenerInstalled = true;
                    window.addEventListener('resize', emitViewport);
                }}
                if (!window.__diagramToolSidebarHotkeyInstalled) {{
                    window.__diagramToolSidebarHotkeyInstalled = true;
                    window.addEventListener('keydown', (event) => {{
                        const active = document.activeElement;
                        const editing = active && (
                            active.tagName === 'INPUT' ||
                            active.tagName === 'TEXTAREA' ||
                            active.isContentEditable
                        );
                        const modifier = event.metaKey || event.ctrlKey;
                        const keyB = event.code === 'KeyB' || event.key === 'b' || event.key === 'B';
                        if (!event.defaultPrevented && modifier && keyB && !editing) {{
                            event.preventDefault();
                            dioxus.send({{ type: 'toggle-sidebar' }});
                        }}
                    }});
                }}
                if (!window.__diagramToolSidebarPrefLoaded) {{
                    window.__diagramToolSidebarPrefLoaded = true;
                    readSidebarPreference();
                }}
                if (!window.__diagramToolViewportMediaInstalled) {{
                    window.__diagramToolViewportMediaInstalled = true;
                    const mediaQuery = window.matchMedia('(max-width: {}px)');
                    mediaQuery.addEventListener('change', emitViewport);
                }}
                if (!window.__diagramToolViewportBooted) {{
                    window.__diagramToolViewportBooted = true;
                    emitViewport();
                }}
            ",
            MOBILE_BREAKPOINT,
            SIDEBAR_COOKIE_NAME,
            SIDEBAR_LEGACY_LOCAL_STORAGE_KEY,
            MOBILE_BREAKPOINT.saturating_sub(1)
        ));

        spawn(async move {
            while let Ok(message) = eval.recv::<serde_json::Value>().await {
                match message_type(&message) {
                    "viewport" => {
                        let is_mobile =
                            message_bool(&message, "isMobile").is_some_and(|value| value);
                        sidebar_ui.with_mut(|state| {
                            state.is_mobile = is_mobile;
                            if !is_mobile {
                                state.open_mobile = false;
                            }
                        });
                    }
                    "sidebar-open" => {
                        let open = message_bool(&message, "open").is_none_or(|value| value);
                        sidebar_ui.with_mut(|state| {
                            state.open = open;
                        });
                    }
                    "toggle-sidebar" => {
                        toggle_sidebar(sidebar_ui, panels);
                    }
                    _ => {}
                }
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::prelude::*;

    #[test]
    fn test_sidebar_state_default() {
        let state = SidebarUiState::default();
        assert!(!state.is_mobile);
        assert!(state.open);
        assert!(!state.open_mobile);
    }

    #[test]
    fn test_open_sidebar() {
        #[component]
        fn TestComponent() -> Element {
            let sidebar_ui = Signal::new(SidebarUiState {
                is_mobile: true,
                open: true,
                open_mobile: false,
            });

            open_sidebar(sidebar_ui);
            assert!(sidebar_ui.read().open_mobile);

            close_sidebar(sidebar_ui);
            assert!(!sidebar_ui.read().open_mobile);

            rsx! { div {} }
        }

        let mut vdom = VirtualDom::new(TestComponent);
        vdom.rebuild_in_place();
    }

    #[test]
    fn test_toggle_sidebar() {
        #[component]
        fn TestComponent() -> Element {
            let sidebar_ui = Signal::new(SidebarUiState {
                is_mobile: true,
                open: true,
                open_mobile: false,
            });
            let panels = Signal::new(PanelVisibility {
                sidebar: true,
                ..Default::default()
            });

            toggle_sidebar(sidebar_ui, panels);
            assert!(sidebar_ui.read().open_mobile);

            toggle_sidebar(sidebar_ui, panels);
            assert!(!sidebar_ui.read().open_mobile);

            rsx! { div {} }
        }

        let mut vdom = VirtualDom::new(TestComponent);
        vdom.rebuild_in_place();
    }
}
