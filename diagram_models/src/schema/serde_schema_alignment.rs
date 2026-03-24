// Test module: unwrap/expect/panic are standard test assertion patterns.
// boon_smoke_tests.rs demonstrates the stricter Result<(), String> pattern.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::module_name_repetitions
)]

use boon::{Compiler, Schemas};
use im::HashMap;
use serde_json::{json, Value};

use crate::document::*;
use crate::port::{NormalizedOffset, PortAnchor};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const SCHEMA_JSON: &str = include_str!("../diagram.schema.json");

fn load_schema_value() -> Value {
    serde_json::from_str(SCHEMA_JSON).expect("diagram.schema.json must be valid JSON")
}

fn compile_diagram_schema() -> (Schemas, boon::SchemaIndex) {
    let schema = load_schema_value();
    let mut schemas = Schemas::new();
    let mut compiler = Compiler::new();
    compiler
        .add_resource("diagram.schema.json", schema)
        .expect("add_resource must succeed");
    let sch_index = compiler
        .compile("diagram.schema.json", &mut schemas)
        .expect("compile must succeed");
    (schemas, sch_index)
}

macro_rules! assert_boon_valid {
    ($result:expr, $msg:expr) => {
        match $result {
            Ok(()) => {}
            Err(e) => panic!("{}: validation error: {}", $msg, e),
        }
    };
}

fn compile_schema_def_as_root(def_name: &str) -> (Schemas, boon::SchemaIndex) {
    let schema = load_schema_value();
    let defs = schema.get("$defs").expect("$defs must exist").clone();
    let wrapper = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$ref": format!("#/$defs/{def_name}"),
        "$defs": defs
    });
    let mut schemas = Schemas::new();
    let mut compiler = Compiler::new();
    compiler
        .add_resource("def-schema.json", wrapper)
        .expect("add_resource must succeed");
    let sch_index = compiler
        .compile("def-schema.json", &mut schemas)
        .expect("compile must succeed");
    (schemas, sch_index)
}

fn make_node_full(label: &str) -> Node {
    Node {
        kind: NodeKind::Subgraph,
        icon: "icon/subgraph".to_string(),
        label: label.to_string(),
        x: OrderedFloat::new_unchecked(100.0),
        y: OrderedFloat::new_unchecked(200.0),
        width: OrderedFloat::new_unchecked(300.0),
        height: OrderedFloat::new_unchecked(200.0),
        font_size: Some(OrderedFloat::new_unchecked(12.0)),
        font_weight: Some(FontWeight::Bold),
        lock_state: LockState::Locked,
        parent: None,
        dag_rank: Some(3),
        tags: im::vector!["tag-a".to_string(), "tag-b".to_string()],
        metadata: {
            let mut m = HashMap::new();
            m.insert("key".to_string(), json!("value"));
            m
        },
        z_index: 5,
        style: Some(NodeStyle::Cloud),
        collapsed: Some(true),
    }
}

fn make_node_minimal(label: &str) -> Node {
    Node {
        kind: NodeKind::Node,
        icon: String::new(),
        label: label.to_string(),
        x: OrderedFloat::new_unchecked(0.0),
        y: OrderedFloat::new_unchecked(0.0),
        width: OrderedFloat::new_unchecked(100.0),
        height: OrderedFloat::new_unchecked(60.0),
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
    }
}

fn make_edge_full(source: &str, target: &str) -> Edge {
    Edge {
        source: NodeId::new(source.to_string()),
        target: NodeId::new(target.to_string()),
        label: "label".to_string(),
        style: EdgeStyle::Dashed,
        arrow_type: ArrowType::Step,
        label_offset_t: OrderedFloat::new_unchecked(0.25),
        color: Some("#ff0000".to_string()),
        thickness: OrderedFloat::new_unchecked(2.5),
        directed: true,
        bend_points: im::vector![SerializedPoint {
            x: OrderedFloat::new_unchecked(10.0),
            y: OrderedFloat::new_unchecked(20.0),
        }],
        tags: im::vector!["tag".to_string()],
        metadata: {
            let mut m = HashMap::new();
            m.insert("key".to_string(), json!("value"));
            m
        },
        font_size: Some(OrderedFloat::new_unchecked(14.0)),
        source_port: Some(PortAnchor::Bottom),
        target_port: Some(PortAnchor::Custom(NormalizedOffset {
            x: OrderedFloat::new_unchecked(0.25),
            y: OrderedFloat::new_unchecked(0.75),
        })),
    }
}

