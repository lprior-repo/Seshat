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
    pub properties: bool,
    pub minimap: bool,
    pub validation: bool,
}

impl Default for PanelVisibility {
    fn default() -> Self {
        Self {
            sidebar: true,
            properties: false,
            minimap: true,
            validation: false,
        }
    }
}
