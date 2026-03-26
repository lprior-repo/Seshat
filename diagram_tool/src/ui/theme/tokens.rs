pub(super) struct ThemeTokens {
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

const fn dark_tokens() -> ThemeTokens {
    ThemeTokens {
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
    }
}

const fn light_tokens() -> ThemeTokens {
    ThemeTokens {
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
    }
}

fn white_tokens() -> ThemeTokens {
    unreachable!("White tokens not yet implemented (seshat-b36)")
}

pub(super) fn tokens_for(scheme: super::ThemeScheme) -> ThemeTokens {
    match scheme {
        super::ThemeScheme::Dark => dark_tokens(),
        super::ThemeScheme::Light => light_tokens(),
        super::ThemeScheme::White => white_tokens(),
    }
}

impl ThemeTokens {
    /// Core theme token CSS variables (21 mappings).
    pub(super) fn token_vars(&self) -> String {
        format!(
            "--bg-base:{};--bg-surface:{};--bg-elevated:{};--border:{};--border-subtle:{};\
             --text-main:{};--text-muted:{};--text-dim:{};--accent:{};--accent-soft:{};\
             --selection-rect-fill:{};--subgraph-preview-fill:{};--node-bg:{};\
             --node-bg-subgraph:{};--node-border:{};--grid-dot:{};--edge-default:{};\
             --toolbar-bg:{};--success:{};--error:{};--warning:{};",
            self.bg_base,
            self.bg_surface,
            self.bg_elevated,
            self.border,
            self.border_subtle,
            self.text_main,
            self.text_muted,
            self.text_dim,
            self.accent,
            self.accent_soft,
            self.selection_rect_fill,
            self.subgraph_preview_fill,
            self.node_bg,
            self.node_bg_subgraph,
            self.node_border,
            self.grid_dot,
            self.edge_default,
            self.toolbar_bg,
            self.success,
            self.error,
            self.warning,
        )
    }

    /// Shadcn-compatible semantic CSS variables (16 mappings).
    pub(super) fn shadcn_vars(&self) -> String {
        format!(
            "--background:{};--foreground:{};--card:{};--card-foreground:{};\
             --popover:{};--popover-foreground:{};--primary:{};--primary-foreground:{};\
             --secondary:{};--secondary-foreground:{};--muted:{};--muted-foreground:{};\
             --destructive:{};--destructive-foreground:{};--input:{};--ring:{};",
            self.bg_base,
            self.text_main,
            self.bg_elevated,
            self.text_main,
            self.bg_elevated,
            self.text_main,
            self.accent,
            self.bg_base,
            self.bg_surface,
            self.text_main,
            self.bg_surface,
            self.text_muted,
            self.error,
            self.text_main,
            self.border_subtle,
            self.accent,
        )
    }

    /// Sidebar CSS variables (8 mappings).
    pub(super) fn sidebar_vars(&self) -> String {
        format!(
            "--sidebar:{};--sidebar-foreground:{};--sidebar-primary:{};\
             --sidebar-primary-foreground:{};--sidebar-accent:{};\
             --sidebar-accent-foreground:{};--sidebar-border:{};--sidebar-ring:{};",
            self.bg_surface,
            self.text_main,
            self.accent,
            self.bg_base,
            self.bg_surface,
            self.text_main,
            self.border,
            self.accent,
        )
    }

    /// Canvas, diagram, and chart CSS variables (11 mappings).
    pub(super) fn canvas_vars(&self) -> String {
        format!(
            "--canvas:{};--canvas-grid:{};--canvas-dot:{};--node-selected:{};\
             --edge-selected:{};--minimap-bg:{};--chart-1:{};--chart-2:{};\
             --chart-3:{};--chart-4:{};--chart-5:{};",
            self.bg_base,
            self.border_subtle,
            self.grid_dot,
            self.accent,
            self.accent,
            self.bg_surface,
            self.chart_1,
            self.chart_2,
            self.chart_3,
            self.chart_4,
            self.chart_5,
        )
    }
}
