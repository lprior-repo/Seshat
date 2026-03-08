//! Golden scene fixtures for snapshot-based testing
//!
//! This module provides canonical test fixtures for deterministic comparison
//! of move/resize/rotate/group/reparent operations across code changes.
//!
//! ## Fixtures
//!
//! - `mixed_selection.json`: Heterogeneous element types (rect, ellipse, arrow, text, image)
//! - `nested_subgraph.json`: Hierarchical containment (frame > group > shapes)
//! - Stress scene: Large scene with 5000+ nodes/edges (generated programmatically)
//!
//! ## Operation Snapshots
//!
//! - `move_before.json` / `move_after.json`: Single node moved by (100, 50)
//! - `resize_before.json` / `resize_after.json`: Node scaled from 80x40 to 160x80
//! - `rotate_before.json` / `rotate_after.json`: Node rotated 45 degrees
//! - `group_before.json` / `group_after.json`: 3 nodes combined into subgraph
//! - `reparent_before.json` / `reparent_after.json`: Node moved between subgraphs

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![forbid(unsafe_code)]

use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

/// Fixture directory path
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Load a fixture file as JSON
fn load_fixture(name: &str) -> Value {
    let path = fixtures_dir().join(name);
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read fixture '{}': {}", name, e));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse fixture '{}' as JSON: {}", name, e))
}

/// Get nodes from a document
fn get_nodes(doc: &Value) -> &serde_json::Map<String, Value> {
    doc["document"]["nodes"].as_object().unwrap()
}

/// Get edges from a document
fn get_edges(doc: &Value) -> &serde_json::Map<String, Value> {
    doc["document"]["edges"].as_object().unwrap()
}

// ============================================================================
// Mixed Selection Scene Tests
// ============================================================================

#[test]
fn golden_mixed_selection_loads_successfully() {
    let doc = load_fixture("mixed_selection.json");
    assert_eq!(doc["version"].as_u64(), Some(2));
    assert_eq!(doc["revision"].as_u64(), Some(0));
}

#[test]
fn golden_mixed_selection_has_required_elements() {
    let doc = load_fixture("mixed_selection.json");
    let nodes = get_nodes(&doc);
    let edges = get_edges(&doc);

    // Verify node count (5 nodes minimum)
    assert!(
        nodes.len() >= 5,
        "Expected at least 5 nodes, got {}",
        nodes.len()
    );

    // Verify edge count (1 edge minimum)
    assert!(
        edges.len() >= 1,
        "Expected at least 1 edge, got {}",
        edges.len()
    );
}

#[test]
fn golden_mixed_selection_has_rectangle() {
    let doc = load_fixture("mixed_selection.json");
    let nodes = get_nodes(&doc);

    let rect = nodes.get("rect-1").expect("Missing rect-1 node");
    assert_eq!(rect["kind"].as_str(), Some("node"));
    assert_eq!(rect["style"].as_str(), Some("box"));
}

#[test]
fn golden_mixed_selection_has_ellipse() {
    let doc = load_fixture("mixed_selection.json");
    let nodes = get_nodes(&doc);

    let ellipse = nodes.get("ellipse-1").expect("Missing ellipse-1 node");
    assert_eq!(ellipse["kind"].as_str(), Some("node"));
    assert_eq!(ellipse["style"].as_str(), Some("cloud"));
}

#[test]
fn golden_mixed_selection_has_text() {
    let doc = load_fixture("mixed_selection.json");
    let nodes = get_nodes(&doc);

    let text = nodes.get("text-1").expect("Missing text-1 node");
    assert_eq!(text["kind"].as_str(), Some("text"));
}

#[test]
fn golden_mixed_selection_has_arrow_edge() {
    let doc = load_fixture("mixed_selection.json");
    let edges = get_edges(&doc);

    let arrow = edges.get("arrow-1").expect("Missing arrow-1 edge");
    assert_eq!(arrow["arrowType"].as_str(), Some("sharp"));
    assert_eq!(arrow["directed"].as_bool(), Some(true));
}

