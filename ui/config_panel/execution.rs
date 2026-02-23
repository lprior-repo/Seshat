use super::{get_str_val, get_u64_val};
use crate::ui::icons::{icon_by_name, CopyIcon};
use dioxus::prelude::*;
use serde_json::Value;
use wasm_bindgen::JsCast;
use web_sys::window;

const PINNED_OUTPUT_KEY: &str = "pinnedOutputSample";
const DEFAULT_PREVIEW_LINES: usize = 10;

fn copy_to_clipboard(text: &str) -> bool {
    if let Some(window) = window() {
        let navigator = window.navigator();
        if let Ok(clipboard) =
            js_sys::Reflect::get(&navigator, &js_sys::JsString::from("clipboard"))
        {
            if let Ok(write_text) =
                js_sys::Reflect::get(&clipboard, &js_sys::JsString::from("writeText"))
            {
                if let Some(write_text_fn) = write_text.dyn_ref::<js_sys::Function>() {
                    let _ = write_text_fn.call1(&clipboard, &js_sys::JsString::from(text));
                    return true;
                }
            }
        }
    }
    false
}

#[component]
fn CopyButton(text: String, compact: bool) -> Element {
    let mut show_feedback = use_signal(|| false);

    let btn_class = if compact {
        "flex h-5 w-5 items-center justify-center rounded text-slate-400 transition-colors hover:bg-slate-700 hover:text-slate-200"
    } else {
        "flex h-6 items-center gap-1 rounded px-2 text-[10px] text-slate-400 transition-colors hover:bg-slate-700 hover:text-slate-200"
    };

    rsx! {
        button {
            class: "{btn_class}",
            onclick: move |_| {
                if copy_to_clipboard(&text) {
                    show_feedback.set(true);
                    let mut feedback = show_feedback;
                    wasm_bindgen_futures::spawn_local(async move {
                        gloo_timers::future::TimeoutFuture::new(1500).await;
                        feedback.set(false);
                    });
                }
            },
            if *show_feedback.read() {
                if compact {
                    {icon_by_name("check", "h-3 w-3 text-emerald-400".to_string())}
                } else {
                    span { class: "text-emerald-400", "Copied!" }
                }
            } else {
                CopyIcon { class: if compact { "h-3 w-3" } else { "h-3.5 w-3.5" } }
                if !compact {
                    span { "Copy" }
                }
            }
        }
    }
}

