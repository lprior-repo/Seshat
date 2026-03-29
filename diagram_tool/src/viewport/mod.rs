//! Viewport module for camera/viewport operations
//!
//! This module provides the ViewportState struct and operations for managing
//! the camera transformation between screen coordinates and world coordinates.
//!
//! ## Design by Contract
//!
//! ### Preconditions
//! - P1: Zoom value must be finite and positive (default fallback to 1.0)
//! - P2: Camera coordinates must be finite (clamped if invalid)
//! - P3: Viewport dimensions must be positive (minimum 1.0)
//! - P4: Coordinate transforms require valid zoom/pan state
//! - P5: Zoom bounds: 0.1 <= zoom <= 4.0 (clamped)
//! - P6: Fit-to-viewport requires valid content bounds (returns None if invalid)
//! - P7: Pan delta must be finite
//!
//! ### Postconditions
//! - Q1: After zoom: new zoom within [0.1, 4.0]
//! - Q2: After pan: camera coordinates are finite
//! - Q3: Screen-to-world is inverse of world-to-screen
//! - Q4: Fit-to-viewport preserves aspect ratio
//! - Q5: Zoom around point keeps point under cursor
//! - Q6: State changes return true if modified, false if no change
//! - Q7: Operations are idempotent at boundaries
//!
//! ### Invariants
//! - I1: 0.1 <= zoom <= 4.0
//! - I2: camera_x is always finite
//! - I3: camera_y is always finite
//! - I4: Coordinate transforms are reversible
//! - I5: Viewport dimensions are positive

#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::suboptimal_flops)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::similar_names)]
#![allow(clippy::missing_const_for_fn)]
#![forbid(unsafe_code)]

mod fit;
mod operations;
mod state;
mod transform;
mod types;

pub use canvas_math::{MAX_ZOOM, MIN_ZOOM, ZOOM_IN_FACTOR, ZOOM_OUT_FACTOR};

pub use fit::*;
pub use operations::*;
pub use state::*;
pub use transform::*;
pub use types::*;

#[cfg(test)]
mod tests;