fn make_edge_minimal(source: &str, target: &str) -> Edge {
    Edge {
        source: NodeId::new(source.to_string()),
        target: NodeId::new(target.to_string()),
        label: String::new(),
        style: EdgeStyle::Solid,
        arrow_type: ArrowType::Default,
        label_offset_t: OrderedFloat::new_unchecked(0.5),
        color: None,
        thickness: OrderedFloat::new_unchecked(1.5),
        directed: true,
        bend_points: im::Vector::new(),
        tags: im::Vector::new(),
        metadata: HashMap::new(),
        font_size: None,
        source_port: None,
        target_port: None,
    }
}

fn make_editor_state_full() -> EditorState {
    EditorState {
        camera_x: OrderedFloat::new_unchecked(50.0),
        camera_y: OrderedFloat::new_unchecked(100.0),
        zoom: OrderedFloat::new_unchecked(2.0),
        grid_size: GridSize::new(40.0).expect("valid grid size"),
        snap_to_grid: false,
        selected_items: im::hashset!["n1".to_string(), "n2".to_string()],
        edit_mode_target: Some("n1".to_string()),
        editing_edge_id: Some("e1".to_string()),
        theme: EditorTheme::Dark,
        show_grid: false,
        minimap_visible: true,
    }
}

fn make_minimal_document() -> DiagramDocument {
    DiagramDocument {
        version: 2,
        revision: Revision::INITIAL,
        document: DocumentData {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        },
        editor_state: EditorState::default(),
    }
}

fn make_full_document() -> DiagramDocument {
    let n1 = NodeId::new("n1".to_string());
    let n2 = NodeId::new("n2".to_string());
    let e1 = EdgeId::new("e1".to_string());
    DiagramDocument {
        version: 2,
        revision: Revision::new(5),
        document: DocumentData {
            nodes: HashMap::new()
                .update(n1.clone(), make_node_full("Node 1"))
                .update(n2.clone(), make_node_minimal("Node 2")),
            edges: HashMap::new().update(e1, make_edge_full("n1", "n2")),
        },
        editor_state: make_editor_state_full(),
    }
}

// ===================================================================
// B01-B07: Field Name Alignment (unit)
// ===================================================================

#[test]
fn schema_node_uses_fontSize_key_not_font_size() {
    let schema = load_schema_value();
    let node_props = schema
        .pointer("/$defs/node/properties")
        .expect("$defs/node/properties must exist");
    assert!(
        node_props.get("fontSize").is_some(),
        "Node properties must contain 'fontSize'"
    );
    assert!(
        node_props.get("font_size").is_none(),
        "Node properties must NOT contain 'font_size'"
    );
}

#[test]
fn schema_edge_uses_arrowType_key_not_arrowhead() {
    let schema = load_schema_value();
    let edge_props = schema
        .pointer("/$defs/edge/properties")
        .expect("$defs/edge/properties must exist");
    assert!(
        edge_props.get("arrowType").is_some(),
        "Edge properties must contain 'arrowType'"
    );
    assert!(
        edge_props.get("arrowhead").is_none(),
        "Edge properties must NOT contain 'arrowhead'"
    );
}

#[test]
fn schema_edge_uses_fontSize_key_not_font_size() {
    let schema = load_schema_value();
    let edge_props = schema
        .pointer("/$defs/edge/properties")
        .expect("$defs/edge/properties must exist");
    assert!(
        edge_props.get("fontSize").is_some(),
        "Edge properties must contain 'fontSize'"
    );
    assert!(
        edge_props.get("font_size").is_none(),
        "Edge properties must NOT contain 'font_size'"
    );
}

#[test]
fn schema_edge_contains_source_port_property() {
    let schema = load_schema_value();
    let edge_props = schema
        .pointer("/$defs/edge/properties")
        .expect("$defs/edge/properties must exist");
    assert!(
        edge_props.get("source_port").is_some(),
        "Edge properties must contain 'source_port'"
    );
}

#[test]
fn schema_edge_contains_target_port_property() {
    let schema = load_schema_value();
    let edge_props = schema
        .pointer("/$defs/edge/properties")
        .expect("$defs/edge/properties must exist");
    assert!(
        edge_props.get("target_port").is_some(),
        "Edge properties must contain 'target_port'"
    );
}

#[test]
fn schema_editor_state_contains_edit_mode_target_property() {
    let schema = load_schema_value();
    let es_props = schema
        .pointer("/$defs/editor_state/properties")
        .expect("$defs/editor_state/properties must exist");
    assert!(
        es_props.get("edit_mode_target").is_some(),
        "EditorState properties must contain 'edit_mode_target'"
    );
}