#[test]
fn golden_mixed_selection_all_ids_unique() {
    let doc = load_fixture("mixed_selection.json");
    let nodes = get_nodes(&doc);
    let edges = get_edges(&doc);

    // Check node ID uniqueness
    let node_ids: Vec<_> = nodes.keys().collect();
    let unique_node_ids: HashSet<_> = node_ids.iter().collect();
    assert_eq!(
        node_ids.len(),
        unique_node_ids.len(),
        "Duplicate node IDs found"
    );

    // Check edge ID uniqueness
    let edge_ids: Vec<_> = edges.keys().collect();
    let unique_edge_ids: HashSet<_> = edge_ids.iter().collect();
    assert_eq!(
        edge_ids.len(),
        unique_edge_ids.len(),
        "Duplicate edge IDs found"
    );
}

// ============================================================================
// Nested Subgraph Scene Tests
// ============================================================================

#[test]
fn golden_nested_subgraph_loads_successfully() {
    let doc = load_fixture("nested_subgraph.json");
    assert_eq!(doc["version"].as_u64(), Some(2));
}

#[test]
fn golden_nested_subgraph_has_frame() {
    let doc = load_fixture("nested_subgraph.json");
    let nodes = get_nodes(&doc);

    let frame = nodes.get("frame-1").expect("Missing frame-1");
    assert_eq!(frame["kind"].as_str(), Some("subgraph"));
    assert!(frame["parent"].is_null(), "Frame should not have a parent");
}

#[test]
fn golden_nested_subgraph_has_group_nested_in_frame() {
    let doc = load_fixture("nested_subgraph.json");
    let nodes = get_nodes(&doc);

    let group = nodes.get("group-1").expect("Missing group-1");
    assert_eq!(group["kind"].as_str(), Some("subgraph"));
    assert_eq!(group["parent"].as_str(), Some("frame-1"));
}

#[test]
fn golden_nested_subgraph_shapes_nested_in_group() {
    let doc = load_fixture("nested_subgraph.json");
    let nodes = get_nodes(&doc);

    for shape_id in &["shape-1", "shape-2", "shape-3"] {
        let shape = nodes
            .get(*shape_id)
            .unwrap_or_else(|| panic!("Missing {}", shape_id));
        assert_eq!(shape["kind"].as_str(), Some("node"));
        assert_eq!(shape["parent"].as_str(), Some("group-1"));
    }
}

#[test]
fn golden_nested_subgraph_has_crossing_edge() {
    let doc = load_fixture("nested_subgraph.json");
    let edges = get_edges(&doc);

    let edge = edges.get("edge-crossing").expect("Missing edge-crossing");
    assert_eq!(edge["source"].as_str(), Some("shape-3"));
    assert_eq!(edge["target"].as_str(), Some("external-1"));
}

#[test]
fn golden_nested_subgraph_parent_tree_valid() {
    let doc = load_fixture("nested_subgraph.json");
    let nodes = get_nodes(&doc);

    // Build parent map
    let mut parent_map: std::collections::HashMap<&str, Option<&str>> =
        std::collections::HashMap::new();
    for (id, node) in nodes {
        let parent = node["parent"].as_str();
        parent_map.insert(id, parent);
    }

    // Verify no cycles
    for id in nodes.keys() {
        let mut visited = HashSet::new();
        let mut current: Option<&str> = Some(id);

        while let Some(node_id) = current {
            assert!(visited.insert(node_id), "Cycle detected in parent tree");
            current = parent_map.get(node_id).and_then(|p| *p);
        }
    }
}

// ============================================================================
// Operation Snapshot Tests: Move
// ============================================================================

