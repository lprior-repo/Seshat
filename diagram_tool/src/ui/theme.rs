#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

impl ThemeMode {
    #[must_use]
    pub const fn persisted_key(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    #[must_use]
    pub fn from_persisted_key(value: &str) -> Option<Self> {
        match value {
            "system" => Some(Self::System),
            "light" => Some(Self::Light),
            "dark" => Some(Self::Dark),
            _ => None,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }

    #[must_use]
    pub const fn resolve(self, system: ThemeScheme) -> ThemeScheme {
        match self {
            Self::System => system,
            Self::Light => ThemeScheme::Light,
            Self::Dark => ThemeScheme::Dark,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeScheme {
    Light,
    Dark,
}

impl ThemeScheme {
    #[must_use]
    #[cfg(target_arch = "wasm32")]
    pub fn from_str(value: &str) -> Self {
        if value == "light" {
            Self::Light
        } else {
            Self::Dark
        }
    }
}

struct ThemeTokens {
    bg_base: &'static str,
    bg_surface: &'static str,
    bg_elevated: &'static str,
    border: &'static str,
    border_subtle: &'static str,
    text_main: &'static str,
    text_muted: &'static str,
    text_dim: &'static str,
    accent: &'static str,
    accent_soft: &'static str,
    selection_rect_fill: &'static str,
    subgraph_preview_fill: &'static str,
    node_bg: &'static str,
    node_bg_subgraph: &'static str,
    node_border: &'static str,
    grid_dot: &'static str,
    edge_default: &'static str,
    toolbar_bg: &'static str,
    success: &'static str,
    error: &'static str,
    warning: &'static str,
    chart_1: &'static str,
    chart_2: &'static str,
    chart_3: &'static str,
    chart_4: &'static str,
    chart_5: &'static str,
}

const fn tokens_for(scheme: ThemeScheme) -> ThemeTokens {
    match scheme {
        ThemeScheme::Dark => ThemeTokens {
            bg_base: "oklch(0.11 0.005 260)",
            bg_surface: "oklch(0.15 0.005 260)",
            bg_elevated: "oklch(0.17 0.005 260)",
            border: "oklch(0.26 0.005 260)",
            border_subtle: "oklch(0.22 0.005 260)",
            text_main: "oklch(0.95 0 0)",
            text_muted: "oklch(0.66 0 0)",
            text_dim: "oklch(0.53 0 0)",
            accent: "oklch(0.72 0.14 165)",
            accent_soft: "color-mix(in oklch, oklch(0.72 0.14 165) 25%, transparent)",
            selection_rect_fill: "color-mix(in oklch, oklch(0.72 0.14 165) 15%, transparent)",
            subgraph_preview_fill: "color-mix(in oklch, oklch(0.72 0.14 165) 8%, transparent)",
            node_bg: "oklch(0.19 0.005 260)",
            node_bg_subgraph: "color-mix(in oklch, oklch(0.22 0.005 260) 60%, transparent)",
            node_border: "oklch(0.30 0.01 260)",
            grid_dot: "oklch(0.25 0.005 260)",
            edge_default: "oklch(0.50 0.01 260)",
            toolbar_bg: "oklch(0.17 0.005 260)",
            success: "#22c55e",
            error: "#ef4444",
            warning: "#f59e0b",
            chart_1: "oklch(0.72 0.14 165)",
            chart_2: "oklch(0.60 0.118 184.704)",
            chart_3: "oklch(0.398 0.07 227.392)",
            chart_4: "oklch(0.828 0.189 84.429)",
            chart_5: "oklch(0.769 0.188 70.08)",
        },
        ThemeScheme::Light => ThemeTokens {
            bg_base: "oklch(0.96 0.005 260)",
            bg_surface: "oklch(0.985 0.004 260)",
            bg_elevated: "oklch(1 0 0)",
            border: "oklch(0.82 0.01 260)",
            border_subtle: "oklch(0.88 0.01 260)",
            text_main: "oklch(0.2 0.01 260)",
            text_muted: "oklch(0.45 0.01 260)",
            text_dim: "oklch(0.57 0.01 260)",
            accent: "oklch(0.62 0.16 192)",
            accent_soft: "color-mix(in oklch, oklch(0.62 0.16 192) 18%, transparent)",
            selection_rect_fill: "color-mix(in oklch, oklch(0.62 0.16 192) 18%, transparent)",
            subgraph_preview_fill: "color-mix(in oklch, oklch(0.62 0.16 192) 10%, transparent)",
            node_bg: "oklch(0.995 0.002 260)",
            node_bg_subgraph: "color-mix(in oklch, oklch(0.94 0.02 260) 55%, transparent)",
            node_border: "oklch(0.76 0.01 260)",
            grid_dot: "oklch(0.85 0.01 260)",
            edge_default: "oklch(0.56 0.02 260)",
            toolbar_bg: "oklch(0.975 0.004 260)",
            success: "#16a34a",
            error: "#dc2626",
            warning: "#d97706",
            chart_1: "oklch(0.62 0.16 192)",
            chart_2: "oklch(0.60 0.118 184.704)",
            chart_3: "oklch(0.49 0.08 227.392)",
            chart_4: "oklch(0.74 0.17 84.429)",
            chart_5: "oklch(0.69 0.17 70.08)",
        },
    }
}

#[must_use]
pub fn css_vars_for(scheme: ThemeScheme) -> String {
    let t = tokens_for(scheme);
    format!(
        "--bg-base:{};--bg-surface:{};--bg-elevated:{};--border:{};--border-subtle:{};--text-main:{};--text-muted:{};--text-dim:{};--accent:{};--accent-soft:{};--selection-rect-fill:{};--subgraph-preview-fill:{};--node-bg:{};--node-bg-subgraph:{};--node-border:{};--grid-dot:{};--edge-default:{};--toolbar-bg:{};--success:{};--error:{};--warning:{};--background:{};--foreground:{};--card:{};--card-foreground:{};--popover:{};--popover-foreground:{};--primary:{};--primary-foreground:{};--secondary:{};--secondary-foreground:{};--muted:{};--muted-foreground:{};--destructive:{};--destructive-foreground:{};--input:{};--ring:{};--sidebar:{};--sidebar-foreground:{};--sidebar-primary:{};--sidebar-primary-foreground:{};--sidebar-accent:{};--sidebar-accent-foreground:{};--sidebar-border:{};--sidebar-ring:{};--canvas:{};--canvas-grid:{};--canvas-dot:{};--node-selected:{};--edge-selected:{};--minimap-bg:{};--chart-1:{};--chart-2:{};--chart-3:{};--chart-4:{};--chart-5:{};--radius:0.5rem;--radius-sm:calc(var(--radius) - 4px);--radius-md:calc(var(--radius) - 2px);--radius-lg:var(--radius);--radius-xl:calc(var(--radius) + 4px);--color-background:var(--background);--color-foreground:var(--foreground);--color-card:var(--card);--color-card-foreground:var(--card-foreground);--color-popover:var(--popover);--color-popover-foreground:var(--popover-foreground);--color-primary:var(--primary);--color-primary-foreground:var(--primary-foreground);--color-secondary:var(--secondary);--color-secondary-foreground:var(--secondary-foreground);--color-muted:var(--muted);--color-muted-foreground:var(--muted-foreground);--color-accent:var(--accent);--color-accent-foreground:var(--primary-foreground);--color-destructive:var(--destructive);--color-destructive-foreground:var(--destructive-foreground);--color-border:var(--border);--color-input:var(--input);--color-ring:var(--ring);--color-chart-1:var(--chart-1);--color-chart-2:var(--chart-2);--color-chart-3:var(--chart-3);--color-chart-4:var(--chart-4);--color-chart-5:var(--chart-5);--color-sidebar:var(--sidebar);--color-sidebar-foreground:var(--sidebar-foreground);--color-sidebar-primary:var(--sidebar-primary);--color-sidebar-primary-foreground:var(--sidebar-primary-foreground);--color-sidebar-accent:var(--sidebar-accent);--color-sidebar-accent-foreground:var(--sidebar-accent-foreground);--color-sidebar-border:var(--sidebar-border);--color-sidebar-ring:var(--sidebar-ring);--color-canvas:var(--canvas);--color-canvas-grid:var(--canvas-grid);--color-canvas-dot:var(--canvas-dot);--color-node-bg:var(--node-bg);--color-node-border:var(--node-border);--color-node-selected:var(--node-selected);--color-edge-default:var(--edge-default);--color-edge-selected:var(--edge-selected);--color-minimap-bg:var(--minimap-bg);--color-toolbar-bg:var(--toolbar-bg);",
        t.bg_base,
        t.bg_surface,
        t.bg_elevated,
        t.border,
        t.border_subtle,
        t.text_main,
        t.text_muted,
        t.text_dim,
        t.accent,
        t.accent_soft,
        t.selection_rect_fill,
        t.subgraph_preview_fill,
        t.node_bg,
        t.node_bg_subgraph,
        t.node_border,
        t.grid_dot,
        t.edge_default,
        t.toolbar_bg,
        t.success,
        t.error,
        t.warning,
        t.bg_base,
        t.text_main,
        t.bg_elevated,
        t.text_main,
        t.bg_elevated,
        t.text_main,
        t.accent,
        t.bg_base,
        t.bg_surface,
        t.text_main,
        t.bg_surface,
        t.text_muted,
        t.error,
        t.text_main,
        t.border_subtle,
        t.accent,
        t.bg_surface,
        t.text_main,
        t.accent,
        t.bg_base,
        t.bg_surface,
        t.text_main,
        t.border,
        t.accent,
        t.bg_base,
        t.border_subtle,
        t.grid_dot,
        t.accent,
        t.accent,
        t.bg_surface,
        t.chart_1,
        t.chart_2,
        t.chart_3,
        t.chart_4,
        t.chart_5,
    )
}

pub const APP_FONT: &str = "Iosevka, SFMono-Regular, Menlo, Monaco, Consolas, monospace";

pub const BG_BASE: &str = "var(--bg-base)";
pub const BG_SURFACE: &str = "var(--bg-surface)";
pub const BG_ELEVATED: &str = "var(--bg-elevated)";

pub const BORDER: &str = "var(--border)";
pub const BORDER_SUBTLE: &str = "var(--border-subtle)";

pub const TEXT_MAIN: &str = "var(--text-main)";
pub const TEXT_MUTED: &str = "var(--text-muted)";
pub const TEXT_DIM: &str = "var(--text-dim)";

pub const ACCENT: &str = "var(--accent)";
pub const ACCENT_SOFT: &str = "var(--accent-soft)";
pub const ACCENT_DASH_BORDER: &str = "2px dashed var(--accent)";
pub const SELECTION_RECT_FILL: &str = "var(--selection-rect-fill)";
pub const SELECTION_RECT_STROKE: &str = "var(--accent)";
pub const SUBGRAPH_PREVIEW_FILL: &str = "var(--subgraph-preview-fill)";
pub const SUBGRAPH_PREVIEW_STROKE: &str = "var(--accent)";
pub const NODE_BG: &str = "var(--node-bg)";
pub const NODE_BG_SUBGRAPH: &str = "var(--node-bg-subgraph)";
pub const NODE_BORDER: &str = "var(--node-border)";
pub const GRID_DOT: &str = "var(--grid-dot)";
pub const EDGE_DEFAULT: &str = "var(--edge-default)";
pub const EDGE_SELECTED: &str = "var(--accent)";
pub const TOOLBAR_BG: &str = "var(--toolbar-bg)";
pub const SELECTION_BOUNDS_STROKE: &str =
    "1px dashed color-mix(in oklch, var(--accent) 55%, transparent)";
pub const SUCCESS: &str = "var(--success)";
pub const ERROR: &str = "var(--error)";
pub const WARNING: &str = "var(--warning)";
