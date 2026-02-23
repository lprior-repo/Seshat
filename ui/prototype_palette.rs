#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use dioxus::prelude::*;
use std::fmt::Write as _;

/// A node the user has added to their sketch during prototype mode.
#[derive(Clone, PartialEq, Eq)]
pub struct SketchNode {
    pub node_type: &'static str,
    pub label: String,
}

/// Palette entry: every node type available in prototype mode.
#[derive(Clone, Copy)]
struct PaletteEntry {
    node_type: &'static str,
    label: &'static str,
    icon: &'static str,
}

const PALETTE_ENTRIES: [PaletteEntry; 9] = [
    PaletteEntry {
        node_type: "http-handler",
        label: "HTTP Handler",
        icon: "🌐",
    },
    PaletteEntry {
        node_type: "run",
        label: "Durable Step",
        icon: "🛡️",
    },
    PaletteEntry {
        node_type: "sleep",
        label: "Sleep / Timer",
        icon: "⏱️",
    },
    PaletteEntry {
        node_type: "set-state",
        label: "Set State",
        icon: "⬆️",
    },
    PaletteEntry {
        node_type: "get-state",
        label: "Get State",
        icon: "⬇️",
    },
    PaletteEntry {
        node_type: "send-message",
        label: "Send Message",
        icon: "📤",
    },
    PaletteEntry {
        node_type: "awakeable",
        label: "Awakeable",
        icon: "🔔",
    },
    PaletteEntry {
        node_type: "parallel",
        label: "Parallel",
        icon: "⫿",
    },
    PaletteEntry {
        node_type: "condition",
        label: "If / Else",
        icon: "⑂",
    },
];

// ── Pure skeleton generator ────────────────────────────────────────────────────

