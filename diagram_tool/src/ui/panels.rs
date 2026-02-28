#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct PanelVisibility {
    pub sidebar: bool,
    pub minimap: bool,
    pub validation: bool,
}

impl Default for PanelVisibility {
    fn default() -> Self {
        Self {
            sidebar: true,
            minimap: true,
            validation: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PanelVisibility;

    #[test]
    fn given_default_panel_visibility_when_created_then_sidebar_and_minimap_visible() {
        let panels = PanelVisibility::default();
        assert!(panels.sidebar);
        assert!(panels.minimap);
        assert!(!panels.validation);
    }
}