#[test]
fn schema_node_uses_lock_state_key_matching_serde_output() {
    let schema = load_schema_value();
    let node_props = schema
        .pointer("/$defs/node/properties")
        .expect("$defs/node/properties must exist");
    assert!(
        node_props.get("lock_state").is_some(),
        "Node properties must contain 'lock_state' (serde field name, no rename attr)"
    );
    assert!(
        node_props.get("locked").is_none(),
        "Node properties must NOT contain 'locked' (serde outputs 'lock_state')"
    );
}

// ===================================================================
// B08-B13: Enum Value Alignment (unit)
// ===================================================================

#[test]
fn schema_node_kind_enum_matches_serde_output() {
    let schema = load_schema_value();
    let kind_enum = schema
        .pointer("/$defs/node/properties/kind/enum")
        .expect("node kind enum must exist");
    assert_eq!(*kind_enum, json!(["node", "subgraph", "text"]));
}

#[test]
fn schema_node_style_enum_matches_serde_output() {
    let schema = load_schema_value();
    let style_enum = schema
        .pointer("/$defs/node/properties/style/enum")
        .expect("node style enum must exist");
    assert_eq!(
        *style_enum,
        json!(["box", "cloud", "cylinder", "dashed", null])
    );
}

#[test]
fn schema_font_weight_enum_matches_serde_output() {
    let schema = load_schema_value();
    let fw_enum = schema
        .pointer("/$defs/node/properties/font_weight/enum")
        .expect("node font_weight enum must exist");
    assert_eq!(*fw_enum, json!(["normal", "bold", null]));
}

#[test]
fn schema_edge_style_enum_matches_serde_output() {
    let schema = load_schema_value();
    let style_enum = schema
        .pointer("/$defs/edge/properties/style/enum")
        .expect("edge style enum must exist");
    assert_eq!(*style_enum, json!(["solid", "dashed", "dotted"]));
}

#[test]
fn schema_arrow_type_enum_matches_serde_output() {
    let schema = load_schema_value();
    let arrow_enum = schema
        .pointer("/$defs/edge/properties/arrowType/enum")
        .expect("edge arrowType enum must exist");
    assert_eq!(
        *arrow_enum,
        json!(["default", "sharp", "curved", "step", "straight"])
    );
}

#[test]
fn schema_editor_theme_enum_matches_serde_output() {
    let schema = load_schema_value();
    let theme_enum = schema
        .pointer("/$defs/editor_state/properties/theme/enum")
        .expect("editor_state theme enum must exist");
    assert_eq!(*theme_enum, json!(["light", "dark", "system"]));
}

// ===================================================================
// B14-B18: Required/OPTIONAL Alignment (unit)
// ===================================================================

#[test]
fn schema_root_required_is_version_revision_document_only() {
    let schema = load_schema_value();
    let required = schema
        .pointer("/required")
        .expect("root required must exist");
    assert_eq!(*required, json!(["version", "revision", "document"]));
}

#[test]
fn schema_node_required_is_kind_label_x_y_width_height_only() {
    let schema = load_schema_value();
    let required = schema
        .pointer("/$defs/node/required")
        .expect("node required must exist");
    assert_eq!(
        *required,
        json!(["kind", "label", "x", "y", "width", "height"])
    );
}

#[test]
fn schema_edge_required_is_source_target_only() {
    let schema = load_schema_value();
    let required = schema
        .pointer("/$defs/edge/required")
        .expect("edge required must exist");
    assert_eq!(*required, json!(["source", "target"]));
}

#[test]
fn schema_editor_state_required_is_camera_x_y_zoom_only() {
    let schema = load_schema_value();
    let required = schema
        .pointer("/$defs/editor_state/required")
        .expect("editor_state required must exist");
    assert_eq!(*required, json!(["camera_x", "camera_y", "zoom"]));
}

#[test]
fn schema_document_data_required_is_nodes_edges() {
    let schema = load_schema_value();
    let required = schema
        .pointer("/properties/document/required")
        .expect("document required must exist");
    assert_eq!(*required, json!(["nodes", "edges"]));
}

// ===================================================================
// B19-B24: additionalProperties Alignment (unit)
// ===================================================================

#[test]
fn schema_root_has_additional_properties_false() {
    let schema = load_schema_value();
    let ap = schema
        .get("additionalProperties")
        .expect("root additionalProperties must exist");
    assert_eq!(ap, &json!(false));
}

#[test]
fn schema_document_data_has_additional_properties_false() {
    let schema = load_schema_value();
    let ap = schema
        .pointer("/properties/document/additionalProperties")
        .expect("document additionalProperties must exist");
    assert_eq!(*ap, json!(false));
}