/// Generates a YAML workflow skeleton from a slice of sketch nodes.
/// The resulting workflow is a linear chain: each step `depends_on` the one
/// before it.
#[must_use]
pub fn generate_skeleton(nodes: &[SketchNode]) -> String {
    let mut out = String::from("name: \"prototype-workflow\"\nsteps:\n");

    for (i, node) in nodes.iter().enumerate() {
        let step_id = format!("step-{}", i + 1);
        let _ = writeln!(out, "  - id: {step_id}");
        let _ = writeln!(out, "    type: {}", node.node_type);
        if i > 0 {
            let prev_id = format!("step-{i}");
            let _ = writeln!(out, "    depends_on: [{prev_id}]");
        }
        out.push_str("    config: {}\n");
    }

    out
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn PrototypePalette(
    open: ReadSignal<bool>,
    on_close: EventHandler<()>,
    on_add_node: EventHandler<&'static str>,
) -> Element {
    if !*open.read() {
        return rsx! {};
    }

    let mut sketch_nodes: Signal<Vec<SketchNode>> = use_signal(Vec::new);
    let mut generated_skeleton: Signal<Option<String>> = use_signal(|| None);

    let nodes_snapshot = sketch_nodes.read().clone();
    let skeleton_snapshot = generated_skeleton.read().clone();

    rsx! {
        // Full-screen backdrop
        div {
            class: "fixed inset-0 z-50 bg-black/40 backdrop-blur-sm flex items-start justify-center overflow-y-auto",
            onclick: move |_| on_close.call(()),

            // Centered panel (stop propagation so clicks inside don't close)
            div {
                class: "relative w-full max-w-2xl mx-auto mt-24 mb-12 bg-white rounded-xl shadow-2xl overflow-hidden",
                onclick: move |evt| evt.stop_propagation(),

                // ── Header ────────────────────────────────────────────────────
                div {
                    class: "flex items-start justify-between px-6 py-5 border-b border-slate-200",
                    div {
                        h2 { class: "text-lg font-bold text-slate-900", "Prototype Mode" }
                        p {
                            class: "mt-0.5 text-sm text-slate-500",
                            "Sketch your workflow, then generate a code skeleton"
                        }
                    }
                    button {
                        class: "rounded-lg border border-slate-200 px-3 py-1.5 text-xs font-medium text-slate-500 transition-colors hover:border-slate-400 hover:text-slate-700",
                        onclick: move |_| on_close.call(()),
                        "Close"
                    }
                }

                // ── Node palette grid ─────────────────────────────────────────
                div { class: "px-6 py-4 border-b border-slate-100",
                    h3 { class: "mb-3 text-xs font-semibold uppercase tracking-wide text-slate-400", "Node Types" }
                    div { class: "grid grid-cols-3 gap-2",
                        for entry in PALETTE_ENTRIES {
                            button {
                                key: "{entry.node_type}",
                                class: "flex flex-col items-center gap-1.5 rounded-lg border border-slate-200 bg-slate-50 px-3 py-3 text-center transition-colors hover:border-indigo-300 hover:bg-indigo-50 hover:text-indigo-700 active:scale-95",
                                onclick: move |_| {
                                    let new_node = SketchNode {
                                        node_type: entry.node_type,
                                        label: entry.label.to_string(),
                                    };
                                    sketch_nodes.write().push(new_node);
                                    // reset skeleton when nodes change
                                    *generated_skeleton.write() = None;
                                    on_add_node.call(entry.node_type);
                                },
                                span { class: "text-2xl leading-none", "{entry.icon}" }
                                span { class: "font-mono text-[10px] text-slate-500", "{entry.node_type}" }
                                span { class: "text-[11px] font-medium text-slate-700", "{entry.label}" }
                            }
                        }
                    }
                }

                // ── Sketch summary (chips) ────────────────────────────────────
                div { class: "px-6 py-4 border-b border-slate-100 min-h-[64px]",
                    h3 { class: "mb-2 text-xs font-semibold uppercase tracking-wide text-slate-400", "Sketch" }
                    if nodes_snapshot.is_empty() {
                        p { class: "text-sm text-slate-400 italic", "Click node types above to add them." }
                    } else {
                        div { class: "flex flex-wrap gap-2",
                            for (idx, node) in nodes_snapshot.iter().enumerate() {
                                div {
                                    key: "{idx}",
                                    class: "inline-flex items-center gap-1 rounded-full bg-indigo-100 px-3 py-1 text-xs font-medium text-indigo-800",
                                    span { "{node.node_type}" }
                                    button {
                                        class: "ml-1 rounded-full text-indigo-500 hover:text-indigo-900 leading-none",
                                        "aria-label": "Remove {node.node_type}",
                                        onclick: move |_| {
                                            let mut nodes = sketch_nodes.write();
                                            if idx < nodes.len() {
                                                nodes.remove(idx);
                                            }
                                            *generated_skeleton.write() = None;
                                        },
                                        "×"
                                    }
                                }
                            }
                        }
                    }
                }

                // ── Generate button ───────────────────────────────────────────
                div { class: "px-6 py-4",
                    if !nodes_snapshot.is_empty() {
                        button {
                            class: "w-full rounded-lg bg-indigo-600 px-4 py-2.5 text-sm font-semibold text-white shadow-sm transition-colors hover:bg-indigo-700 active:bg-indigo-800",
                            onclick: move |_| {
                                let nodes = sketch_nodes.read().clone();
                                *generated_skeleton.write() = Some(generate_skeleton(&nodes));
                            },
                            "Generate Code Skeleton"
                        }
                    }

                    // ── Skeleton output ───────────────────────────────────────
                    if let Some(ref skeleton) = skeleton_snapshot {
                        div { class: "mt-4",
                            div { class: "mb-2 flex items-center justify-between",
                                h3 { class: "text-xs font-semibold uppercase tracking-wide text-slate-400", "Generated Skeleton" }
                                button {
                                    class: "rounded border border-slate-200 px-2.5 py-1 text-xs font-medium text-slate-500 transition-colors hover:border-slate-400 hover:text-slate-700",
                                    onclick: {
                                        let skeleton_to_copy = skeleton.clone();
                                        move |_| {
                                            // Clipboard write via eval
                                            let js = format!(
                                                "navigator.clipboard.writeText({skeleton_to_copy:?}).catch(()=>{{}})"
                                            );
                                            document::eval(&js);
                                        }
                                    },
                                    "Copy"
                                }
                            }
                            pre {
                                class: "overflow-x-auto rounded-lg bg-slate-900 p-4 font-mono text-xs text-green-400 whitespace-pre",
                                "{skeleton}"
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::{generate_skeleton, SketchNode};

    fn node(node_type: &'static str) -> SketchNode {
        SketchNode {
            node_type,
            label: node_type.to_string(),
        }
    }

    #[test]
    fn given_empty_nodes_when_generating_skeleton_then_produces_header_only() {
        let result = generate_skeleton(&[]);
        assert!(result.contains("name: \"prototype-workflow\""));
        assert!(result.contains("steps:"));
        // No step entries
        assert!(!result.contains("step-1"));
    }

    #[test]
    fn given_two_nodes_when_generating_skeleton_then_second_has_depends_on() {
        let nodes = vec![node("http-handler"), node("run")];
        let result = generate_skeleton(&nodes);

        assert!(result.contains("id: step-1"));
        assert!(result.contains("type: http-handler"));
        assert!(result.contains("id: step-2"));
        assert!(result.contains("type: run"));
        // step-1 has no depends_on, step-2 does
        assert!(!result
            .lines()
            .take_while(|l| !l.contains("step-2"))
            .any(|l| l.contains("depends_on")));
        assert!(result.contains("depends_on: [step-1]"));
    }

    #[test]
    fn given_three_nodes_when_generating_skeleton_then_linear_chain_is_correct() {
        let nodes = vec![node("http-handler"), node("run"), node("sleep")];
        let result = generate_skeleton(&nodes);

        // Collect lines for structural inspection
        let lines: Vec<&str> = result.lines().collect();

        // step-1: no depends_on
        let step1_block: Vec<&str> = lines
            .iter()
            .skip_while(|l| !l.contains("id: step-1"))
            .take_while(|l| !l.contains("id: step-2"))
            .copied()
            .collect();
        assert!(!step1_block.iter().any(|l| l.contains("depends_on")));

        // step-2: depends on step-1
        let step2_block: Vec<&str> = lines
            .iter()
            .skip_while(|l| !l.contains("id: step-2"))
            .take_while(|l| !l.contains("id: step-3"))
            .copied()
            .collect();
        assert!(step2_block
            .iter()
            .any(|l| l.contains("depends_on: [step-1]")));

        // step-3: depends on step-2
        let step3_block: Vec<&str> = lines
            .iter()
            .skip_while(|l| !l.contains("id: step-3"))
            .copied()
            .collect();
        assert!(step3_block
            .iter()
            .any(|l| l.contains("depends_on: [step-2]")));
        assert!(step3_block.iter().any(|l| l.contains("type: sleep")));
    }
}
