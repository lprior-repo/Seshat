//! Golden scene fixtures for snapshot-based testing
//!
//! This module provides canonical test fixtures for deterministic comparison
//! of move/resize/rotate/group/reparent operations across code changes.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![forbid(unsafe_code)]

mod golden_scenes {
    pub mod scene;
    pub mod stress_gen;
}

use golden_scenes::scene::Scene;
use golden_scenes::stress_gen::generate_stress_scene_json;
use serde_json::Value;
use std::collections::HashSet;

#[cfg_attr(kani, kani::proof)]
fn golden_mixed_selection_validity() {
    let scene = Scene::load("mixed_selection.json");
    assert_eq!(scene.version(), 2);
    assert_eq!(scene.revision(), 0);
    assert!(scene.nodes().len() >= 5);
    assert!(!scene.edges().is_empty());

    assert_eq!(scene.node("rect-1")["kind"].as_str(), Some("node"));
    assert_eq!(scene.node("rect-1")["style"].as_str(), Some("box"));
    assert_eq!(scene.node("ellipse-1")["kind"].as_str(), Some("node"));
    assert_eq!(scene.node("ellipse-1")["style"].as_str(), Some("cloud"));
    assert_eq!(scene.node("text-1")["kind"].as_str(), Some("text"));

    let arrow = scene.edge("arrow-1");
    assert_eq!(arrow["arrowType"].as_str(), Some("sharp"));
    assert_eq!(arrow["directed"].as_bool(), Some(true));

    scene.assert_unique_ids();
}

#[cfg_attr(kani, kani::proof)]
fn golden_nested_subgraph_validity() {
    let scene = Scene::load("nested_subgraph.json");
    assert_eq!(scene.version(), 2);

    let frame = scene.node("frame-1");
    assert_eq!(frame["kind"].as_str(), Some("subgraph"));
    assert!(frame["parent"].is_null());

    let group = scene.node("group-1");
    assert_eq!(group["kind"].as_str(), Some("subgraph"));
    assert_eq!(group["parent"].as_str(), Some("frame-1"));

    for id in ["shape-1", "shape-2", "shape-3"] {
        assert_eq!(scene.node(id)["kind"].as_str(), Some("node"));
        assert_eq!(scene.node(id)["parent"].as_str(), Some("group-1"));
    }

    let cross = scene.edge("edge-crossing");
    assert_eq!(cross["source"].as_str(), Some("shape-3"));
    assert_eq!(cross["target"].as_str(), Some("external-1"));

    // Parent tree cycle check
    let mut parents = std::collections::HashMap::new();
    for (id, node) in scene.nodes() {
        parents.insert(id.as_str(), node["parent"].as_str());
    }
    for id in scene.nodes().keys() {
        let mut visited = HashSet::new();
        let mut curr = Some(id.as_str());
        while let Some(n) = curr {
            assert!(visited.insert(n), "Cycle detected");
            curr = parents.get(n).and_then(|p| *p);
        }
    }
}

fn assert_operation(before_name: &str, after_name: &str, check: impl FnOnce(&Scene, &Scene)) {
    let before = Scene::load(before_name);
    let after = Scene::load(after_name);
    assert_eq!(
        before.revision() + 1,
        after.revision(),
        "Revision must increment"
    );
    check(&before, &after);
}

#[cfg_attr(kani, kani::proof)]
fn golden_operations_snapshots() {
    assert_operation("move_before.json", "move_after.json", |b, a| {
        assert_eq!(b.nodes().len(), a.nodes().len());
        assert_eq!(b.edges().len(), a.edges().len());
        let (nb, na) = (b.node("node-move-test"), a.node("node-move-test"));
        assert!((na["x"].as_f64().unwrap() - nb["x"].as_f64().unwrap() - 100.0).abs() < 0.001);
        assert!((na["y"].as_f64().unwrap() - nb["y"].as_f64().unwrap() - 50.0).abs() < 0.001);
        assert_eq!(nb["width"], na["width"]);
    });

    assert_operation("resize_before.json", "resize_after.json", |b, a| {
        assert_eq!(b.nodes().len(), a.nodes().len());
        let (nb, na) = (b.node("node-resize-test"), a.node("node-resize-test"));
        assert_eq!(nb["x"], na["x"]);
        assert!((na["width"].as_f64().unwrap() - 160.0).abs() < 0.001);
        assert!((na["height"].as_f64().unwrap() - 80.0).abs() < 0.001);
    });

    assert_operation("rotate_before.json", "rotate_after.json", |b, a| {
        let (nb, na) = (b.node("node-rotate-test"), a.node("node-rotate-test"));
        assert_eq!(nb["x"], na["x"]);
        let rb = nb["metadata"]["rotation"].as_f64().unwrap_or(0.0);
        let ra = na["metadata"]["rotation"].as_f64().unwrap_or(0.0);
        assert!((ra - 45.0).abs() < 0.001);
        assert!((rb - 0.0).abs() < 0.001);
    });

    assert_operation("group_before.json", "group_after.json", |b, a| {
        assert_eq!(b.nodes().len() + 1, a.nodes().len());
        assert_eq!(a.node("group-1")["kind"].as_str(), Some("subgraph"));
        for id in ["shape-a", "shape-b", "shape-c"] {
            assert_eq!(a.node(id)["parent"].as_str(), Some("group-1"));
        }
    });

    assert_operation("reparent_before.json", "reparent_after.json", |b, a| {
        assert_eq!(
            b.node("movable-node")["parent"].as_str(),
            Some("group-source")
        );
        assert_eq!(
            a.node("movable-node")["parent"].as_str(),
            Some("group-target")
        );
    });
}

#[cfg_attr(kani, kani::proof)]
fn golden_stress_scene_validity() {
    let doc = generate_stress_scene_json();
    let scene = Scene { doc: doc.clone() };
    assert!(scene.nodes().len() >= 5000);
    assert!(scene.edges().len() >= 5000);

    for edge in scene.edges().values() {
        assert!(scene.nodes().contains_key(edge["source"].as_str().unwrap()));
        assert!(scene.nodes().contains_key(edge["target"].as_str().unwrap()));
    }
    scene.assert_unique_ids();

    let json = serde_json::to_string(&doc).unwrap();
    assert!(json.len() < 50 * 1024 * 1024, "Too large");

    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(scene.nodes().len(), Scene { doc: parsed }.nodes().len());
}

#[cfg_attr(kani, kani::proof)]
fn all_fixtures_have_valid_schema() {
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
    for fix in fixtures {
        let scene = Scene::load(fix);
        assert!(scene.version() >= 2);
        if let Some(editor) = scene.doc.get("editor_state") {
            assert!(editor["zoom"].as_f64().unwrap_or(0.0) > 0.0);
            assert!(editor["grid_size"].as_f64().unwrap_or(0.0) > 0.0);
        }
    }
}
