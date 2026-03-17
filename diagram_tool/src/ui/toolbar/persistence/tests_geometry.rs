#[allow(clippy::unwrap_used, clippy::expect_used)]
use diagram_models::document::{DiagramDocument, LockState, Node, NodeId, NodeKind, OrderedFloat};
use im::HashMap;

/// IO-TEST-3: Save/Reopen Exact Geometry (bd-1u1)
/// Given: A document with nodes at precise fractional coordinates
/// When: Saving to JSON and reopening
/// Then: All geometry values are exactly preserved
#[cfg(kani)]
#[kani::proof]
#[test]
fn given_document_with_fractional_coords_when_round_trip_then_geometry_preserved() {
    use crate::ui::toolbar::persistence_compat::parse_diagram_document_with_compat;
    use diagram_models::canonical_json::to_canonical_pretty_json;

    // Given - document with precise fractional coordinates
    let mut doc = DiagramDocument::default();
    let precise_x = 123.456789;
    let precise_y = 987.654321;
    let precise_width = 45.125;
    let precise_height = 67.875;

    let _ = doc.document.nodes.insert(
        NodeId::new("precise-node".to_string()),
        Node {
            kind: NodeKind::Text,
            icon: String::new(),
            label: String::from("Precise"),
            x: OrderedFloat(precise_x),
            y: OrderedFloat(precise_y),
            width: OrderedFloat(precise_width),
            height: OrderedFloat(precise_height),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        },
    );

    // When - serialize to JSON and reload
    let json = to_canonical_pretty_json(&doc).expect("serialization should succeed");
    let reloaded: DiagramDocument =
        parse_diagram_document_with_compat(&json).expect("parsing should succeed");

    // Then - geometry should be exactly preserved
    let reloaded_node = reloaded
        .document
        .nodes
        .get(&NodeId::new("precise-node".to_string()))
        .expect("node should exist");

    assert_eq!(
        reloaded_node.x.0, precise_x,
        "x coordinate should be exactly preserved"
    );
    assert_eq!(
        reloaded_node.y.0, precise_y,
        "y coordinate should be exactly preserved"
    );
    assert_eq!(
        reloaded_node.width.0, precise_width,
        "width should be exactly preserved"
    );
    assert_eq!(
        reloaded_node.height.0, precise_height,
        "height should be exactly preserved"
    );
}

/// IO-TEST-3b: Multiple nodes with various precision levels
#[cfg(kani)]
#[kani::proof]
#[test]
fn given_document_with_various_precision_coords_when_round_trip_then_all_preserved() {
    use crate::ui::toolbar::persistence_compat::parse_diagram_document_with_compat;
    use diagram_models::canonical_json::to_canonical_pretty_json;

    // Given
    let mut doc = DiagramDocument::default();

    // Test various precision levels
    let test_cases: [(&str, f64, f64, f64, f64); 5] = [
        ("integer", 100.0, 200.0, 50.0, 30.0),
        ("one_decimal", 100.5, 200.5, 50.5, 30.5),
        ("two_decimals", 100.25, 200.75, 50.25, 30.75),
        (
            "many_decimals",
            123.456789012,
            987.654321098,
            45.123456789,
            67.987654321,
        ),
        ("small_values", 0.001, 0.002, 0.5, 0.25),
    ];

    for (name, x, y, w, h) in test_cases {
        let _ = doc.document.nodes.insert(
            NodeId::new(name.to_string()),
            Node {
                kind: NodeKind::Text,
                icon: String::new(),
                label: name.to_string(),
                x: OrderedFloat(x),
                y: OrderedFloat(y),
                width: OrderedFloat(w),
                height: OrderedFloat(h),
                font_size: None,
                font_weight: None,
                lock_state: LockState::Unlocked,
                parent: None,
                dag_rank: None,
                tags: im::Vector::new(),
                metadata: HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            },
        );
    }

    // When
    let json = to_canonical_pretty_json(&doc).expect("serialization should succeed");
    let reloaded: DiagramDocument =
        parse_diagram_document_with_compat(&json).expect("parsing should succeed");

    // Then
    for (name, x, y, w, h) in test_cases {
        let node = reloaded
            .document
            .nodes
            .get(&NodeId::new(name.to_string()))
            .expect("node should exist");
        assert_eq!(node.x.0, x, "{name}: x should be preserved");
        assert_eq!(node.y.0, y, "{name}: y should be preserved");
        assert_eq!(node.width.0, w, "{name}: width should be preserved");
        assert_eq!(node.height.0, h, "{name}: height should be preserved");
    }
}

