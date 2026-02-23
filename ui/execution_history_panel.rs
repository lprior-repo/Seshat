#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

use crate::graph::{NodeId, RunRecord};
use dioxus::prelude::*;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

const fn status_badge_classes(success: bool) -> &'static str {
    if success {
        "bg-emerald-50 text-emerald-700 border-emerald-200"
    } else {
        "bg-red-50 text-red-700 border-red-200"
    }
}

const fn status_icon_classes(success: bool) -> &'static str {
    if success {
        "h-3 w-3 text-emerald-500"
    } else {
        "h-3 w-3 text-red-500"
    }
}

fn format_timestamp(ts: &chrono::DateTime<chrono::Utc>) -> String {
    ts.format("%H:%M:%S").to_string()
}

fn format_elapsed(ts: &chrono::DateTime<chrono::Utc>) -> String {
    let elapsed = chrono::Utc::now().signed_duration_since(*ts);
    if elapsed.num_minutes() < 1 {
        "just now".to_string()
    } else if elapsed.num_hours() < 1 {
        format!("{}m ago", elapsed.num_minutes())
    } else if elapsed.num_days() < 1 {
        format!("{}h ago", elapsed.num_hours())
    } else {
        format!("{}d ago", elapsed.num_days())
    }
}

const fn panel_height_class(collapsed: bool) -> &'static str {
    if collapsed {
        "h-10"
    } else {
        "h-[280px]"
    }
}

const fn chevron_rotation_class(collapsed: bool) -> &'static str {
    if collapsed {
        "-rotate-90"
    } else {
        ""
    }
}

/// Returns the first 8 hex characters of a UUID for compact display.
#[must_use]
pub fn truncate_id(id: &uuid::Uuid) -> String {
    let full = id.to_string();
    // UUID format: xxxxxxxx-xxxx-..., take first 8 chars (no dash)
    full.chars().filter(|c| *c != '-').take(8).collect()
}

/// Human-readable status label for a run.
#[must_use]
pub const fn format_run_status(success: bool) -> &'static str {
    if success {
        "Success"
    } else {
        "Failed"
    }
}

/// Tailwind badge classes for the status pill in the table.
#[must_use]
pub const fn run_status_badge_class(success: bool) -> &'static str {
    if success {
        "inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-[10px] font-semibold bg-emerald-50 text-emerald-700 border-emerald-200"
    } else {
        "inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-[10px] font-semibold bg-red-50 text-red-700 border-red-200"
    }
}

/// Placeholder duration formatter — returns "—" until `RunRecord` carries
/// timing fields.  Replace with real computation once the struct is richer.
#[must_use]
pub fn format_run_duration(_run: &RunRecord) -> String {
    "—".to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{
        format_elapsed, format_run_duration, format_run_status, run_status_badge_class,
        status_badge_classes, truncate_id,
    };
    use crate::graph::RunRecord;
    use std::collections::HashMap;
    use uuid::Uuid;

    fn make_run(success: bool) -> RunRecord {
        RunRecord {
            id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            results: HashMap::new(),
            success,
        }
    }

    #[test]
    fn given_recent_timestamp_when_formatting_elapsed_then_it_returns_just_now() {
        let timestamp = chrono::Utc::now() - chrono::Duration::seconds(30);
        assert_eq!(format_elapsed(&timestamp), "just now");
    }

    #[test]
    fn given_hour_old_timestamp_when_formatting_elapsed_then_it_returns_hours_ago() {
        let timestamp = chrono::Utc::now() - chrono::Duration::hours(2);
        assert_eq!(format_elapsed(&timestamp), "2h ago");
    }

    #[test]
    fn given_success_status_when_requesting_badge_classes_then_success_classes_are_returned() {
        assert_eq!(
            status_badge_classes(true),
            "bg-emerald-50 text-emerald-700 border-emerald-200"
        );
    }

    #[test]
    fn given_uuid_when_truncating_then_first_8_hex_chars_are_returned() {
        // xxxxxxxx-yyyy-... → first 8 chars of no-dash form
        let id =
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap_or_else(|_| Uuid::nil());
        assert_eq!(truncate_id(&id), "550e8400");
    }

    #[test]
    fn given_nil_uuid_when_truncating_then_eight_zeros_are_returned() {
        let id = Uuid::nil();
        assert_eq!(truncate_id(&id), "00000000");
    }

    #[test]
    fn given_success_true_when_formatting_status_then_success_label_is_returned() {
        assert_eq!(format_run_status(true), "Success");
    }

    #[test]
    fn given_success_false_when_formatting_status_then_failed_label_is_returned() {
        assert_eq!(format_run_status(false), "Failed");
    }

    #[test]
    fn given_success_run_when_requesting_badge_class_then_emerald_classes_are_returned() {
        assert!(run_status_badge_class(true).contains("emerald"));
    }

    #[test]
    fn given_failed_run_when_requesting_badge_class_then_red_classes_are_returned() {
        assert!(run_status_badge_class(false).contains("red"));
    }

    #[test]
    fn given_run_record_when_formatting_duration_then_placeholder_is_returned() {
        let run = make_run(true);
        assert_eq!(format_run_duration(&run), "—");
    }
}

