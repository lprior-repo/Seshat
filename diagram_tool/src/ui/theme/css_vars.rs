/// Static CSS: radius tokens and `--color-*` aliases (no dynamic values).
pub(super) const CSS_STATIC_VARS: &str = "\
--radius:0.5rem;--radius-sm:calc(var(--radius) - 4px);\
--radius-md:calc(var(--radius) - 2px);--radius-lg:var(--radius);\
--radius-xl:calc(var(--radius) + 4px);\
--color-background:var(--background);--color-foreground:var(--foreground);\
--color-card:var(--card);--color-card-foreground:var(--card-foreground);\
--color-popover:var(--popover);--color-popover-foreground:var(--popover-foreground);\
--color-primary:var(--primary);--color-primary-foreground:var(--primary-foreground);\
--color-secondary:var(--secondary);--color-secondary-foreground:var(--secondary-foreground);\
--color-muted:var(--muted);--color-muted-foreground:var(--muted-foreground);\
--color-accent:var(--accent);--color-accent-foreground:var(--primary-foreground);\
--color-destructive:var(--destructive);--color-destructive-foreground:var(--destructive-foreground);\
--color-border:var(--border);--color-input:var(--input);--color-ring:var(--ring);\
--color-chart-1:var(--chart-1);--color-chart-2:var(--chart-2);\
--color-chart-3:var(--chart-3);--color-chart-4:var(--chart-4);\
--color-chart-5:var(--chart-5);\
--color-sidebar:var(--sidebar);--color-sidebar-foreground:var(--sidebar-foreground);\
--color-sidebar-primary:var(--sidebar-primary);\
--color-sidebar-primary-foreground:var(--sidebar-primary-foreground);\
--color-sidebar-accent:var(--sidebar-accent);\
--color-sidebar-accent-foreground:var(--sidebar-accent-foreground);\
--color-sidebar-border:var(--sidebar-border);--color-sidebar-ring:var(--sidebar-ring);\
--color-canvas:var(--canvas);--color-canvas-grid:var(--canvas-grid);\
--color-canvas-dot:var(--canvas-dot);--color-node-bg:var(--node-bg);\
--color-node-border:var(--node-border);--color-node-selected:var(--node-selected);\
--color-edge-default:var(--edge-default);--color-edge-selected:var(--edge-selected);\
--color-minimap-bg:var(--minimap-bg);--color-toolbar-bg:var(--toolbar-bg);";

pub const APP_FONT: &str = "Iosevka, SFMono-Regular, Menlo, Monaco, Consolas, monospace";

pub const BG_BASE: &str = "var(--bg-base)";
pub const BG_SURFACE: &str = "var(--bg-surface)";
pub const BG_ELEVATED: &str = "var(--bg-elevated)";

pub const BORDER: &str = "var(--border)";
pub const BORDER_SUBTLE: &str = "var(--border-subtle)";

pub const TEXT_MAIN: &str = "var(--text-main)";
pub const TEXT_MUTED: &str = "var(--text-muted)";
#[allow(dead_code)]
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
