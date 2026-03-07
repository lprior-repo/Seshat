//! Diagram Tool Library
//!
//! This module exposes the library components for use in integration tests.
//!
//! ## API Stability
//!
//! This crate distinguishes between **stable** public APIs and **internal** APIs:
//!
//! - **Stable APIs** (`pub`) - Guaranteed not to break between minor versions.
//!   These are documented in the `# Public API` section below.
//! - **Internal APIs** (`pub(crate)`) - For use within the crate only. May break
//!   at any time without notice.
//! - **Experimental APIs** - Marked with `#[doc(hidden)]` or unstable feature flags.
//!
//! ## Public API
//!
//! The following types and modules are considered stable public API:
//!
//! ### Viewport Module
//! - [`viewport::ViewportState`] - Camera/viewport state management
//! - [`viewport::WorldPoint`] - World coordinate point
//! - [`viewport::ScreenPoint`] - Screen coordinate point
//! - [`viewport::FitTransform`] - Fit-to-content transformation
//! - [`viewport::MIN_ZOOM`], [`viewport::MAX_ZOOM`] - Zoom bounds
//!
//! ### Models Module
//! - [`models::DiagramDocument`] - Complete diagram document
//! - [`models::DocumentData`] - Document nodes and edges
//! - [`models::Node`] - Diagram node
//! - [`models::Edge`] - Diagram edge
//! - [`models::NodeId`] - Node identifier (newtype)
//! - [`models::EdgeId`] - Edge identifier (newtype)
//! - [`models::Revision`] - Document revision counter
//!
//! ### History Module
//! - [`history::History`] - Undo/redo history manager
//!
//! ### Geometry Module
//! - [`geometry::AABB`] - Axis-aligned bounding box
//! - [`geometry::Point`] - 2D point
//!
//! ### Export Module
//! - [`export::SvgExport`] - SVG export functionality
//! - [`export::PngExport`] - PNG export functionality
//!
//! ## Internal Modules
//!
//! The following modules are internal and not part of the public API:
//! - `app`, `backend`, `cli`, `cli_persistence`, `hooks`, `icons`, `layout`,
//!   `mutation`, `perf`, `store`, `ui`

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

pub mod app;
#[cfg(not(target_arch = "wasm32"))]
pub mod backend;
#[cfg(not(target_arch = "wasm32"))]
pub mod cli;
#[cfg(not(target_arch = "wasm32"))]
pub mod cli_persistence;
pub mod core;
pub mod export;
pub mod geometry;
pub mod history;
pub mod hooks;
pub mod icons;
pub mod layout;
pub mod models;
pub mod mutation;
#[cfg(not(target_arch = "wasm32"))]
pub mod perf;
#[cfg(not(target_arch = "wasm32"))]
pub mod store;
#[cfg(all(
    not(target_arch = "wasm32"),
    feature = "async-db"
))]
pub mod store_async;
pub mod ui;
pub mod viewport;

#[cfg(test)]
mod tests {
    mod contracts;
}