// ---------------------------------------------------------------------------
// ExecutionHistoryTable
// ---------------------------------------------------------------------------

/// A standalone full-width table of execution runs.
///
/// # Props
/// - `history` — reactive list of [`RunRecord`] entries (newest first internally)
/// - `active_run_id` — UUID of the run currently loaded in frozen-replay mode
/// - `on_run_select` — called with the [`uuid::Uuid`] when a row is clicked
#[component]
pub fn ExecutionHistoryTable(
    history: ReadSignal<Vec<RunRecord>>,
    active_run_id: ReadSignal<Option<uuid::Uuid>>,
    on_run_select: EventHandler<uuid::Uuid>,
) -> Element {
    rsx! {
        div { class: "w-full overflow-x-auto",
            table { class: "w-full border-collapse text-left",
                thead {
                    tr { class: "bg-slate-50 border-b border-slate-200",
                        th { class: "text-[11px] font-semibold text-slate-500 uppercase tracking-wide px-3 py-2 border-b border-slate-200", "ID" }
                        th { class: "text-[11px] font-semibold text-slate-500 uppercase tracking-wide px-3 py-2 border-b border-slate-200", "Status" }
                        th { class: "text-[11px] font-semibold text-slate-500 uppercase tracking-wide px-3 py-2 border-b border-slate-200", "Start Time" }
                        th { class: "text-[11px] font-semibold text-slate-500 uppercase tracking-wide px-3 py-2 border-b border-slate-200", "Duration" }
                        th { class: "text-[11px] font-semibold text-slate-500 uppercase tracking-wide px-3 py-2 border-b border-slate-200", "Steps OK" }
                        th { class: "text-[11px] font-semibold text-slate-500 uppercase tracking-wide px-3 py-2 border-b border-slate-200", "Steps Failed" }
                    }
                }
                tbody {
                    for run in history.read().iter().rev() {
                        {
                            let run_id = run.id;
                            let is_active = active_run_id.read().is_some_and(|a| a == run_id);
                            let short_id = truncate_id(&run_id);
                            let status_label = format_run_status(run.success);
                            let badge_class = run_status_badge_class(run.success);
                            let start_time = format_timestamp(&run.timestamp);
                            let duration = format_run_duration(run);
                            // Derive steps OK / failed from results map (placeholder logic).
                            // Once RunRecord carries per-step success flags this can be refined.
                            let steps_total = run.results.len();
                            let steps_ok = if run.success { steps_total } else { 0usize };
                            let steps_failed = steps_total - steps_ok;

                            let row_base = "cursor-pointer transition-colors border-b border-slate-100 last:border-b-0";
                            let row_class = if is_active {
                                format!("{row_base} bg-indigo-50 border-l-2 border-indigo-500")
                            } else {
                                format!("{row_base} hover:bg-slate-50")
                            };

                            rsx! {
                                tr {
                                    class: "{row_class}",
                                    key: "{run_id}",
                                    onclick: move |_| { on_run_select.call(run_id); },

                                    td { class: "text-[12px] text-slate-700 px-3 py-2",
                                        span { class: "font-mono", "{short_id}" }
                                    }
                                    td { class: "text-[12px] text-slate-700 px-3 py-2",
                                        span { class: "{badge_class}",
                                            if run.success {
                                                crate::ui::icons::CheckIcon { class: "h-2.5 w-2.5" }
                                            } else {
                                                crate::ui::icons::XCircleIcon { class: "h-2.5 w-2.5" }
                                            }
                                            "{status_label}"
                                        }
                                    }
                                    td { class: "text-[12px] text-slate-700 px-3 py-2 font-mono", "{start_time}" }
                                    td { class: "text-[12px] text-slate-700 px-3 py-2", "{duration}" }
                                    td { class: "text-[12px] text-slate-700 px-3 py-2", "{steps_ok}" }
                                    td { class: "text-[12px] text-slate-700 px-3 py-2", "{steps_failed}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// FrozenModeBanner
// ---------------------------------------------------------------------------

/// Banner shown at the top of the panel when a historical run is active.
#[component]
fn FrozenModeBanner(
    active_run_id: ReadSignal<Option<uuid::Uuid>>,
    on_exit_frozen: EventHandler<()>,
) -> Element {
    let Some(id) = *active_run_id.read() else {
        return rsx! {};
    };
    let short_id = truncate_id(&id);

    rsx! {
        div { class: "flex items-center justify-between px-3 py-2 bg-indigo-50 border-b border-indigo-200 text-[11px]",
            div { class: "flex items-center gap-2",
                div { class: "w-2 h-2 rounded-full bg-indigo-500 animate-pulse" }
                span { class: "text-indigo-700 font-medium",
                    "Viewing historical run "
                    span { class: "font-mono", "{short_id}" }
                    " — Frozen mode"
                }
            }
            button {
                class: "text-indigo-600 hover:text-indigo-800 font-semibold border border-indigo-300 rounded px-2 py-0.5 hover:bg-indigo-100 transition-colors",
                onclick: move |_| { on_exit_frozen.call(()); },
                "Exit frozen mode"
            }
        }
    }
}

// ---------------------------------------------------------------------------
// ExecutionHistoryPanel  (main public component — backward-compatible)
// ---------------------------------------------------------------------------

/// The collapsible execution history panel rendered at the bottom of the
/// canvas layout.
///
/// # New props (v2)
/// - `active_run_id` — which run UUID is currently shown in frozen replay
/// - `on_run_select` — fires when the user clicks a row in the history table
/// - `on_exit_frozen` — fires when the user clicks "Exit frozen mode"
///
/// # Unchanged props
/// - `history`, `nodes_by_id`, `on_select_node`, `collapsed`
#[component]
pub fn ExecutionHistoryPanel(
    history: Memo<Vec<RunRecord>>,
    nodes_by_id: ReadSignal<HashMap<NodeId, crate::graph::Node>>,
    on_select_node: EventHandler<NodeId>,
    collapsed: Signal<bool>,
    /// UUID of the run currently shown in frozen-replay mode, if any.
    active_run_id: ReadSignal<Option<uuid::Uuid>>,
    /// Called when the user clicks a history row.
    on_run_select: EventHandler<uuid::Uuid>,
    /// Called when the user clicks "Exit frozen mode".
    on_exit_frozen: EventHandler<()>,
) -> Element {
    let mut expanded_runs: Signal<std::collections::HashSet<uuid::Uuid>> =
        use_signal(std::collections::HashSet::new);
    let history_len = history.read().len();
    let is_collapsed = *collapsed.read();
    let height_class = panel_height_class(is_collapsed);
    let chevron_class = chevron_rotation_class(is_collapsed);

    // Convert Memo<Vec<RunRecord>> to ReadSignal for the table sub-component.
    let history_signal: ReadSignal<Vec<RunRecord>> = ReadSignal::from(history);

    rsx! {
        aside {
            class: "flex flex-col border-t border-slate-200 bg-white/95 transition-all duration-200 {height_class}",

            // ── Header ──────────────────────────────────────────────────────
            div {
                class: "flex items-center justify-between px-3 py-2 border-b border-slate-100",
                button {
                    class: "flex items-center gap-2 text-slate-700 hover:text-slate-900 transition-colors",
                    onclick: move |_| {
                        let _ = collapsed.try_write().map(|mut c| *c = !*c);
                    },
                    crate::ui::icons::ClockIcon { class: "h-4 w-4 text-slate-500" }
                    span { class: "text-[12px] font-semibold", "Execution History" }
                    span { class: "rounded bg-slate-100 px-1.5 py-0.5 text-[10px] text-slate-600", "{history_len}" }
                    div { class: "transition-transform {chevron_class}",
                        crate::ui::icons::ChevronDownIcon { class: "h-3 w-3 text-slate-400" }
                    }
                }
            }

            if !is_collapsed {
                // ── Frozen-mode banner ───────────────────────────────────────
                FrozenModeBanner {
                    active_run_id,
                    on_exit_frozen,
                }

                // ── Body ─────────────────────────────────────────────────────
                div { class: "flex-1 overflow-y-auto",
                    if history.read().is_empty() {
                        div { class: "flex flex-col items-center justify-center h-full text-center px-4",
                            crate::ui::icons::ClockIcon { class: "h-8 w-8 text-slate-300 mb-2" }
                            p { class: "text-[12px] text-slate-500", "No executions yet" }
                            p { class: "text-[10px] text-slate-400 mt-1", "Run the workflow to see history" }
                        }
                    } else {
                        // ── Table view ────────────────────────────────────────
                        ExecutionHistoryTable {
                            history: history_signal,
                            active_run_id,
                            on_run_select,
                        }

                        // ── Expanded node-detail rows ─────────────────────────
                        // Kept below the table for detail drill-down (click to expand).
                        div { class: "flex flex-col border-t border-slate-100 mt-1",
                            for run in history.read().iter().rev() {
                                {
                                    let run_id = run.id;
                                    let is_expanded = expanded_runs.read().contains(&run_id);
                                    let status_class = status_badge_classes(run.success);
                                    let icon_class = status_icon_classes(run.success);
                                    let timestamp_str = format_timestamp(&run.timestamp);
                                    let elapsed_str = format_elapsed(&run.timestamp);
                                    let node_count = run.results.len();
                                    let item_chevron_class = chevron_rotation_class(!is_expanded);

                                    rsx! {
                                        div {
                                            class: "border-b border-slate-100 last:border-b-0",
                                            key: "{run_id}",

                                            button {
                                                class: "flex w-full items-center gap-2 px-3 py-2 hover:bg-slate-50 transition-colors",
                                                onclick: move |_| {
                                                    let _ = expanded_runs.try_write().map(|mut set| {
                                                        if set.contains(&run_id) {
                                                            set.remove(&run_id);
                                                        } else {
                                                            set.insert(run_id);
                                                        }
                                                    });
                                                },

                                                div { class: "transition-transform {item_chevron_class}",
                                                    crate::ui::icons::ChevronDownIcon { class: "h-3 w-3 text-slate-400" }
                                                }

                                                div { class: "flex-1 flex items-center gap-2",
                                                    span { class: "font-mono text-[11px] text-slate-600", "{timestamp_str}" }
                                                    span { class: "text-[10px] text-slate-400", "{elapsed_str}" }
                                                }

                                                span { class: "text-[10px] text-slate-500", "{node_count} nodes" }

                                                div { class: "flex items-center gap-1 px-1.5 py-0.5 rounded border {status_class}",
                                                    if run.success {
                                                        crate::ui::icons::CheckIcon { class: "{icon_class}" }
                                                    } else {
                                                        crate::ui::icons::XCircleIcon { class: "{icon_class}" }
                                                    }
                                                    span { class: "text-[10px] font-medium",
                                                        if run.success { "Success" } else { "Failed" }
                                                    }
                                                }
                                            }

                                            if is_expanded {
                                                div { class: "bg-slate-50/50 px-3 pb-2",
                                                    for (node_id, result) in run.results.iter() {
                                                        {
                                                            let node_name = nodes_by_id
                                                                .read()
                                                                .get(node_id)
                                                                .map_or_else(
                                                                    || "Unknown".to_string(),
                                                                    |n| n.name.clone(),
                                                                );
                                                            let node_id_for_click = *node_id;
                                                            let result_preview = serde_json::to_string(result)
                                                                .unwrap_or_else(|_| "{}".to_string());
                                                            let truncated_result = if result_preview.len() > 30 {
                                                                format!("{}...", &result_preview[..27])
                                                            } else {
                                                                result_preview
                                                            };

                                                            rsx! {
                                                                button {
                                                                    class: "flex w-full items-center gap-2 px-2 py-1.5 rounded hover:bg-white transition-colors text-left",
                                                                    key: "{node_id}",
                                                                    onclick: move |_| {
                                                                        on_select_node.call(node_id_for_click);
                                                                    },

                                                                    div { class: "w-1.5 h-1.5 rounded-full bg-indigo-400 shrink-0" }
                                                                    span { class: "text-[11px] text-slate-700 flex-1 truncate", "{node_name}" }
                                                                    span { class: "text-[10px] font-mono text-slate-400 shrink-0",
                                                                        "{truncated_result}"
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
