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
//! - `app`, `hooks`, `icons`, `layout`, `mutation`, `store`, `ui`

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![allow(
    clippy::assigning_clones,
    clippy::branches_sharing_code,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cloned_instead_of_copied,
    clippy::default_trait_access,
    clippy::derive_partial_eq_without_eq,
    clippy::double_must_use,
    clippy::float_cmp,
    clippy::imprecise_flops,
    clippy::iter_on_single_items,
    clippy::manual_let_else,
    clippy::manual_midpoint,
    clippy::manual_range_contains,
    clippy::match_same_arms,
    clippy::missing_const_for_fn,
    clippy::missing_errors_doc,
    clippy::module_inception,
    clippy::must_use_candidate,
    clippy::needless_pass_by_value,
    clippy::option_if_let_else,
    clippy::or_fun_call,
    clippy::ptr_arg,
    clippy::redundant_else,
    clippy::ref_option,
    clippy::suboptimal_flops,
    clippy::too_many_lines,
    clippy::trivially_copy_pass_by_ref,
    clippy::unnecessary_result_map_or_else,
    clippy::unnecessary_to_owned,
    clippy::unnecessary_wraps
)]
#![cfg_attr(
    test,
    allow(
        dead_code,
        unused_imports,
        unused_variables,
        clippy::duplicated_attributes,
        clippy::expect_used,
        clippy::ignore_without_reason,
        clippy::panic,
        clippy::similar_names,
        clippy::unwrap_used
    )
)]
#![forbid(unsafe_code)]

pub mod app;
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
pub mod store;
#[cfg(not(target_arch = "wasm32"))]
pub mod store_async;
#[cfg(not(target_arch = "wasm32"))]
pub mod store_bridge;
pub mod ui;
pub mod viewport;

// Test harness is public for use in tests
pub mod test_utils;

#[cfg(test)]
mod test_infrastructure_tests;
