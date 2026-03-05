//! Mutation operations module
//!
//! Provides document mutation operations including layout calculations and transformations.
//!
//! ## Design by Contract
//!
//! ### Preconditions
//! - P1: Document must be valid (well-formed nodes and edges)
//! - P2: Cell size must be non-negative (0 allowed, uses defaults)
//! - P3: Pipeline operations require non-empty operation queue
//!
//! ### Postconditions
//! - Q1: Layout operations preserve node count
//! - Q2: Mutations return new document (immutable, original unchanged)
//! - Q3: Failed operations return error, never panic
//! - Q4: Pipeline processes operations in order
//!
//! ### Invariants
//! - I1: Node IDs remain stable across mutations
//! - I2: Edge references remain valid after layout
//! - I3: Document revision increments on each mutation

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

pub mod error;
pub mod ops;
pub mod pipeline;