#[test]
fn golden_move_snapshot_delta_is_single_move() {
    let before = load_fixture("move_before.json");
    let after = load_fixture("move_after.json");

    // Revision should increment by 1
    assert_eq!(
        before["revision"].as_u64().unwrap() + 1,
        after["revision"].as_u64().unwrap()
    );

    // Same node count
    assert_eq!(get_nodes(&before).len(), get_nodes(&after).len());

    // Same edge count
    assert_eq!(get_edges(&before).len(), get_edges(&after).len());

    // Find the moved node
    let node_before = get_nodes(&before).get("node-move-test").unwrap();
    let node_after = get_nodes(&after).get("node-move-test").unwrap();

    // Position delta should be (100, 50)
    let x_before = node_before["x"].as_f64().unwrap();
    let y_before = node_before["y"].as_f64().unwrap();
    let x_after = node_after["x"].as_f64().unwrap();
    let y_after = node_after["y"].as_f64().unwrap();

    let dx = x_after - x_before;
    let dy = y_after - y_before;
    assert!((dx - 100.0).abs() < 0.001, "Expected dx=100, got {}", dx);
    assert!((dy - 50.0).abs() < 0.001, "Expected dy=50, got {}", dy);

    // Dimensions should be unchanged
    assert_eq!(node_before["width"], node_after["width"]);
    assert_eq!(node_before["height"], node_after["height"]);
}

// ============================================================================
// Operation Snapshot Tests: Resize
// ============================================================================

#[test]
fn golden_resize_snapshot_delta_is_single_resize() {
    let before = load_fixture("resize_before.json");
    let after = load_fixture("resize_after.json");

    // Revision should increment by 1
    assert_eq!(
        before["revision"].as_u64().unwrap() + 1,
        after["revision"].as_u64().unwrap()
    );

    // Same node count
    assert_eq!(get_nodes(&before).len(), get_nodes(&after).len());

    let node_before = get_nodes(&before).get("node-resize-test").unwrap();
    let node_after = get_nodes(&after).get("node-resize-test").unwrap();

    // Position should be unchanged
    assert_eq!(node_before["x"], node_after["x"]);
    assert_eq!(node_before["y"], node_after["y"]);

    // Size should double (80x40 -> 160x80)
    let width_after = node_after["width"].as_f64().unwrap();
    let height_after = node_after["height"].as_f64().unwrap();
    assert!(
        (width_after - 160.0).abs() < 0.001,
        "Expected width=160, got {}",
        width_after
    );
    assert!(
        (height_after - 80.0).abs() < 0.001,
        "Expected height=80, got {}",
        height_after
    );
}

// ============================================================================
// Operation Snapshot Tests: Rotate
// ============================================================================

#[test]
fn golden_rotate_snapshot_delta_is_single_rotation() {
    let before = load_fixture("rotate_before.json");
    let after = load_fixture("rotate_after.json");

    // Revision should increment by 1
    assert_eq!(
        before["revision"].as_u64().unwrap() + 1,
        after["revision"].as_u64().unwrap()
    );

    let node_before = get_nodes(&before).get("node-rotate-test").unwrap();
    let node_after = get_nodes(&after).get("node-rotate-test").unwrap();

    // Position and dimensions should be unchanged
    assert_eq!(node_before["x"], node_after["x"]);
    assert_eq!(node_before["y"], node_after["y"]);
    assert_eq!(node_before["width"], node_after["width"]);
    assert_eq!(node_before["height"], node_after["height"]);

    // Rotation metadata should change from 0 to 45
    let rot_before = node_before["metadata"]["rotation"].as_f64().unwrap_or(0.0);
    let rot_after = node_after["metadata"]["rotation"].as_f64().unwrap_or(0.0);

    assert!(
        (rot_before - 0.0).abs() < 0.001,
        "Expected before rotation=0"
    );
    assert!(
        (rot_after - 45.0).abs() < 0.001,
        "Expected after rotation=45"
    );
}

// ============================================================================
// Operation Snapshot Tests: Group
// ============================================================================

#[test]
fn golden_group_snapshot_creates_subgraph() {
    let before = load_fixture("group_before.json");
    let after = load_fixture("group_after.json");

    // Revision should increment by 1
    assert_eq!(
        before["revision"].as_u64().unwrap() + 1,
        after["revision"].as_u64().unwrap()
    );

    // After should have one more node (the group)
    assert_eq!(get_nodes(&before).len() + 1, get_nodes(&after).len());

    // Verify group exists
    let group = get_nodes(&after)
        .get("group-1")
        .expect("Group should exist after operation");
    assert_eq!(group["kind"].as_str(), Some("subgraph"));
}