#[test]
fn schema_node_has_additional_properties_false() {
    let schema = load_schema_value();
    let ap = schema
        .pointer("/$defs/node/additionalProperties")
        .expect("node additionalProperties must exist");
    assert_eq!(*ap, json!(false));
}

#[test]
fn schema_edge_has_additional_properties_false() {
    let schema = load_schema_value();
    let ap = schema
        .pointer("/$defs/edge/additionalProperties")
        .expect("edge additionalProperties must exist");
    assert_eq!(*ap, json!(false));
}

#[test]
fn schema_point_has_additional_properties_false() {
    let schema = load_schema_value();
    let ap = schema
        .pointer("/$defs/point/additionalProperties")
        .expect("point additionalProperties must exist");
    assert_eq!(*ap, json!(false));
}

#[test]
fn schema_editor_state_lacks_additional_properties_false() {
    let schema = load_schema_value();
    let ap = schema.pointer("/$defs/editor_state/additionalProperties");
    let is_false = ap.is_some_and(|v| v.as_bool() == Some(false));
    assert!(
        !is_false,
        "EditorState must NOT have additionalProperties: false (struct lacks deny_unknown_fields)"
    );
}

// ===================================================================
// B25: Forward Compatibility
// ===================================================================

#[test]
fn schema_metadata_allows_additional_properties() {
    let schema = load_schema_value();
    let node_meta = schema
        .pointer("/$defs/node/properties/metadata")
        .expect("node metadata property must exist");
    let meta_type = node_meta.get("type").expect("metadata type must exist");
    assert_eq!(meta_type, &json!("object"));
    let meta_ap = node_meta.get("additionalProperties");
    let is_false = meta_ap.is_some_and(|v| v.as_bool() == Some(false));
    assert!(
        !is_false,
        "Node metadata must allow additional properties for forward compatibility"
    );
}

// ===================================================================
// B26-B29: Alias Exclusion (unit)
// ===================================================================

#[test]
fn schema_edge_excludes_arrowhead_alias_key() {
    let schema = load_schema_value();
    let edge_props = schema
        .pointer("/$defs/edge/properties")
        .expect("$defs/edge/properties must exist");
    assert!(
        edge_props.get("arrowhead").is_none(),
        "Edge properties must NOT contain 'arrowhead' (serde alias, not canonical key)"
    );
}

#[test]
fn schema_edge_excludes_arrow_type_alias_key() {
    let schema = load_schema_value();
    let edge_props = schema
        .pointer("/$defs/edge/properties")
        .expect("$defs/edge/properties must exist");
    assert!(
        edge_props.get("arrow_type").is_none(),
        "Edge properties must NOT contain 'arrow_type' (serde alias, not canonical key)"
    );
}

#[test]
fn schema_node_excludes_font_size_alias_key() {
    let schema = load_schema_value();
    let node_props = schema
        .pointer("/$defs/node/properties")
        .expect("$defs/node/properties must exist");
    assert!(
        node_props.get("font_size").is_none(),
        "Node properties must NOT contain 'font_size' (canonical key is 'fontSize')"
    );
}

#[test]
fn schema_edge_excludes_font_size_alias_key() {
    let schema = load_schema_value();
    let edge_props = schema
        .pointer("/$defs/edge/properties")
        .expect("$defs/edge/properties must exist");
    assert!(
        edge_props.get("font_size").is_none(),
        "Edge properties must NOT contain 'font_size' (canonical key is 'fontSize')"
    );
}

// ===================================================================
// B35-B36: Completeness ($defs)
// ===================================================================

#[test]
fn schema_defs_contains_port_anchor_definition() {
    let schema = load_schema_value();
    let defs = schema.get("$defs").expect("$defs must exist");
    assert!(
        defs.get("port_anchor").is_some(),
        "$defs must contain 'port_anchor' definition"
    );
}

#[test]
fn schema_defs_contains_normalized_offset_definition() {
    let schema = load_schema_value();
    let defs = schema.get("$defs").expect("$defs must exist");
    let norm = defs
        .get("normalized_offset")
        .expect("$defs must contain 'normalized_offset'");
    let required = norm
        .get("required")
        .expect("normalized_offset must have required");
    assert_eq!(*required, json!(["x", "y"]));
    let x_type = norm
        .pointer("/properties/x/type")
        .expect("x type must exist");
    assert_eq!(*x_type, json!("number"));
    let y_type = norm
        .pointer("/properties/y/type")
        .expect("y type must exist");
    assert_eq!(*y_type, json!("number"));
}

