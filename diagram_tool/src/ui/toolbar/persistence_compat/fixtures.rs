pub const TS_STYLE_JSON: &str = r#"{
  "version": 2,
  "revision": 1,
  "document": {
    "nodes": {
      "n1": {
        "id": "n1",
        "kind": "node",
        "icon": "aws/compute/ec2",
        "label": "EC2",
        "x": 10,
        "y": 20,
        "width": 64,
        "height": 64,
        "locked": true,
        "parent": null,
        "tags": ["aws", "compute"],
        "metadata": {}
      }
    },
    "edges": {
      "e1": {
        "id": "e1",
        "source": "n1",
        "target": "n1",
        "label": "",
        "style": "solid",
        "arrowType": "curved",
        "directed": true,
        "bend_points": []
      }
    }
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

pub const LEGACY_FONT_SIZE_JSON: &str = r#"{
  "version": 2,
  "revision": 1,
  "document": {
    "nodes": {
      "n1": {
        "id": "n1",
        "kind": "node",
        "icon": "aws/compute/ec2",
        "label": "EC2",
        "x": 10,
        "y": 20,
        "width": 64,
        "height": 64,
        "font_size": null,
        "locked": true,
        "parent": null,
        "tags": [],
        "metadata": {}
      }
    },
    "edges": {
      "e1": {
        "id": "e1",
        "source": "n1",
        "target": "n1",
        "label": "",
        "style": "solid",
        "arrowType": "curved",
        "directed": true,
        "font_size": null,
        "bend_points": []
      }
    }
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

pub const LEGACY_A: &str = r#"{
  "version": 2,
  "revision": 0,
  "document": {
    "nodes": {
      "n1": {
        "id": "n1",
        "kind": "node",
        "icon": "",
        "label": "A",
        "x": 0,
        "y": 0,
        "width": 80,
        "height": 60,
        "locked": false,
        "parent": null,
        "tags": [],
        "metadata": {},
        "font_size": 12,
        "dagRank": 7
      }
    },
    "edges": {
      "e1": {
        "id": "e1",
        "source": "n1",
        "target": "n1",
        "label": "",
        "style": "solid",
        "arrow_type": "diamond",
        "labelOffsetT": 0.25,
        "bendPoints": [],
        "directed": true,
        "metadata": {}
      }
    }
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

pub const LEGACY_B: &str = r#"{
  "version": 2,
  "revision": 0,
  "document": {
    "nodes": {
      "n1": {
        "kind": "node",
        "icon": "",
        "label": "A",
        "x": 0,
        "y": 0,
        "width": 80,
        "height": 60,
        "locked": false,
        "parent": null,
        "tags": [],
        "metadata": {},
        "fontSize": 12,
        "dag_rank": 7
      }
    },
    "edges": {
      "e1": {
        "source": "n1",
        "target": "n1",
        "label": "",
        "style": "solid",
        "arrowType": "step",
        "label_offset_t": 0.25,
        "bend_points": [],
        "directed": true,
        "metadata": {}
      }
    }
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

pub const VERSION_1_DOCUMENT: &str = r#"{
    "version": 1,
    "revision": 0,
    "document": {
        "nodes": {
            "legacy_node": {
                "kind": "node",
                "icon": "",
                "label": "Legacy",
                "x": 100,
                "y": 200,
                "width": 80,
                "height": 40,
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

pub const LEGACY_FIELDS_DOCUMENT: &str = r#"{
    "version": 2,
    "revision": 0,
    "document": {
        "nodes": {
            "legacy_fields": {
                "kind": "node",
                "icon": "",
                "label": "Legacy Fields",
                "x": 50,
                "y": 50,
                "width": 100,
                "height": 60,
                "font_size": 14,
                "fontWeight": "bold",
                "dagRank": 5,
                "locked": false,
                "parent": null,
                "tags": [],
                "metadata": {}
            }
        },
        "edges": {
            "legacy_edge": {
                "source": "legacy_fields",
                "target": "legacy_fields",
                "label": "",
                "style": "solid",
                "arrowhead": "diamond",
                "labelOffsetT": 0.75,
                "bendPoints": [],
                "directed": true,
                "metadata": {}
            }
        }
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

pub const NO_VERSION_DOCUMENT: &str = r#"{
    "revision": 0,
    "document": {
        "nodes": {
            "no_version": {
                "kind": "node",
                "icon": "",
                "label": "No Version",
                "x": 0,
                "y": 0,
                "width": 80,
                "height": 40,
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