#[test]
fn golden_group_snapshot_assigns_parent_references() {
    let after = load_fixture("group_after.json");
    let nodes = get_nodes(&after);

    // All three shapes should have parent = group-1
    for shape_id in &["shape-a", "shape-b", "shape-c"] {
        let shape = nodes.get(*shape_id).unwrap();
        assert_eq!(shape["parent"].as_str(), Some("group-1"));
    }
}

// ============================================================================
// Operation Snapshot Tests: Reparent
// ============================================================================

#[test]
fn golden_reparent_snapshot_changes_parent() {
    let before = load_fixture("reparent_before.json");
    let after = load_fixture("reparent_after.json");

    // Revision should increment by 1
    assert_eq!(
        before["revision"].as_u64().unwrap() + 1,
        after["revision"].as_u64().unwrap()
    );

    // Same node count
    assert_eq!(get_nodes(&before).len(), get_nodes(&after).len());

    let node_before = get_nodes(&before).get("movable-node").unwrap();
    let node_after = get_nodes(&after).get("movable-node").unwrap();

    // Parent should change
    assert_eq!(node_before["parent"].as_str(), Some("group-source"));
    assert_eq!(node_after["parent"].as_str(), Some("group-target"));
}

// ============================================================================
// Stress Scene Tests (Programmatic Generation)
// ============================================================================

/// Generate stress scene JSON with 5000+ nodes and edges
fn generate_stress_scene_json() -> Value {
    let node_count: usize = 5000;
    let edge_count: usize = 5000;

    let mut nodes = serde_json::Map::new();
    let mut edges = serde_json::Map::new();

    // Use seeded random for determinism
    let mut rng_state: u64 = 12345;

    let mut next_random = || -> f64 {
        // Simple LCG random number generator for determinism
        rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
        let normalized = ((rng_state >> 16) & 0xFFFF) as f64 / 65535.0;
        normalized
    };

    // Collect node IDs for edge generation
    let mut node_ids: Vec<String> = Vec::with_capacity(node_count);

    // Generate nodes
    for i in 0..node_count {
        let id = format!("stress-node-{}", i);
        node_ids.push(id.clone());

        // 80% Node, 15% Text, 5% Subgraph
        let kind_roll = next_random();
        let kind = if kind_roll < 0.80 {
            "node"
        } else if kind_roll < 0.95 {
            "text"
        } else {
            "subgraph"
        };

        let x = next_random() * 5000.0;
        let y = next_random() * 5000.0;
        let width = 80.0 + next_random() * 40.0;
        let height = 40.0 + next_random() * 20.0;

        let node = serde_json::json!({
            "kind": kind,
            "icon": "",
            "label": format!("Node {}", i),
            "x": x,
            "y": y,
            "width": width,
            "height": height,
            "locked": false,
            "parent": null,
            "tags": [],
            "metadata": {},
            "z_index": i as i64
        });

        nodes.insert(id, node);
    }

    // Generate edges (ensure valid source/target)
    for i in 0..edge_count {
        let edge_id = format!("stress-edge-{}", i);

        let source_idx = (next_random() * node_count as f64) as usize;
        let mut target_idx = (next_random() * node_count as f64) as usize;

        // Avoid self-loops for most edges
        if source_idx == target_idx {
            target_idx = (target_idx + 1) % node_count;
        }

        let edge = serde_json::json!({
            "source": node_ids[source_idx],
            "target": node_ids[target_idx],
            "label": "",
            "style": "solid",
            "arrowType": "default",
            "label_offset_t": 0.5,
            "thickness": 1.5,
            "directed": true,
            "bend_points": [],
            "tags": [],
            "metadata": {}
        });

        edges.insert(edge_id, edge);
    }

    serde_json::json!({
        "version": 2,
        "revision": 0,
        "document": {
            "nodes": nodes,
            "edges": edges
        },
        "editor_state": {
            "camera_x": 0.0,
            "camera_y": 0.0,
            "zoom": 0.5,
            "grid_size": 20.0,
            "snap_to_grid": true,
            "selected_items": [],
            "editing_edge_id": null,
            "theme": "system",
            "show_grid": true,
            "minimap_visible": true
        }
    })
}