// ===================================================================
// B30-B34: Compat Bridge Preservation (I-S7)
//
// The persistence_compat module in diagram_tool remaps legacy field
// names to the canonical serde output names. These tests verify that
// every remapped target name exists in the JSON schema.
// ===================================================================

#[test]
fn compat_bridge_node_font_size_remapped_to_fontSize_exists_in_schema() {
    let schema = load_schema_value();
    let node_props = schema
        .pointer("/$defs/node/properties")
        .expect("$defs/node/properties must exist");
    assert!(
        node_props.get("fontSize").is_some(),
        "Node schema must contain 'fontSize' (target of compat remap from 'font_size')"
    );
}

#[test]
fn compat_bridge_edge_font_size_remapped_to_fontSize_exists_in_schema() {
    let schema = load_schema_value();
    let edge_props = schema
        .pointer("/$defs/edge/properties")
        .expect("$defs/edge/properties must exist");
    assert!(
        edge_props.get("fontSize").is_some(),
        "Edge schema must contain 'fontSize' (target of compat remap from 'font_size')"
    );
}

#[test]
fn compat_bridge_edge_arrowhead_and_arrow_type_remapped_to_arrowType_exists_in_schema() {
    let schema = load_schema_value();
    let edge_props = schema
        .pointer("/$defs/edge/properties")
        .expect("$defs/edge/properties must exist");
    assert!(
        edge_props.get("arrowType").is_some(),
        "Edge schema must contain 'arrowType' (target of compat remap from 'arrowhead'/'arrow_type')"
    );
}

#[test]
fn compat_bridge_legacy_arrow_enum_values_map_to_schema_enum() {
    let schema = load_schema_value();
    let arrow_enum = schema
        .pointer("/$defs/edge/properties/arrowType/enum")
        .expect("edge arrowType enum must exist");
    let schema_values: Vec<&str> = arrow_enum
        .as_array()
        .expect("enum must be array")
        .iter()
        .map(|v| v.as_str().expect("enum value must be string"))
        .collect();

    let compat_remappings = [
        ("arrow", "default"),
        ("open", "straight"),
        ("diamond", "step"),
        ("circle", "curved"),
        ("none", "sharp"),
    ];
    for (legacy, canonical) in &compat_remappings {
        assert!(
            schema_values.contains(canonical),
            "Schema arrowType enum must contain '{canonical}' (compat remap target from legacy '{legacy}')"
        );
    }
}

#[test]
fn compat_bridge_edge_bend_points_and_label_offset_t_exist_in_schema() {
    let schema = load_schema_value();
    let edge_props = schema
        .pointer("/$defs/edge/properties")
        .expect("$defs/edge/properties must exist");
    assert!(
        edge_props.get("bend_points").is_some(),
        "Edge schema must contain 'bend_points' (target of compat remap from 'bendPoints')"
    );
    assert!(
        edge_props.get("label_offset_t").is_some(),
        "Edge schema must contain 'label_offset_t' (target of compat remap from 'labelOffsetT')"
    );
}

// ===================================================================
// B37-B39: PortAnchor/NormalizedOffset Integration
// ===================================================================

#[test]
fn port_anchor_top_validates_against_schema_def() {
    let (schemas, sch_index) = compile_schema_def_as_root("port_anchor");
    let serialized = serde_json::to_value(&PortAnchor::Top).expect("serialize");
    let result = schemas.validate(&serialized, sch_index);
    assert_boon_valid!(result, "PortAnchor::Top must validate");
}

#[test]
fn port_anchor_bottom_validates_against_schema_def() {
    let (schemas, sch_index) = compile_schema_def_as_root("port_anchor");
    let serialized = serde_json::to_value(&PortAnchor::Bottom).expect("serialize");
    let result = schemas.validate(&serialized, sch_index);
    assert_boon_valid!(result, "PortAnchor::Bottom must validate");
}

#[test]
fn port_anchor_left_validates_against_schema_def() {
    let (schemas, sch_index) = compile_schema_def_as_root("port_anchor");
    let serialized = serde_json::to_value(&PortAnchor::Left).expect("serialize");
    let result = schemas.validate(&serialized, sch_index);
    assert_boon_valid!(result, "PortAnchor::Left must validate");
}

#[test]
fn port_anchor_right_validates_against_schema_def() {
    let (schemas, sch_index) = compile_schema_def_as_root("port_anchor");
    let serialized = serde_json::to_value(&PortAnchor::Right).expect("serialize");
    let result = schemas.validate(&serialized, sch_index);
    assert_boon_valid!(result, "PortAnchor::Right must validate");
}

