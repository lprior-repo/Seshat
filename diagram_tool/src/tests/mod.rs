//! Category-Organized Test Suite for Seshat (240 tests, 12 categories)
//!
//! This module provides the top-level test structure organized by domain category.
//! Each sub-module corresponds to a TestCategory variant and contains test stubs
//! that will be populated with actual implementations by future beads.
//!
//! ## Test Category Map
//!
//! | Category | Code | Count | Module | Focus |
//! |----------|------|-------|--------|-------|
//! | Document | DOC  | 12    | doc_tests    | Serialization, schema, versioning, round-trip |
//! | Geometry | GEO  | 30    | geo_tests    | AABB, transforms, intersections, bounds |
//! | Selection| SEL  | 25    | sel_tests    | Click, marquee, multi-select, deselect |
//! | Multi    | MUL  | 37    | mul_tests    | Drag, rotate, scale, align, distribute |
//! | Subgraph | SUB  | 34    | sub_tests    | Create, destroy, reparent, cascade, transform |
//! | Edge     | EDG  | 35    | edg_tests    | Route, bend, label, port, direction |
//! | Viewport | CAM  | 12    | cam_tests    | Pan, zoom, fit, screen/canvas transforms |
//! | Snap     | SNP  | 10    | snp_tests    | Grid snap, alignment, distribution |
//! | Clipboard| CLP  | 10    | clp_tests    | Copy, paste, cut, cross-document |
//! | History  | HIS  | 13    | his_tests    | Undo, redo, branch, merge, eviction |
//! | IO       | IO   | 15    | io_tests     | Import, export, SVG, PNG, JSON schema |
//! | Perf     | PERF |  7    | perf_tests   | Large docs, rendering, memory, regression |
//!
//! Total: 240

#![cfg(test)]
#![allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]

pub mod cam_tests;
pub mod clp_tests;
pub mod doc_tests;
pub mod edg_tests;
pub mod geo_tests;
pub mod his_tests;
pub mod io_tests;
pub mod mul_tests;
pub mod perf_tests;
pub mod sel_tests;
pub mod snp_tests;
pub mod sub_tests;
