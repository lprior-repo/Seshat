#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

pub mod core;
pub mod group;
pub mod menu;
pub mod provider;
pub mod trigger;
pub mod types;
pub mod utils;

pub use core::*;
pub use group::*;
pub use menu::*;
pub use provider::*;
pub use trigger::*;
pub use types::*;
