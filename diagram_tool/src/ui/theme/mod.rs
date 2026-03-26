#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

mod css_vars;
mod tokens;

use css_vars::CSS_STATIC_VARS;
pub use css_vars::{
    ACCENT, ACCENT_DASH_BORDER, ACCENT_SOFT, APP_FONT, BG_BASE, BG_ELEVATED, BG_SURFACE, BORDER,
    BORDER_SUBTLE, EDGE_DEFAULT, EDGE_SELECTED, ERROR, GRID_DOT, NODE_BG, NODE_BG_SUBGRAPH,
    NODE_BORDER, SELECTION_BOUNDS_STROKE, SELECTION_RECT_FILL, SELECTION_RECT_STROKE,
    SUBGRAPH_PREVIEW_FILL, SUBGRAPH_PREVIEW_STROKE, SUCCESS, TEXT_DIM, TEXT_MAIN, TEXT_MUTED,
    TOOLBAR_BG, WARNING,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeMode {
    System,
    Light,
    Dark,
    White,
}

impl ThemeMode {
    #[must_use]
    pub const fn persisted_key(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
            Self::White => "white",
        }
    }

    #[must_use]
    pub fn from_persisted_key(value: &str) -> Option<Self> {
        match value {
            "system" => Some(Self::System),
            "light" => Some(Self::Light),
            "dark" => Some(Self::Dark),
            "white" => Some(Self::White),
            _ => None,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Light => "Light",
            Self::Dark => "Dark",
            Self::White => "White",
        }
    }

    /// Returns the next theme mode in the cycle: System → Light → Dark → White → System.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::System => Self::Light,
            Self::Light => Self::Dark,
            Self::Dark => Self::White,
            Self::White => Self::System,
        }
    }

    #[must_use]
    pub const fn resolve(self, system: ThemeScheme) -> ThemeScheme {
        match self {
            Self::System => system,
            Self::Light => ThemeScheme::Light,
            Self::Dark => ThemeScheme::Dark,
            Self::White => ThemeScheme::White,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeScheme {
    Light,
    Dark,
    White,
}

impl ThemeScheme {
    /// Parse theme from string. Returns None for invalid inputs instead of silent fallback.
    #[must_use]
    #[cfg(target_arch = "wasm32")]
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "light" => Some(Self::Light),
            "dark" => Some(Self::Dark),
            "white" => Some(Self::White),
            _ => None,
        }
    }
}

#[must_use]
pub fn css_vars_for(scheme: ThemeScheme) -> String {
    let t = tokens::tokens_for(scheme);
    let dynamic = [
        t.token_vars(),
        t.shadcn_vars(),
        t.sidebar_vars(),
        t.canvas_vars(),
    ]
    .concat();
    dynamic + CSS_STATIC_VARS
}

#[cfg(test)]
mod css_var_tests;
#[cfg(test)]
mod theme_mode_tests;
#[cfg(test)]
mod theme_scheme_tests;