/// IO-TEST-4: Import Large Coordinates No Float Crash (bd-1u1)
/// Given: A JSON document with very large coordinate values
/// When: Importing the document
/// Then: Import succeeds without floating-point overflow/crash
#[cfg(kani)]
#[kani::proof]
#[test]
fn given_document_with_large_coordinates_when_import_then_succeeds() {
    use crate::ui::toolbar::persistence_compat::parse_diagram_document_with_compat;

    // Given - JSON with very large but finite coordinates
    let json = r#"{
        "version": 2,
        "revision": 0,
        "document": {
            "nodes": {
                "large_coord": {
                    "kind": "text",
                    "icon": "",
                    "label": "Large",
                    "x": 1e15,
                    "y": 1e15,
                    "width": 1000000000000.0,
                    "height": 500000000000.0,
                    "locked": false,
                    "parent": null,
                    "tags": [],
                    "metadata": {}
                }
            },
            "edges": {}
        },
        "editor_state": {
            "camera_x": 0,
            "camera_y": 0,
            "zoom": 1,
            "grid_size": 20,
            "snap_to_grid": true,
            "selected_items": []
        }
    }"#;

    // When
    let result = parse_diagram_document_with_compat(json);

    // Then - should parse without crash
    assert!(
        result.is_ok(),
        "Large coordinates should parse without crash: {:?}",
        result.err()
    );
    let doc = result.expect("should have document");
    let node = doc
        .document
        .nodes
        .get(&NodeId::new("large_coord".to_string()))
        .expect("node should exist");

    // Verify the large values are preserved
    assert!(node.x.0.is_finite(), "x should be finite");
    assert!(node.y.0.is_finite(), "y should be finite");
    assert!(node.x.0 > 1e14, "x should be very large");
}

/// IO-TEST-4b: Extreme but finite values
#[cfg(kani)]
#[kani::proof]
#[test]
fn given_document_with_extreme_finite_coords_when_import_then_succeeds() {
    use crate::ui::toolbar::persistence_compat::parse_diagram_document_with_compat;

    // Given - JSON with values near f64::MAX
    let large_value = 1e300_f64;
    let json = format!(
        r#"{{
        "version": 2,
        "revision": 0,
        "document": {{
            "nodes": {{
                "extreme": {{
                    "kind": "text",
                    "icon": "",
                    "label": "Extreme",
                    "x": {large_value:e},
                    "y": {large_value:e},
                    "width": 100.0,
                    "height": 50.0,
                    "locked": false,
                    "parent": null,
                    "tags": [],
                    "metadata": {{}}
                }}
            }},
            "edges": {{}}
        }},
        "editor_state": {{
            "camera_x": 0,
            "camera_y": 0,
            "zoom": 1,
            "grid_size": 20,
            "snap_to_grid": true,
            "selected_items": []
        }}
    }}"#
    );

    // When
    let result = parse_diagram_document_with_compat(&json);

    // Then - should parse without crash
    assert!(
        result.is_ok(),
        "Extreme coordinates should parse: {:?}",
        result.err()
    );
    let doc = result.expect("should have document");
    let node = doc
        .document
        .nodes
        .get(&NodeId::new("extreme".to_string()))
        .expect("node should exist");

    assert!(node.x.0.is_finite(), "x should be finite");
    assert!(node.y.0.is_finite(), "y should be finite");
}