#[component]
fn PayloadPreview(payload: Value, label: String, shape: String, max_lines: usize) -> Element {
    let mut expanded = use_signal(|| false);
    let json_full = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| payload.to_string());
    let lines: Vec<&str> = json_full.lines().collect();
    let display_text = if *expanded.read() {
        json_full.clone()
    } else {
        json_preview(&payload, max_lines)
    };

    rsx! {
        div { class: "rounded-lg border border-slate-700 bg-slate-900/65 p-2",
            div { class: "mb-1 flex items-center justify-between",
                span { class: "text-[10px] font-medium text-slate-300", "{label}" }
                div { class: "flex items-center gap-2",
                    span { class: "rounded bg-slate-800 px-1.5 py-0.5 text-[9px] text-slate-400", "{shape}" }
                    CopyButton { text: json_full.clone(), compact: true }
                }
            }
            pre { class: "overflow-x-auto rounded bg-slate-950 p-2 font-mono text-[10px] leading-relaxed text-slate-300", "{display_text}" }
            if lines.len() > max_lines {
                button {
                    class: "mt-1.5 text-[9px] text-indigo-400 hover:text-indigo-300",
                    onclick: move |_| expanded.toggle(),
                    if *expanded.read() {
                        "Show less"
                    } else {
                        "Show full payload ({lines.len()} lines)"
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
enum ExecutionEventKind {
    Status,
    Journal,
    Retry,
}

#[derive(Clone)]
struct ExecutionTimelineEvent {
    kind: ExecutionEventKind,
    label: String,
    detail: String,
}

#[component]
pub(super) fn ExecutionTab(
    config: Value,
    last_output: Option<Value>,
    input_payloads: Vec<Value>,
    on_pin_sample: EventHandler<Option<Value>>,
) -> Element {
    let status = get_str_val(&config, "status");
    let is_executed = !status.is_empty();

    let journal_idx = get_u64_val(&config, "journalIndex");
    let retry_count = get_u64_val(&config, "retryCount");
    let timeline = build_execution_timeline(&status, journal_idx, retry_count);
    let pinned_output = get_pinned_output(&config);
    let output_payload = last_output.clone().or_else(|| pinned_output.clone());

    rsx! {
        div { class: "flex flex-col gap-4",
            div { class: "flex flex-col gap-2",
                label { class: "text-[11px] font-medium uppercase tracking-wide text-slate-500", "Invocation Status" }
                if is_executed {
                    {
                        let (bg_color, text_color, border_color, icon_name, is_spin) = match status.as_str() {
                            "running" => ("bg-indigo-500/15", "text-indigo-400", "border-indigo-500/30", "loader", true),
                            "suspended" => ("bg-pink-500/15", "text-pink-400", "border-pink-500/30", "pause", false),
                            "completed" => ("bg-emerald-500/15", "text-emerald-400", "border-emerald-500/30", "check-circle", false),
                            "failed" => ("bg-red-500/15", "text-red-400", "border-red-500/30", "alert-circle", false),
                            "retrying" => ("bg-amber-500/15", "text-amber-400", "border-amber-500/30", "refresh", true),
                            _ => ("bg-slate-500/15", "text-slate-400", "border-slate-500/30", "help-circle", false),
                        };
                        let label = match status.as_str() {
                            "running" => "Running",
                            "suspended" => "Suspended",
                            "completed" => "Completed",
                            "failed" => "Failed",
                            "retrying" => "Retrying",
                            other => other,
                        };
                        let icon_class = if is_spin { "h-3 w-3 animate-spin".to_string() } else { "h-3 w-3".to_string() };
                        rsx! {
                            div {
                                class: "inline-flex self-start items-center gap-1.5 rounded-md border px-2.5 py-1 text-[11px] font-medium {bg_color} {text_color} {border_color}",
                                {icon_by_name(icon_name, icon_class)}
                                "{label}"
                            }
                        }
                    }
                } else {
                    span { class: "text-[11px] text-slate-500", "Not yet executed" }
                }
            }

            if let Some(idx) = journal_idx {
                div { class: "flex flex-col gap-1",
                    label { class: "text-[11px] font-medium uppercase tracking-wide text-slate-500", "Journal Entry" }
                    div { class: "flex items-center gap-2",
                        span { class: "rounded bg-slate-800 px-2 py-0.5 font-mono text-[11px] text-slate-300", "#{idx}" }
                        span { class: "text-[10px] text-slate-500", "Position in durable execution log" }
                    }
                }
            }

            if let Some(count) = retry_count {
                if count > 0 {
                    div { class: "flex flex-col gap-1",
                        label { class: "text-[11px] font-medium uppercase tracking-wide text-slate-500", "Retry Attempts" }
                        div { class: "flex items-center gap-2",
                            span { class: "rounded bg-red-500/10 px-2 py-0.5 font-mono text-[11px] text-red-400", "{count}" }
                            span { class: "text-[10px] text-slate-500", "Times retried before success/failure" }
                        }
                    }
                }
            }

            div { class: "h-px bg-slate-800" }

            div { class: "flex flex-col gap-2",
                div { class: "flex items-center justify-between",
                    h4 { class: "text-[11px] font-semibold uppercase tracking-wide text-slate-300", "Input Payloads" }
                    span { class: "rounded bg-slate-800 px-1.5 py-0.5 text-[10px] text-slate-400", "{input_payloads.len()}" }
                }
                if input_payloads.is_empty() {
                    p { class: "rounded-lg border border-dashed border-slate-700 bg-slate-800/50 px-3 py-2 text-[11px] text-slate-500", "No upstream payloads available yet." }
                } else {
                    div { class: "flex flex-col gap-2",
                        for (index, payload) in input_payloads.iter().enumerate() {
                            PayloadPreview {
                                payload: payload.clone(),
                                label: format!("Input #{}", index + 1),
                                shape: payload_shape(payload),
                                max_lines: DEFAULT_PREVIEW_LINES,
                            }
                        }
                    }
                }
            }

            div { class: "h-px bg-slate-800" }

            div { class: "flex flex-col gap-2",
                div { class: "flex items-center justify-between",
                    h4 { class: "text-[11px] font-semibold uppercase tracking-wide text-slate-300", "Output Payload" }
                    span { class: "rounded bg-slate-800 px-1.5 py-0.5 text-[10px] text-slate-400", "{output_origin_label(last_output.is_some(), pinned_output.is_some())}" }
                }

                if let Some(output) = output_payload.as_ref() {
                    PayloadPreview {
                        payload: output.clone(),
                        label: "Payload".to_string(),
                        shape: payload_shape(output),
                        max_lines: 14,
                    }
                } else {
                    p { class: "rounded-lg border border-dashed border-slate-700 bg-slate-800/50 px-3 py-2 text-[11px] text-slate-500", "Run the workflow to inspect output for this node." }
                }

                div { class: "flex items-center gap-2",
                    button {
                        class: "h-7 rounded-md border border-indigo-500/40 bg-indigo-500/10 px-2.5 text-[10px] font-medium text-indigo-300 transition-colors hover:bg-indigo-500/20 disabled:cursor-not-allowed disabled:opacity-50",
                        disabled: last_output.is_none(),
                        onclick: move |_| {
                            if let Some(output) = last_output.clone() {
                                on_pin_sample.call(Some(output));
                            }
                        },
                        "Pin latest output"
                    }
                    if pinned_output.is_some() {
                        button {
                            class: "h-7 rounded-md border border-slate-600 bg-slate-800/60 px-2.5 text-[10px] font-medium text-slate-300 transition-colors hover:bg-slate-700/60",
                            onclick: move |_| on_pin_sample.call(None),
                            "Unpin"
                        }
                    }
                }
            }

            div { class: "h-px bg-slate-800" }

            div { class: "rounded-lg border border-slate-700 bg-slate-900/65 p-3",
                div { class: "mb-2 flex items-center justify-between",
                    h4 { class: "text-[11px] font-semibold uppercase tracking-wide text-slate-300", "Execution Timeline" }
                    span { class: "rounded bg-slate-800 px-1.5 py-0.5 text-[10px] text-slate-400", "{timeline.len()}" }
                }
                if timeline.is_empty() {
                    p { class: "text-[11px] text-slate-500", "No execution telemetry yet." }
                } else {
                    div { class: "flex flex-col gap-1.5",
                        for event in timeline.iter() {
                            {
                                let (dot_class, pill_class) = execution_event_style(event.kind);
                                rsx! {
                                    div { class: "flex gap-2 rounded-md border border-slate-700 bg-slate-900/80 px-2 py-1.5",
                                        div { class: "mt-[2px] h-2 w-2 rounded-full {dot_class}" }
                                        div {
                                            p { class: "text-[10px] font-medium text-slate-200", "{event.label}" }
                                            p { class: "text-[10px] text-slate-400", "{event.detail}" }
                                        }
                                        span { class: "ml-auto inline-flex h-fit rounded px-1.5 py-0.5 text-[9px] font-medium {pill_class}", "{event_kind_label(event.kind)}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            div { class: "rounded-lg border border-dashed border-slate-700 bg-slate-800/50 p-3",
                p { class: "text-[11px] leading-relaxed text-slate-400", "Restate persists each step in a durable journal. On failure, execution replays from the journal, skipping already-completed steps. This ensures exactly-once semantics." }
            }
        }
    }
}

fn build_execution_timeline(
    status: &str,
    journal_idx: Option<u64>,
    retry_count: Option<u64>,
) -> Vec<ExecutionTimelineEvent> {
    let mut events = Vec::new();

    if !status.is_empty() {
        events.push(ExecutionTimelineEvent {
            kind: ExecutionEventKind::Status,
            label: "Invocation status updated".to_string(),
            detail: status.to_string(),
        });
    }

    if let Some(index) = journal_idx {
        events.push(ExecutionTimelineEvent {
            kind: ExecutionEventKind::Journal,
            label: "Durable journal checkpoint".to_string(),
            detail: format!("journal #{index}"),
        });
    }

    if let Some(retry) = retry_count {
        if retry > 0 {
            events.push(ExecutionTimelineEvent {
                kind: ExecutionEventKind::Retry,
                label: "Retry attempts recorded".to_string(),
                detail: format!("{retry} retries"),
            });
        }
    }

    events
}

const fn execution_event_style(kind: ExecutionEventKind) -> (&'static str, &'static str) {
    match kind {
        ExecutionEventKind::Status => ("bg-indigo-500", "bg-indigo-500/15 text-indigo-300"),
        ExecutionEventKind::Journal => ("bg-emerald-500", "bg-emerald-500/15 text-emerald-300"),
        ExecutionEventKind::Retry => ("bg-amber-500", "bg-amber-500/15 text-amber-300"),
    }
}

const fn event_kind_label(kind: ExecutionEventKind) -> &'static str {
    match kind {
        ExecutionEventKind::Status => "Status",
        ExecutionEventKind::Journal => "Journal",
        ExecutionEventKind::Retry => "Retry",
    }
}

fn get_pinned_output(config: &Value) -> Option<Value> {
    config.get(PINNED_OUTPUT_KEY).cloned()
}

const fn output_origin_label(has_live_output: bool, has_pinned_output: bool) -> &'static str {
    if has_live_output {
        "Live output"
    } else if has_pinned_output {
        "Pinned sample"
    } else {
        "No output"
    }
}

fn payload_shape(payload: &Value) -> String {
    match payload {
        Value::Object(map) => format!("object ({})", map.len()),
        Value::Array(arr) => format!("array ({})", arr.len()),
        Value::String(text) => format!("string ({})", text.len()),
        Value::Number(_) => "number".to_string(),
        Value::Bool(_) => "boolean".to_string(),
        Value::Null => "null".to_string(),
    }
}

fn json_preview(payload: &Value, max_lines: usize) -> String {
    let pretty = serde_json::to_string_pretty(payload).unwrap_or_else(|_| payload.to_string());
    let lines: Vec<&str> = pretty.lines().collect();
    if lines.len() <= max_lines {
        return pretty;
    }

    let mut preview = lines[..max_lines].join("\n");
    preview.push_str("\n... (truncated)");
    preview
}

#[cfg(test)]
mod tests {
    use super::{
        build_execution_timeline, get_pinned_output, json_preview, output_origin_label,
        payload_shape, ExecutionEventKind,
    };
    use serde_json::json;

    #[test]
    fn timeline_includes_status_journal_and_retry_when_present() {
        let timeline = build_execution_timeline("retrying", Some(7), Some(2));

        assert_eq!(timeline.len(), 3);
        assert!(matches!(timeline[0].kind, ExecutionEventKind::Status));
        assert!(timeline[1].detail.contains("#7"));
        assert!(matches!(timeline[2].kind, ExecutionEventKind::Retry));
    }

    #[test]
    fn timeline_skips_retry_when_zero() {
        let timeline = build_execution_timeline("running", Some(1), Some(0));

        assert_eq!(timeline.len(), 2);
        assert!(!timeline
            .iter()
            .any(|entry| matches!(entry.kind, ExecutionEventKind::Retry)));
    }

    #[test]
    fn pinned_output_is_read_from_config() {
        let config = json!({"pinnedOutputSample": {"ok": true}});
        assert_eq!(get_pinned_output(&config), Some(json!({"ok": true})));
    }

    #[test]
    fn output_origin_prefers_live_data() {
        assert_eq!(output_origin_label(true, true), "Live output");
        assert_eq!(output_origin_label(false, true), "Pinned sample");
        assert_eq!(output_origin_label(false, false), "No output");
    }

    #[test]
    fn payload_shape_reports_kind_and_size() {
        assert_eq!(payload_shape(&json!({"a": 1, "b": 2})), "object (2)");
        assert_eq!(payload_shape(&json!([1, 2, 3])), "array (3)");
        assert_eq!(payload_shape(&json!("hello")), "string (5)");
    }

    #[test]
    fn json_preview_truncates_large_payloads() {
        let payload = json!({
            "a": 1,
            "b": 2,
            "c": 3,
            "d": 4,
            "e": 5,
            "f": 6,
            "g": 7,
            "h": 8,
            "i": 9,
            "j": 10,
            "k": 11,
            "l": 12,
            "m": 13
        });

        let preview = json_preview(&payload, 6);
        assert!(preview.contains("... (truncated)"));
    }
}