#[test]
fn port_anchor_center_validates_against_schema_def() {
    let (schemas, sch_index) = compile_schema_def_as_root("port_anchor");
    let serialized = serde_json::to_value(&PortAnchor::Center).expect("serialize");
    let result = schemas.validate(&serialized, sch_index);
    assert_boon_valid!(result, "PortAnchor::Center must validate");
}

#[test]
fn port_anchor_custom_variant_validates_against_schema_def() {
    let (schemas, sch_index) = compile_schema_def_as_root("port_anchor");
    let port = PortAnchor::Custom(NormalizedOffset {
        x: OrderedFloat::new_unchecked(0.5),
        y: OrderedFloat::new_unchecked(0.5),
    });
    let serialized = serde_json::to_value(&port).expect("serialize");
    let result = schemas.validate(&serialized, sch_index);
    assert_boon_valid!(result, "PortAnchor::Custom must validate");
}

#[test]
fn normalized_offset_validates_against_schema_def() {
    let (schemas, sch_index) = compile_schema_def_as_root("normalized_offset");
    let offset = NormalizedOffset {
        x: OrderedFloat::new_unchecked(0.5),
        y: OrderedFloat::new_unchecked(0.5),
    };
    let serialized = serde_json::to_value(&offset).expect("serialize");
    let result = schemas.validate(&serialized, sch_index);
    assert_boon_valid!(result, "NormalizedOffset must validate");
}

// ===================================================================
// B40-B46: Roundtrip Integration (serialize → validate)
// ===================================================================

#[test]
fn full_diagram_document_validates_against_schema() {
    let doc = make_full_document();
    let serialized = serde_json::to_value(&doc).expect("document must serialize");
    let (schemas, sch_index) = compile_diagram_schema();
    assert_boon_valid!(
        schemas.validate(&serialized, sch_index),
        "Full DiagramDocument must validate against schema"
    );
}

#[test]
fn minimal_diagram_document_validates_against_schema() {
    let doc = make_minimal_document();
    let serialized = serde_json::to_value(&doc).expect("document must serialize");
    let (schemas, sch_index) = compile_diagram_schema();
    assert_boon_valid!(
        schemas.validate(&serialized, sch_index),
        "Minimal DiagramDocument must validate against schema"
    );
}

#[test]
fn document_with_optional_defaults_validates_against_schema() {
    let n1 = NodeId::new("n1".to_string());
    let n2 = NodeId::new("n2".to_string());
    let e1 = EdgeId::new("e1".to_string());
    let doc = DiagramDocument {
        version: 2,
        revision: Revision::INITIAL,
        document: DocumentData {
            nodes: HashMap::new()
                .update(n1.clone(), make_node_minimal("A"))
                .update(n2.clone(), make_node_minimal("B")),
            edges: HashMap::new().update(e1, make_edge_minimal("n1", "n2")),
        },
        editor_state: EditorState {
            camera_x: OrderedFloat::new_unchecked(0.0),
            camera_y: OrderedFloat::new_unchecked(0.0),
            zoom: OrderedFloat::new_unchecked(1.0),
            grid_size: GridSize::default(),
            snap_to_grid: true,
            selected_items: im::HashSet::new(),
            edit_mode_target: None,
            editing_edge_id: None,
            theme: EditorTheme::System,
            show_grid: true,
            minimap_visible: false,
        },
    };
    let serialized = serde_json::to_value(&doc).expect("document must serialize");
    let (schemas, sch_index) = compile_diagram_schema();
    assert_boon_valid!(
        schemas.validate(&serialized, sch_index),
        "Document with optional defaults must validate"
    );
}

#[test]
fn edge_with_simple_source_port_validates_against_schema() {
    let n1 = NodeId::new("n1".to_string());
    let n2 = NodeId::new("n2".to_string());
    let mut edge = make_edge_minimal("n1", "n2");
    edge.source_port = Some(PortAnchor::Bottom);
    let doc = DiagramDocument {
        version: 2,
        revision: Revision::INITIAL,
        document: DocumentData {
            nodes: HashMap::new()
                .update(n1, make_node_minimal("A"))
                .update(n2, make_node_minimal("B")),
            edges: HashMap::new().update(EdgeId::new("e1".to_string()), edge),
        },
        editor_state: EditorState::default(),
    };
    let serialized = serde_json::to_value(&doc).expect("document must serialize");
    let (schemas, sch_index) = compile_diagram_schema();
    assert_boon_valid!(
        schemas.validate(&serialized, sch_index),
        "Edge with PortAnchor::Bottom must validate"
    );
}

