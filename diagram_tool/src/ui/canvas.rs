#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::cast_precision_loss)]
#![forbid(unsafe_code)]

use dioxus::prelude::*;
use dioxus::html::geometry::WheelDelta;
use dioxus::html::input_data::MouseButton;
use crate::models::document::{DiagramDocument, Node, NodeKind, Edge, NodeId, EdgeId, OrderedFloat, NodeStyle, EdgeStyle};
use crate::history::History;
use im::HashMap;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub enum InteractionMode {
    Select,
    RubberBand { start: (f64, f64), current: (f64, f64) },
    DraggingSelection {
        anchor: (f64, f64),
        original_positions: HashMap<NodeId, (f64, f64)>,
    },
    DrawingEdge { from_node: NodeId, current_pos: (f64, f64) },
    Panning { last_pos: (f64, f64) },
}

#[component]
pub fn Canvas() -> Element {
    let mut doc_signal = use_context::<Signal<DiagramDocument>>();
    let mut dragging_icon = use_context::<Signal<Option<String>>>();
    let mut history_signal = use_context::<Signal<History>>();
    
    let mut interaction_mode = use_signal(|| InteractionMode::Select);
    let mut alt_pressed = use_signal(|| false);
    let mut space_pressed = use_signal(|| false);

    let nodes_list = use_memo(move || doc_signal.read().document.nodes.clone());
    let edges_list = use_memo(move || doc_signal.read().document.edges.clone());
    
    let to_canvas_coords = |client_x: f64, client_y: f64, cam_x: f64, cam_y: f64, zoom: f64| {
        ((client_x - cam_x) / zoom, (client_y - cam_y) / zoom)
    };

    let to_screen_coords = |world_x: f64, world_y: f64, cam_x: f64, cam_y: f64, zoom: f64| {
        (world_x.mul_add(zoom, cam_x), world_y.mul_add(zoom, cam_y))
    };

    use_effect(move || {
        let mut eval = document::eval(r#"
            window.addEventListener('keydown', (e) => {
                if (e.key === " ") e.preventDefault();
                dioxus.send({ type: 'keydown', key: e.key, ctrl: e.ctrlKey, shift: e.shiftKey, alt: e.altKey });
            });
            window.addEventListener('keyup', (e) => {
                dioxus.send({ type: 'keyup', key: e.key, ctrl: e.ctrlKey, shift: e.shiftKey, alt: e.altKey });
            });
        "#);
        
        spawn(async move {
            while let Ok(json) = eval.recv::<serde_json::Value>().await {
                let event_type = json["type"].as_str().map_or("", |s| s);
                let key = json["key"].as_str().map_or("", |s| s);
                let ctrl = json["ctrl"].as_bool().is_some_and(|b| b);
                let shift = json["shift"].as_bool().is_some_and(|b| b);
                let alt = json["alt"].as_bool().is_some_and(|b| b);
                
                alt_pressed.set(alt);
                if key == " " { space_pressed.set(event_type == "keydown"); }

                if event_type == "keydown" {
                    match (ctrl, key) {
                        (true, "z" | "Z") if shift => {
                            let (current, history) = (doc_signal.read().clone(), history_signal.read().clone());
                            if let Some((doc, h)) = history.redo(current) {
                                *doc_signal.write() = doc;
                                *history_signal.write() = h;
                            }
                        },
                        (true, "z" | "Z") => {
                            let (current, history) = (doc_signal.read().clone(), history_signal.read().clone());
                            if let Some((doc, h)) = history.undo(current) {
                                *doc_signal.write() = doc;
                                *history_signal.write() = h;
                            }
                        },
                        (true, "y" | "Y") => {
                            let (current, history) = (doc_signal.read().clone(), history_signal.read().clone());
                            if let Some((doc, h)) = history.redo(current) {
                                *doc_signal.write() = doc;
                                *history_signal.write() = h;
                            }
                        },
                        (_, "Delete" | "Backspace") => {
                            let mut doc = doc_signal.write();
                            let selected_ids = doc.editor_state.selected_items.clone();
                            if !selected_ids.is_empty() {
                                let (current_doc, history) = (doc.clone(), history_signal.read().clone());
                                *history_signal.write() = history.push(current_doc);
                                doc.document.nodes = doc.document.nodes.iter().filter(|(id, _)| !selected_ids.contains(&id.to_string())).map(|(id, n)| (id.clone(), n.clone())).collect();
                                let node_ids = doc.document.nodes.keys().cloned().collect::<im::HashSet<NodeId>>();
                                doc.document.edges = doc.document.edges.iter().filter(|(id, edge)| {
                                    node_ids.contains(&edge.source) && node_ids.contains(&edge.target) && !selected_ids.contains(&id.to_string())
                                }).map(|(id, e)| (id.clone(), e.clone())).collect();
                                doc.editor_state.selected_items.clear();
                                doc.revision = doc.revision.increment();
                            }
                        },
                        _ => {}
                    }
                }
            }
        });
    });

    let handle_drop = move |evt: DragEvent| {
        evt.prevent_default();
        dragging_icon.with_mut(|dragging| {
            if let Some(path) = dragging.take() {
                let (current_doc, history) = (doc_signal.read().clone(), history_signal.read().clone());
                *history_signal.write() = history.push(current_doc);
                doc_signal.with_mut(|doc| {
                    let coords = evt.data.coordinates().client();
                    let (x, y) = to_canvas_coords(coords.x, coords.y, doc.editor_state.camera_x.0, doc.editor_state.camera_y.0, doc.editor_state.zoom.0);
                    let _ = doc.document.nodes.insert(NodeId::new(Uuid::new_v4().to_string()), Node {
                        kind: NodeKind::Node,
                        icon: path, label: String::from("New Node"),
                        x: OrderedFloat(x), y: OrderedFloat(y), width: OrderedFloat(64.0), height: OrderedFloat(64.0),
                        locked: false, parent: None, tags: Vec::new(), metadata: HashMap::new(), style: NodeStyle::default(),
                    });
                    doc.revision = doc.revision.increment();
                });
            }
        });
    };

    rsx! {
        div {
            class: "canvas-container",
            style: "flex: 1; position: relative; overflow: hidden; background: #ffffff; cursor: crosshair; user-select: none;",
            
            ondragover: move |evt| { evt.prevent_default(); },
            ondrop: handle_drop,
            
            onwheel: move |evt| {
                evt.prevent_default();
                let dy = match evt.data.delta() {
                    WheelDelta::Pixels(v) => v.y,
                    WheelDelta::Lines(v) => v.y * 20.0,
                    WheelDelta::Pages(v) => v.y * 100.0,
                };
                let zoom_factor = if dy > 0.0 { 0.9 } else { 1.1 };
                
                doc_signal.with_mut(|doc| {
                    let s = &mut doc.editor_state;
                    let old_zoom = s.zoom.0;
                    let new_zoom = (s.zoom.0 * zoom_factor).clamp(0.1, 5.0);
                    let coords = evt.data.coordinates().client();
                    let (wx, wy) = to_canvas_coords(coords.x, coords.y, s.camera_x.0, s.camera_y.0, old_zoom);
                    s.camera_x = OrderedFloat(wx.mul_add(-new_zoom, coords.x));
                    s.camera_y = OrderedFloat(wy.mul_add(-new_zoom, coords.y));
                    s.zoom = OrderedFloat(new_zoom);
                });
            },
            
            onmousedown: move |evt| {
                let coords = evt.data.coordinates().client();
                let is_middle = evt.data.trigger_button() == Some(MouseButton::Auxiliary);
                
                if *space_pressed.read() || is_middle { 
                    interaction_mode.set(InteractionMode::Panning { last_pos: (coords.x, coords.y) });
                } else if !*alt_pressed.read() {
                    let pos = {
                        let doc = doc_signal.read();
                        to_canvas_coords(coords.x, coords.y, doc.editor_state.camera_x.0, doc.editor_state.camera_y.0, doc.editor_state.zoom.0)
                    };
                    doc_signal.with_mut(|d| d.editor_state.selected_items.clear());
                    interaction_mode.set(InteractionMode::RubberBand { start: pos, current: pos });
                }
            },
            
            onmousemove: move |evt| {
                let coords = evt.data.coordinates().client();
                interaction_mode.with_mut(|mode| {
                    match mode {
                        InteractionMode::DraggingSelection { anchor, original_positions } => {
                            doc_signal.with_mut(|doc| {
                                let (curr_x, curr_y) = to_canvas_coords(coords.x, coords.y, doc.editor_state.camera_x.0, doc.editor_state.camera_y.0, doc.editor_state.zoom.0);
                                let (dx, dy) = (curr_x - anchor.0, curr_y - anchor.1);
                                for (id, (ox, oy)) in original_positions.iter() {
                                    doc.document.nodes = doc.document.nodes.alter(|n| {
                                        match n {
                                            Some(mut node) => {
                                                node.x = OrderedFloat(*ox + dx); 
                                                node.y = OrderedFloat(*oy + dy); 
                                                node.locked = true;
                                                Some(node)
                                            },
                                            None => None,
                                        }
                                    }, id.clone());
                                }
                            });
                        },
                        InteractionMode::DrawingEdge { current_pos, .. } => {
                            let doc = doc_signal.read();
                            *current_pos = to_canvas_coords(coords.x, coords.y, doc.editor_state.camera_x.0, doc.editor_state.camera_y.0, doc.editor_state.zoom.0);
                        },
                        InteractionMode::RubberBand { current, .. } => {
                            let doc = doc_signal.read();
                            *current = to_canvas_coords(coords.x, coords.y, doc.editor_state.camera_x.0, doc.editor_state.camera_y.0, doc.editor_state.zoom.0);
                        },
                        InteractionMode::Panning { last_pos } => {
                            let dx = coords.x - last_pos.0;
                            let dy = coords.y - last_pos.1;
                            *last_pos = (coords.x, coords.y);
                            doc_signal.with_mut(|doc| {
                                doc.editor_state.camera_x = OrderedFloat(doc.editor_state.camera_x.0 + dx);
                                doc.editor_state.camera_y = OrderedFloat(doc.editor_state.camera_y.0 + dy);
                            });
                        },
                        InteractionMode::Select => {}
                    }
                });
            },
            
            onmouseup: move |_| {
                interaction_mode.with_mut(|mode| {
                    match mode {
                        InteractionMode::RubberBand { start, current } => {
                            let (min_x, min_y) = (start.0.min(current.0), start.1.min(current.1));
                            let (max_x, max_y) = (start.0.max(current.0), start.1.max(current.1));
                            doc_signal.with_mut(|doc| {
                                doc.editor_state.selected_items = doc.document.nodes.iter()
                                    .filter(|(_, n)| n.x.0 >= min_x && n.y.0 >= min_y && n.x.0 + n.width.0 <= max_x && n.y.0 + n.height.0 <= max_y)
                                    .map(|(id, _)| id.to_string())
                                    .collect();
                            });
                            *mode = InteractionMode::Select;
                        },
                        _ => *mode = InteractionMode::Select,
                    }
                });
            },

            // SVG Layer
            svg {
                style: "position: absolute; top: 0; left: 0; width: 100%; height: 100%; pointer-events: none; z-index: 0;",
                
                // Existing Edges
                {
                    let doc = doc_signal.read();
                    let s = &doc.editor_state;
                    edges_list.read().iter().filter_map(|(id, edge)| {
                        match (doc.document.nodes.get(&edge.source), doc.document.nodes.get(&edge.target)) {
                            (Some(src), Some(tgt)) => {
                                let (sx, sy) = to_screen_coords(src.x.0 + src.width.0/2.0, src.y.0 + src.height.0/2.0, s.camera_x.0, s.camera_y.0, s.zoom.0);
                                let (tx, ty) = to_screen_coords(tgt.x.0 + tgt.width.0/2.0, tgt.y.0 + tgt.height.0/2.0, s.camera_x.0, s.camera_y.0, s.zoom.0);
                                Some(rsx! { line { key: "{id:?}", x1: "{sx}", y1: "{sy}", x2: "{tx}", y2: "{ty}", stroke: "black", stroke_width: "2" } })
                            },
                            _ => None,
                        }
                    }).collect::<Vec<_>>().into_iter()
                }
                
                // Temporary Edge
                {
                    let mode = interaction_mode.read();
                    let doc = doc_signal.read();
                    let s = &doc.editor_state;
                    
                    if let InteractionMode::DrawingEdge { from_node, current_pos } = &*mode {
                        doc.document.nodes.get(from_node).map_or_else(|| rsx! {}, |src| {
                            let (sx, sy) = to_screen_coords(src.x.0 + src.width.0/2.0, src.y.0 + src.height.0/2.0, s.camera_x.0, s.camera_y.0, s.zoom.0);
                            let (tx, ty) = to_screen_coords(current_pos.0, current_pos.1, s.camera_x.0, s.camera_y.0, s.zoom.0);
                            rsx! { line { x1: "{sx}", y1: "{sy}", x2: "{tx}", y2: "{ty}", stroke: "blue", stroke_width: "2", stroke_dasharray: "5,5" } }
                        })
                    } else { rsx! {} }
                }

                // Rubber Band
                {
                    let mode = interaction_mode.read();
                    if let InteractionMode::RubberBand { start, current } = &*mode {
                         let s = &doc_signal.read().editor_state;
                         let (rx, ry) = to_screen_coords(start.0.min(current.0), start.1.min(current.1), s.camera_x.0, s.camera_y.0, s.zoom.0);
                         let rw = (start.0 - current.0).abs() * s.zoom.0;
                         let rh = (start.1 - current.1).abs() * s.zoom.0;
                         rsx! { rect { x: "{rx}", y: "{ry}", width: "{rw}", height: "{rh}", fill: "rgba(0, 0, 255, 0.1)", stroke: "blue", stroke_width: "1", stroke_dasharray: "3,3" } }
                    } else { rsx! {} }
                }
            }

            // HTML Layer
            {
                nodes_list.read().iter().map(|(id, node)| {
                    let (id_mousedown, id_mouseup, node_pos) = (id.clone(), id.clone(), (node.x.0, node.y.0));
                    let doc_clone = doc_signal.read().clone();
                    let s = &doc_clone.editor_state;
                    let is_selected = s.selected_items.contains(&id.to_string());
                    let (left, top) = to_screen_coords(node.x.0, node.y.0, s.camera_x.0, s.camera_y.0, s.zoom.0);
                    let (width, height) = (node.width.0 * s.zoom.0, node.height.0 * s.zoom.0);
                    let border = if is_selected { "2px solid blue" } else { "1px solid black" };
                    
                    rsx! {
                        div {
                            key: "{id:?}",
                            style: "position: absolute; left: {left}px; top: {top}px; width: {width}px; height: {height}px; 
                                    border: {border}; 
                                    background: white; display: flex; flex-direction: column; align-items: center; justify-content: center; cursor: pointer; z-index: 1;",
                            
                            onmousedown: move |evt| {
                                evt.stop_propagation(); 
                                let (doc, alt) = (doc_signal.read().clone(), *alt_pressed.read());
                                let coords = evt.data.coordinates().client();
                                let pos = to_canvas_coords(coords.x, coords.y, doc.editor_state.camera_x.0, doc.editor_state.camera_y.0, doc.editor_state.zoom.0);
                                if alt {
                                    interaction_mode.set(InteractionMode::DrawingEdge { from_node: id_mousedown.clone(), current_pos: pos });
                                } else {
                                    let history = history_signal.read().clone();
                                    *history_signal.write() = history.push(doc);
                                    interaction_mode.set(InteractionMode::DraggingSelection { anchor: pos, original_positions: HashMap::new().update(id_mousedown.clone(), node_pos) });
                                }
                            },
                            
                            onmouseup: move |evt| {
                                evt.stop_propagation();
                                let mode = interaction_mode.read().clone();
                                if let InteractionMode::DrawingEdge { from_node, .. } = mode {
                                    if from_node != id_mouseup {
                                        let (current_doc, history) = (doc_signal.read().clone(), history_signal.read().clone());
                                        *history_signal.write() = history.push(current_doc);
                                        doc_signal.with_mut(|doc| {
                                            doc.document.edges = doc.document.edges.update(EdgeId::new(Uuid::new_v4().to_string()), Edge {
                                                source: from_node.clone(), target: id_mouseup.clone(), label: String::new(),
                                                style: EdgeStyle::default(), directed: true, bend_points: Vec::new(),
                                            });
                                            doc.revision = doc.revision.increment();
                                        });
                                    }
                                    interaction_mode.set(InteractionMode::Select);
                                }
                            },
                            
                            img {
                                src: "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==", 
                                width: "{32.0 * doc_clone.editor_state.zoom.0}px", height: "{32.0 * doc_clone.editor_state.zoom.0}px",
                                draggable: "false" 
                            },
                            span { style: "font-size: {10.0 * doc_clone.editor_state.zoom.0}px", "{node.label}" }
                        }
                    }
                }).collect::<Vec<_>>().into_iter()
            }
        }
    }
}
