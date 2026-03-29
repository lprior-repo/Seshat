#![allow(clippy::all, clippy::pedantic, clippy::nursery, dead_code)]
use serde_json::Value;

pub fn generate_stress_scene_json() -> Value {
    let (node_count, edge_count) = (5000, 5000);
    let mut rng = 12345u64;
    let mut next_rnd = || {
        rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
        ((rng >> 16) & 0xFFFF) as f64 / 65535.0
    };

    let mut nodes = serde_json::Map::new();
    let mut node_ids = Vec::with_capacity(node_count);
    for i in 0..node_count {
        let id = format!("stress-node-{i}");
        node_ids.push(id.clone());
        let r = next_rnd();
        let kind = if r < 0.8 {
            "node"
        } else if r < 0.95 {
            "text"
        } else {
            "subgraph"
        };
        nodes.insert(
            id,
            serde_json::json!({
                "kind": kind, "icon": "", "label": format!("Node {}", i),
                "x": next_rnd() * 5000.0, "y": next_rnd() * 5000.0,
                "width": 80.0 + next_rnd() * 40.0, "height": 40.0 + next_rnd() * 20.0,
                "locked": false, "parent": null, "tags": [], "metadata": {}, "z_index": i as i64
            }),
        );
    }

    let mut edges = serde_json::Map::new();
    for i in 0..edge_count {
        let src = (next_rnd() * node_count as f64) as usize;
        let mut tgt = (next_rnd() * node_count as f64) as usize;
        if src == tgt {
            tgt = (tgt + 1) % node_count;
        }
        edges.insert(
            format!("stress-edge-{i}"),
            serde_json::json!({
                "source": node_ids[src], "target": node_ids[tgt],
                "label": "", "style": "solid", "arrowType": "default", "label_offset_t": 0.5,
                "thickness": 1.5, "directed": true, "bend_points": [], "tags": [], "metadata": {}
            }),
        );
    }

    serde_json::json!({
        "version": 2, "revision": 0, "document": { "nodes": nodes, "edges": edges },
        "editor_state": { "camera_x": 0.0, "camera_y": 0.0, "zoom": 0.5, "grid_size": 20.0,
        "snap_to_grid": true, "selected_items": [], "editing_edge_id": null, "theme": "system",
        "show_grid": true, "minimap_visible": true }
    })
}
