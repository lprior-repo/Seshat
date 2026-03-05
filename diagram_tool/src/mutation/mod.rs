//! Mutation module for diagram operations
//!
//! This module provides mutation operations for diagram documents including
//! layout calculations and error handling for schema and validation issues.
//!
//! ## Design by Contract
//!
//! ### Preconditions
//! - P1: For `apply_layout`: document must be a valid `DiagramDocument`
//! - P2: `cell_size` must be a finite number (can be zero or negative)
//! - P3: Document may contain nodes with parent cycles (handled gracefully)
//! - P4: Locked nodes should not be modified by layout operations
//!
//! ### Postconditions
//! - Q1: `apply_layout` returns a new `DiagramDocument` (immutable, pure function)
//! - Q2: Non-locked nodes have positions recalculated based on grid layout
//! - Q3: Locked nodes preserve their original positions unchanged
//! - Q4: All nodes present in input are present in output
//! - Q5: Parent-child relationships are preserved
//!
//! ### Invariants
//! - I1: Output document has same node count as input
//! - I2: Node IDs are preserved (same set of keys)
//! - I3: Locked nodes have unchanged x, y coordinates after layout
//! - I4: Finite input coordinates produce finite output coordinates
//!
//! ## Error Types
//!
//! ### MutationError
//! - `Schema(String)` - Structural errors in document format
//! - `Semantic(String)` - Validation errors (e.g., invalid positions, missing required fields)
//!
//! ## Operations
//!
//! ### apply_layout
//! Applies a grid layout algorithm to reposition non-locked nodes in a diagram.
//! Locked nodes maintain their positions. The function is pure and returns a new document.
//!
//! # Arguments
//! * `doc` - The input diagram document
//! * `cell_size` - Grid cell size for layout calculation
//!
//! # Returns
//! A new `DiagramDocument` with repositioned non-locked nodes

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

pub mod error;
pub mod ops;
pub mod pipeline;