#[test]
fn golden_stress_scene_generates_5000_nodes() {
    let doc = generate_stress_scene_json();
    let nodes = get_nodes(&doc);
    assert!(
        nodes.len() >= 5000,
        "Expected 5000+ nodes, got {}",
        nodes.len()
    );
}

#[test]
fn golden_stress_scene_generates_5000_edges() {
    let doc = generate_stress_scene_json();
    let edges = get_edges(&doc);
    assert!(
        edges.len() >= 5000,
        "Expected 5000+ edges, got {}",
        edges.len()
    );
}

#[test]
fn golden_stress_scene_all_edges_valid() {
    let doc = generate_stress_scene_json();
    let nodes = get_nodes(&doc);
    let edges = get_edges(&doc);

    for (edge_id, edge) in edges {
        let source = edge["source"].as_str().expect("Edge missing source");
        let target = edge["target"].as_str().expect("Edge missing target");

        assert!(
            nodes.contains_key(source),
            "Edge {} has invalid source {}",
            edge_id,
            source
        );
        assert!(
            nodes.contains_key(target),
            "Edge {} has invalid target {}",
            edge_id,
            target
        );
    }
}

#[test]
fn golden_stress_scene_no_duplicate_ids() {
    let doc = generate_stress_scene_json();
    let nodes = get_nodes(&doc);
    let edges = get_edges(&doc);

    // Check node IDs
    let node_count = nodes.len();
    let node_ids: HashSet<_> = nodes.keys().collect();
    assert_eq!(node_count, node_ids.len(), "Duplicate node IDs detected");

    // Check edge IDs
    let edge_count = edges.len();
    let edge_ids: HashSet<_> = edges.keys().collect();
    assert_eq!(edge_count, edge_ids.len(), "Duplicate edge IDs detected");
}

#[test]
fn golden_stress_scene_serializes_reasonably() {
    let doc = generate_stress_scene_json();

    let json = serde_json::to_string(&doc).expect("Failed to serialize stress scene");
    let size_bytes = json.len();

    // Should be less than 50MB
    assert!(
        size_bytes < 50 * 1024 * 1024,
        "Stress scene too large: {} bytes",
        size_bytes
    );
}

#[test]
fn golden_stress_scene_roundtrips() {
    let doc = generate_stress_scene_json();

    let json = serde_json::to_string(&doc).expect("Failed to serialize");
    let parsed: Value = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(get_nodes(&doc).len(), get_nodes(&parsed).len());
    assert_eq!(get_edges(&doc).len(), get_edges(&parsed).len());
}

// ============================================================================
// Schema Validation Tests
// ============================================================================

#[test]
fn all_fixtures_have_valid_schema_version() {
    let fixtures = [
        "mixed_selection.json",
        "nested_subgraph.json",
        "move_before.json",
        "move_after.json",
        "resize_before.json",
        "resize_after.json",
        "rotate_before.json",
        "rotate_after.json",
        "group_before.json",
        "group_after.json",
        "reparent_before.json",
        "reparent_after.json",
    ];

    for fixture in fixtures {
        let doc = load_fixture(fixture);
        let version = doc["version"].as_u64().unwrap_or(0);
        assert!(
            version >= 2,
            "Fixture {} has invalid version {}",
            fixture,
            version
        );
    }
}

#[test]
fn all_fixtures_have_valid_editor_state() {
    let fixtures = [
        "mixed_selection.json",
        "nested_subgraph.json",
        "move_before.json",
        "resize_before.json",
        "rotate_before.json",
        "group_before.json",
        "reparent_before.json",
    ];

    for fixture in fixtures {
        let doc = load_fixture(fixture);

        // Editor state should have valid zoom
        let zoom = doc["editor_state"]["zoom"].as_f64().unwrap_or(0.0);
        assert!(zoom > 0.0, "Invalid zoom in {}", fixture);

        // Grid size should be positive
        let grid_size = doc["editor_state"]["grid_size"].as_f64().unwrap_or(0.0);
        assert!(grid_size > 0.0, "Invalid grid size in {}", fixture);
    }
}
