use crate::ui::mobile::SidebarUiState;
use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum SidebarSide {
    #[default]
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum SidebarVariant {
    #[default]
    Sidebar,
    Floating,
    Inset,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum SidebarCollapsible {
    #[default]
    Offcanvas,
    Icon,
    None,
}

#[derive(Clone, Copy)]
pub struct SidebarProviderContext {
    pub sidebar_ui: Signal<SidebarUiState>,
    pub side: SidebarSide,
    pub variant: SidebarVariant,
    pub collapsible: SidebarCollapsible,
}
