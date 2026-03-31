//! Criterion microbenchmarks for Seshat hot paths.
//!
//! Run with:  cargo bench -p diagram_models
//! Compare:  cargo bench -p diagram_models -- --baseline main

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::semicolon_if_nothing_returned,
    clippy::let_and_return,
    clippy::redundant_closure,
    clippy::similar_names,
    clippy::many_single_char_names,
    clippy::doc_markdown
)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use diagram_models::document::{
    DiagramDocument, DocumentData, Edge, EdgeId, EditorState, LockState, Node, NodeId, NodeKind,
    OrderedFloat, Revision,
};
use diagram_models::geometry::{Point, AABB};
use diagram_models::history::History;
use diagram_models::spatial_index::{build_spatial_index, query_spatial_index, MarqueeMode};
use diagram_models::subgraph::selection::evaluate_selection;
use diagram_models::subgraph::selection::SelectionModifiers;
use diagram_models::subgraph::types::CanvasState;
use im::HashMap;

// ---------------------------------------------------------------------------
// Shared test-data builders (pure, no mut)
// ---------------------------------------------------------------------------

fn make_node(id: &str, px: f64, py: f64, pw: f64, ph: f64, z: i64) -> (NodeId, Node) {
    (
        NodeId::new(id.to_string()),
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: id.to_string(),
            x: OrderedFloat::new_unchecked(px),
            y: OrderedFloat::new_unchecked(py),
            width: OrderedFloat::new_unchecked(pw),
            height: OrderedFloat::new_unchecked(ph),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: im::HashMap::new(),
            z_index: z,
            style: None,
            collapsed: None,
        },
    )
}

fn make_edge(idx: usize) -> (EdgeId, Edge) {
    (
        EdgeId::new(format!("e{idx}")),
        Edge {
            source: NodeId::new(format!("n{idx}")),
            target: NodeId::new(format!("n{}_tgt", idx + 1)),
            source_port: None,
            target_port: None,
            label: String::new(),
            style: diagram_models::document::EdgeStyle::default(),
            arrow_type: diagram_models::document::ArrowType::default(),
            label_offset_t: OrderedFloat::new_unchecked(0.5),
            color: None,
            thickness: OrderedFloat::new_unchecked(1.5),
            directed: true,
            bend_points: im::Vector::new(),
            tags: im::Vector::new(),
            metadata: im::HashMap::new(),
            font_size: None,
        },
    )
}

fn build_nodes(count: usize) -> HashMap<NodeId, Node> {
    let golden = 137.508_f64;
    let golden_y = 137.508 * 1.618;
    (0..count)
        .map(|i| {
            let fi = i as f64;
            let px = (fi * golden) % 10_000.0;
            let py = (fi * golden_y) % 10_000.0;
            make_node(&format!("n{i}"), px, py, 50.0, 50.0, i as i64)
        })
        .collect()
}

fn build_doc(count: usize) -> DiagramDocument {
    DiagramDocument {
        version: 2,
        revision: Revision::INITIAL,
        document: DocumentData {
            nodes: build_nodes(count),
            edges: HashMap::new(),
        },
        editor_state: EditorState::default(),
    }
}

fn build_canvas_state(count: usize) -> CanvasState {
    CanvasState {
        nodes: build_nodes(count),
        edges: HashMap::new(),
    }
}

// ---------------------------------------------------------------------------
// Benchmark 1: Spatial Index Build
// ---------------------------------------------------------------------------

fn bench_spatial_index_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("spatial_index_build");
    for size in [100_usize, 500, 1_000, 3_000, 5_000] {
        let nodes = build_nodes(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &nodes, |b, nodes| {
            b.iter(|| black_box(build_spatial_index(black_box(nodes))))
        });
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark 2: Spatial Index Query (marquee)
// ---------------------------------------------------------------------------

fn bench_spatial_index_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("spatial_index_query");
    for size in [100_usize, 500, 1_000, 3_000, 5_000] {
        let nodes = build_nodes(size);
        let index = build_spatial_index(&nodes);
        let marquee = AABB::new(1000.0, 1000.0, 1500.0, 1500.0);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &(index, nodes, marquee),
            |b, (index, nodes, marquee)| {
                b.iter(|| {
                    black_box(query_spatial_index(
                        black_box(index),
                        black_box(nodes),
                        black_box(*marquee),
                        MarqueeMode::Intersect,
                    ))
                })
            },
        );
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark 3: evaluate_selection
// ---------------------------------------------------------------------------

fn bench_evaluate_selection(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluate_selection");
    for size in [100_usize, 500, 1_000, 3_000, 5_000] {
        let canvas = build_canvas_state(size);
        let click_pos = Point::new(100.0, 100.0);
        let modifiers = SelectionModifiers { ctrl: false };
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &(canvas, click_pos, modifiers),
            |b, (canvas, pos, mods)| {
                b.iter(|| {
                    let _ = black_box(evaluate_selection(
                        black_box(canvas),
                        black_box(*pos),
                        black_box(mods.clone()),
                    ));
                })
            },
        );
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark 4: History push + undo
// ---------------------------------------------------------------------------

fn bench_history_push_undo(c: &mut Criterion) {
    let mut group = c.benchmark_group("history_push_undo");
    for size in [10_usize, 50, 100] {
        let doc = build_doc(100);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &doc, |b, doc| {
            b.iter(|| {
                let mut history = History::new();
                for i in 0..size {
                    let mut d = doc.clone();
                    d.revision = Revision::new(i as u64);
                    history = history.push(d);
                }
                let mut current = doc.clone();
                for _ in 0..size {
                    if let Some((prev, new_history)) = history.undo(current.clone()) {
                        current = prev;
                        history = new_history;
                    }
                }
                black_box((current, history))
            })
        });
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark 5: DAG validation
// ---------------------------------------------------------------------------

fn bench_dag_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("dag_validation");
    for size in [100_usize, 500, 1_000, 3_000] {
        let nodes = build_nodes(size);
        let edges: HashMap<EdgeId, Edge> = (0..size.saturating_sub(1)).map(make_edge).collect();
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &(nodes, edges),
            |b, (nodes, edges)| {
                b.iter(|| {
                    black_box(diagram_models::dag::validate_dag(
                        black_box(nodes),
                        black_box(edges),
                    ))
                })
            },
        );
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark 6: Canonical JSON serialization
// ---------------------------------------------------------------------------

fn bench_canonical_json(c: &mut Criterion) {
    let mut group = c.benchmark_group("canonical_json");
    for size in [100_usize, 500, 1_000, 3_000] {
        let doc = build_doc(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &doc, |b, doc| {
            b.iter(|| {
                let _ = black_box(diagram_models::canonical_json::to_canonical_pretty_json(
                    black_box(doc),
                ));
            })
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_spatial_index_build,
    bench_spatial_index_query,
    bench_evaluate_selection,
    bench_history_push_undo,
    bench_dag_validation,
    bench_canonical_json,
);
criterion_main!(benches);
