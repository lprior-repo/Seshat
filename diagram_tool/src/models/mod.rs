//! Models module for diagram data structures
//!
//! Provides domain models for nodes, edges, documents, and related structures.
//!
//! ## Design by Contract
//!
//! ### Preconditions
//! - P1: Node positions must be finite (not NaN/Infinite)
//! - P2: Node dimensions must be positive (width > 0, height > 0)
//! - P3: Edge references must point to valid `NodeIds`
//! - P4: Document revision must be non-negative
//!
//! ### Postconditions
//! - Q1: NodeId/EdgeId newtypes wrap non-empty strings
//! - Q2: `OrderedFloat` maintains total ordering for collections
//! - Q3: Serialization roundtrips preserve document equality
//!
//! ### Invariants
//! - I1: Each Node has unique `NodeId`
//! - I2: Each Edge has unique `EdgeId`
//! - I3: Node positions in world coordinates (independent of viewport)
//! - I4: Document revision monotonically increases

pub mod canonical_json;
pub mod conflict;
pub mod dag;
pub mod document;
pub mod envelope;
#[cfg(not(target_arch = "wasm32"))]
pub mod export;
#[cfg(not(target_arch = "wasm32"))]
pub mod harness;
pub mod port;
pub mod projection;
pub mod schema;
pub mod schema_defs; // Single source of truth for SQLite schemas
pub mod selection;
pub mod multi_select;

#[cfg(test)]
pub mod multi_select_tests;

pub mod subgraph;

pub mod selection_ops;

#[cfg(test)]
pub mod selection_ops_tests;

pub mod transform;

#[cfg(test)]
pub mod transform_tests;

pub mod subgraph_events;
pub mod sync;
pub mod validation;

pub mod clipboard_contract;

#[cfg(test)]
pub mod clipboard_contract_tests;

#[cfg(test)]
pub mod subgraph_events_tests;

#[cfg(test)]
pub mod subgraph_persistence_tests;