#[test]
fn edge_with_custom_source_port_validates_against_schema() {
    let n1 = NodeId::new("n1".to_string());
    let n2 = NodeId::new("n2".to_string());
    let mut edge = make_edge_minimal("n1", "n2");
    edge.source_port = Some(PortAnchor::Custom(NormalizedOffset {
        x: OrderedFloat::new_unchecked(0.25),
        y: OrderedFloat::new_unchecked(0.75),
    }));
    let doc = DiagramDocument {
        version: 2,
        revision: Revision::INITIAL,
        document: DocumentData {
            nodes: HashMap::new()
                .update(n1, make_node_minimal("A"))
                .update(n2, make_node_minimal("B")),
            edges: HashMap::new().update(EdgeId::new("e1".to_string()), edge),
        },
        editor_state: EditorState::default(),
    };
    let serialized = serde_json::to_value(&doc).expect("document must serialize");
    let (schemas, sch_index) = compile_diagram_schema();
    assert_boon_valid!(
        schemas.validate(&serialized, sch_index),
        "Edge with PortAnchor::Custom must validate"
    );
}

#[test]
fn node_with_font_size_number_validates_against_schema() {
    let mut node = make_node_minimal("A");
    node.font_size = Some(OrderedFloat::new_unchecked(14.0));
    let doc = DiagramDocument {
        version: 2,
        revision: Revision::INITIAL,
        document: DocumentData {
            nodes: HashMap::new().update(NodeId::new("n1".to_string()), node),
            edges: HashMap::new(),
        },
        editor_state: EditorState::default(),
    };
    let serialized = serde_json::to_value(&doc).expect("document must serialize");
    let (schemas, sch_index) = compile_diagram_schema();
    assert_boon_valid!(
        schemas.validate(&serialized, sch_index),
        "Node with fontSize=14 must validate"
    );
}

#[test]
fn node_with_font_size_null_validates_against_schema() {
    let node = make_node_minimal("A");
    let doc = DiagramDocument {
        version: 2,
        revision: Revision::INITIAL,
        document: DocumentData {
            nodes: HashMap::new().update(NodeId::new("n1".to_string()), node),
            edges: HashMap::new(),
        },
        editor_state: EditorState::default(),
    };
    let serialized = serde_json::to_value(&doc).expect("document must serialize");
    let (schemas, sch_index) = compile_diagram_schema();
    assert_boon_valid!(
        schemas.validate(&serialized, sch_index),
        "Node with fontSize=null must validate"
    );
}

// ===================================================================
// Mutation tests: wrong values / field names must FAIL validation
// These test the FIXED schema (should pass once schema is correct).
// In RED phase they fail because the schema itself is broken.
// ===================================================================

#[test]
fn schema_rejects_missing_required_version_field() {
    let (schemas, sch_index) = compile_diagram_schema();
    let instance = json!({
        "revision": 0,
        "document": { "nodes": {}, "edges": {} }
    });
    let result = schemas.validate(&instance, sch_index);
    let err_msg = result
        .expect_err("Schema must reject document missing 'version'")
        .to_string();
    assert!(
        err_msg.contains("version"),
        "error should mention 'version': {err_msg}"
    );
}

#[test]
fn schema_rejects_unknown_field_on_node() {
    let (schemas, sch_index) = compile_diagram_schema();
    let instance = json!({
        "version": 2,
        "revision": 0,
        "document": {
            "nodes": {
                "n1": {
                    "kind": "node",
                    "label": "A",
                    "x": 0.0, "y": 0.0,
                    "width": 100.0, "height": 60.0,
                    "bogus_field": true
                }
            },
            "edges": {}
        }
    });
    let result = schemas.validate(&instance, sch_index);
    let err_msg = result
        .expect_err("Schema must reject unknown field on Node")
        .to_string();
    assert!(
        err_msg.contains("bogus_field"),
        "error should mention 'bogus_field': {err_msg}"
    );
}

#[test]
fn schema_rejects_unknown_field_on_edge() {
    let (schemas, sch_index) = compile_diagram_schema();
    let instance = json!({
        "version": 2,
        "revision": 0,
        "document": {
            "nodes": {
                "n1": {
                    "kind": "node", "label": "A",
                    "x": 0.0, "y": 0.0,
                    "width": 100.0, "height": 60.0
                },
                "n2": {
                    "kind": "node", "label": "B",
                    "x": 100.0, "y": 0.0,
                    "width": 100.0, "height": 60.0
                }
            },
            "edges": {
                "e1": {
                    "source": "n1", "target": "n2",
                    "bogus_field": true
                }
            }
        }
    });
    let result = schemas.validate(&instance, sch_index);
    let err_msg = result
        .expect_err("Schema must reject unknown field on Edge")
        .to_string();
    assert!(
        err_msg.contains("bogus_field"),
        "error should mention 'bogus_field': {err_msg}"
    );
}

