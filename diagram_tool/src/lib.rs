//! Diagram Tool Library
//!
//! This library exposes two categories of APIs:
//!
//! ## Stable Public APIs (pub)
//! These modules form the stable public interface and are safe for external consumers:
//! - `app` - Application orchestration and state management
//! - `models` - Core data structures and types
//! - `mutation` - State mutation operations
//! - `viewport` - Viewport/canvas management
//! - `store` - State storage and retrieval
//! - `export` - Diagram export functionality
//! - `history` - Undo/redo history management
//! - `geometry` - Geometric calculations and types
//! - `layout` - Layout algorithms
//! - `cli` - Command-line interface
//! - `cli_persistence` - CLI data persistence
//!
//! ## Internal APIs (pub(crate))
//! These modules are for internal use only and may change without notice:
//! - `backend` - Internal backend implementation details
//! - `hooks` - Internal React hooks implementation
//! - `icons` - Internal icon assets
//! - `perf` - Internal performance utilities
//! - `ui` - Internal UI components and rendering
//!
//! # Public API
//!
//! This section documents the stable public API surface. Consumers can depend on
//! these types and functions. Internal modules are marked with `pub(crate)`.
//!
//! ## Core Types
//! - `DiagramDocument` - The root document type representing a diagram
//! - `NodeId`, `EdgeId` - Newtype identifiers for nodes and edges
//! - `Revision` - Document revision counter for optimistic concurrency
//! - `DocumentData` - Container for nodes and edges
//! - `Node` - Individual node with position, size, style, and content
//! - `Edge` - Connection between nodes with style and arrow configuration
//! - `NodeKind` - Enum for node types (rectangle, ellipse, diamond, etc.)
//! - `NodeStyle` - Styling configuration for nodes (fill, stroke, etc.)
//! - `EdgeStyle` - Styling for edges (solid, dashed, etc.)
//! - `ArrowType` - Arrow head types for edges
//! - `Point` - 2D point with x, y coordinates
//! - `AABB` - Axis-aligned bounding box
//! - `Rectangle` - Rectangle with position and dimensions
//! - `EditorState` - Current editor state (selection, tool mode, etc.)
//! - `EditorTheme` - Visual theme configuration
//! - `DiagramProjection` - Projection state for CRDT-based collaboration
//! - `ValidationIssue` - Validation error with severity and message
//!
//! ## Key Functions
//! - `validate_document()` - Pure function to validate diagram documents
//! - `validate_schema()` - Validate document against schema
//! - `apply_layout()` - Apply grid layout to non-locked nodes (pure function)
//! - `calculate_grid_layout()` - Calculate grid-based node positions
//! - `dag_layout()` - Apply DAG (directed acyclic graph) layout algorithm
//! - `snap_to_grid()` - Snap point coordinates to grid
//! - `snap_to_guides()` - Snap to alignment guides
//! - `snap_to_nodes()` - Snap to nearby node edges/centers
//! - `align_left()`, `align_center()`, `align_right()` - Align nodes horizontally
//! - `align_top()`, `align_middle()`, `align_bottom()` - Align nodes vertically
//! - `distribute_horizontally()`, `distribute_vertically()` - Even node distribution
//! - `scale_around_anchor()` - Scale point around an anchor
//! - `rotate_around_center()` - Rotate point around center
//! - `replay_events()` - Replay event stream to build projection
//! - `projection_to_document()` - Convert projection to document
//! - `document_to_projection()` - Convert document to projection
//! - `export_diagram_json()` - Export diagram to JSON format
//! - `import_diagram_json()` - Import diagram from JSON
//! - `write_snapshot()` - Persist snapshot to storage
//! - `load_projection()` - Load projection from storage
//! - `start_store_watcher()` - Start file system watcher for changes
//!
//! ## Modules
//! - `models` - Domain types (DiagramDocument, NodeId, EdgeId, Revision, etc.)
//!   and document model with validation, schema, and export functionality
//! - `mutation` - Document mutation operations including `apply_layout()`
//! - `layout` - Layout algorithms (grid layout, DAG layout)
//! - `geometry` - Geometric types (Point, Rectangle, AABB) and alignment functions
//! - `store` - State storage and retrieval
//! - `viewport` - Viewport/canvas management (transform, pan, zoom)
//! - `history` - Undo/redo history management
//! - `export` - Diagram export functionality (JSON, PNG, SVG)
//! - `app` - Application orchestration and state management
//! - `cli` - Command-line interface
//! - `cli_persistence` - CLI data persistence layer

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

// Stable public APIs
pub mod app;
pub mod cli;
pub mod cli_persistence;
pub mod export;
pub mod geometry;
pub mod history;
pub mod layout;
pub mod models;
pub mod mutation;
pub mod store;
pub mod viewport;

// Internal APIs - for crate-internal use only
pub(crate) mod backend;
pub(crate) mod hooks;
pub(crate) mod icons;
pub(crate) mod perf;
pub(crate) mod ui;

#[allow(dead_code)]
mod internal {
    // Placeholder for truly internal code that should never be exposed
    // This module can contain helper functions, constants, or types
    // that are only used within the crate and have no public interface
}