#[test]
fn schema_rejects_node_missing_required_kind() {
    let (schemas, sch_index) = compile_diagram_schema();
    let instance = json!({
        "version": 2,
        "revision": 0,
        "document": {
            "nodes": {
                "n1": {
                    "label": "A",
                    "x": 0.0, "y": 0.0,
                    "width": 100.0, "height": 60.0
                }
            },
            "edges": {}
        }
    });
    let result = schemas.validate(&instance, sch_index);
    let err_msg = result
        .expect_err("Schema must reject Node missing 'kind'")
        .to_string();
    assert!(
        err_msg.contains("kind"),
        "error should mention 'kind': {err_msg}"
    );
}

#[test]
fn schema_rejects_node_missing_required_label() {
    let (schemas, sch_index) = compile_diagram_schema();
    let instance = json!({
        "version": 2,
        "revision": 0,
        "document": {
            "nodes": {
                "n1": {
                    "kind": "node",
                    "x": 0.0, "y": 0.0,
                    "width": 100.0, "height": 60.0
                }
            },
            "edges": {}
        }
    });
    let result = schemas.validate(&instance, sch_index);
    let err_msg = result
        .expect_err("Schema must reject Node missing 'label'")
        .to_string();
    assert!(
        err_msg.contains("label"),
        "error should mention 'label': {err_msg}"
    );
}

#[test]
fn schema_rejects_edge_missing_required_source() {
    let (schemas, sch_index) = compile_diagram_schema();
    let instance = json!({
        "version": 2,
        "revision": 0,
        "document": {
            "nodes": {
                "n1": {
                    "kind": "node", "label": "A",
                    "x": 0.0, "y": 0.0,
                    "width": 100.0, "height": 60.0
                },
                "n2": {
                    "kind": "node", "label": "B",
                    "x": 100.0, "y": 0.0,
                    "width": 100.0, "height": 60.0
                }
            },
            "edges": {
                "e1": {
                    "target": "n2"
                }
            }
        }
    });
    let result = schemas.validate(&instance, sch_index);
    let err_msg = result
        .expect_err("Schema must reject Edge missing 'source'")
        .to_string();
    assert!(
        err_msg.contains("source"),
        "error should mention 'source': {err_msg}"
    );
}

#[test]
fn schema_rejects_wrong_arrow_type_enum_value() {
    let (schemas, sch_index) = compile_diagram_schema();
    let instance = json!({
        "version": 2,
        "revision": 0,
        "document": {
            "nodes": {
                "n1": {
                    "kind": "node", "label": "A",
                    "x": 0.0, "y": 0.0,
                    "width": 100.0, "height": 60.0
                },
                "n2": {
                    "kind": "node", "label": "B",
                    "x": 100.0, "y": 0.0,
                    "width": 100.0, "height": 60.0
                }
            },
            "edges": {
                "e1": {
                    "source": "n1", "target": "n2",
                    "arrowType": "invalid"
                }
            }
        }
    });
    let result = schemas.validate(&instance, sch_index);
    let err_msg = result
        .expect_err("Schema must reject invalid arrowType enum value")
        .to_string();
    assert!(
        err_msg.contains("arrowType") || err_msg.contains("enum"),
        "error should reference arrowType or enum: {err_msg}"
    );
}

#[test]
fn schema_rejects_unknown_field_on_document_data() {
    let (schemas, sch_index) = compile_diagram_schema();
    let instance = json!({
        "version": 2,
        "revision": 0,
        "document": {
            "nodes": {},
            "edges": {},
            "bogus_field": true
        }
    });
    let result = schemas.validate(&instance, sch_index);
    let err_msg = result
        .expect_err("Schema must reject unknown field on DocumentData")
        .to_string();
    assert!(
        err_msg.contains("bogus_field"),
        "error should mention 'bogus_field': {err_msg}"
    );
}

#[test]
fn schema_accepts_unknown_field_on_editor_state_for_forward_compatibility() {
    let (schemas, sch_index) = compile_diagram_schema();
    let instance = json!({
        "version": 2,
        "revision": 0,
        "document": { "nodes": {}, "edges": {} },
        "editor_state": {
            "camera_x": 0.0,
            "camera_y": 0.0,
            "zoom": 1.0,
            "future_setting": true
        }
    });
    let result = schemas.validate(&instance, sch_index);
    assert_boon_valid!(
        result,
        "EditorState must accept unknown fields for forward compatibility"
    );
}
