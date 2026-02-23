use super::{get_str_val, get_u64_val};
use dioxus::prelude::*;
use serde_json::Value;

#[component]
pub(super) fn EntryConfig(
    icon: String,
    config: Value,
    update_str: EventHandler<(String, String)>,
    input_cls: &'static str,
) -> Element {
    rsx! {
        match icon.as_str() {
            "clock" => rsx! {
                div { class: "flex flex-col gap-1.5",
                    label { class: "text-[11px] font-medium uppercase tracking-wide text-slate-500", "Cron Expression" }
                    input {
                        class: "{input_cls}",
                        placeholder: "\"0 */5 * * *\" (every 5 min)",
                        value: "{get_str_val(&config, \"cronExpression\")}",
                        oninput: move |e| update_str.call(("cronExpression".to_string(), e.value()))
                    }
                }
            },
            "kafka" => rsx! {
                div { class: "flex flex-col gap-1.5",
                    label { class: "text-[11px] font-medium uppercase tracking-wide text-slate-500", "Kafka Topic" }
                    input {
                        class: "{input_cls}",
                        placeholder: "orders-topic",
                        value: "{get_str_val(&config, \"topic\")}",
                        oninput: move |e| update_str.call(("topic".to_string(), e.value()))
                    }
                }
            },
            "play-circle" => rsx! {
                div { class: "flex flex-col gap-1.5",
                    label { class: "text-[11px] font-medium uppercase tracking-wide text-slate-500", "Workflow Key" }
                    input {
                        class: "{input_cls}",
                        placeholder: "user-123",
                        value: "{get_str_val(&config, \"workflowKey\")}",
                        oninput: move |e| update_str.call(("workflowKey".to_string(), e.value()))
                    }
                }
            },
            _ => rsx! {},
        }
    }
}

#[component]
pub(super) fn DurableConfig(
    icon: String,
    config: Value,
    update_str: EventHandler<(String, String)>,
    input_cls: &'static str,
) -> Element {
    rsx! {
        div { class: "flex flex-col gap-1.5",
            label { class: "text-[11px] font-medium uppercase tracking-wide text-slate-500", "Step Name" }
            input {
                class: "{input_cls}",
                placeholder: "e.g. \"create-user\"",
                value: "{get_str_val(&config, \"durableStepName\")}",
                oninput: move |e| update_str.call(("durableStepName".to_string(), e.value()))
            }
            span { class: "font-mono text-[10px] text-slate-500", "ctx.run(\"name\", () => ...)" }
        }

        div { class: "flex flex-col gap-1.5",
            label { class: "text-[11px] font-medium uppercase tracking-wide text-slate-500", "Target Service" }
            input {
                class: "{input_cls}",
                placeholder: "PaymentService",
                value: "{get_str_val(&config, \"targetService\")}",
                oninput: move |e| update_str.call(("targetService".to_string(), e.value()))
            }
            span { class: "font-mono text-[10px] text-slate-500", "ctx.serviceClient<T>(\"name\")" }
        }

        div { class: "flex flex-col gap-1.5",
            label { class: "text-[11px] font-medium uppercase tracking-wide text-slate-500", "Handler / Method" }
            input {
                class: "{input_cls}",
                placeholder: "processPayment",
                value: "{get_str_val(&config, \"targetHandler\")}",
                oninput: move |e| update_str.call(("targetHandler".to_string(), e.value()))
            }
            span { class: "font-mono text-[10px] text-slate-500", ".processPayment(req)" }
        }

        if icon == "send" {
            div { class: "rounded-lg border border-dashed border-indigo-500/30 bg-indigo-500/5 p-2",
                p { class: "text-[10px] text-slate-400", "Fire-and-forget: ctx.objectSendClient<T>(key).method(req)" }
            }
        }

        if icon == "clock-send" {
            div { class: "flex flex-col gap-1.5",
                label { class: "text-[11px] font-medium uppercase tracking-wide text-slate-500", "Delay Duration" }
                input {
                    class: "{input_cls}",
                    placeholder: "\"1h\", \"30m\"",
                    value: "{get_str_val(&config, \"sleepDuration\")}",
                    oninput: move |e| update_str.call(("sleepDuration".to_string(), e.value()))
                }
                span { class: "font-mono text-[10px] text-slate-500", "ctx.objectSendClient(key, {{ delay: ... }})" }
            }
        }
    }
}

#[component]
pub(super) fn StateConfig(
    icon: String,
    config: Value,
    update_str: EventHandler<(String, String)>,
    input_cls: &'static str,
) -> Element {
    rsx! {
        div { class: "flex flex-col gap-1.5",
            label { class: "text-[11px] font-medium uppercase tracking-wide text-slate-500", "State Key" }
            input {
                class: "{input_cls}",
                placeholder: "\"cart\", \"profile\"",
                value: "{get_str_val(&config, \"stateKey\")}",
                oninput: move |e| update_str.call(("stateKey".to_string(), e.value()))
            }
        }
        div { class: "rounded-lg border border-dashed border-orange-500/30 bg-orange-500/5 p-2",
            p { class: "font-mono text-[10px] leading-relaxed text-slate-400",
                if icon == "download" { "await ctx.get<T>(\"key\")" }
                else if icon == "upload" { "ctx.set(\"key\", value)" }
                else if icon == "eraser" { "ctx.clear(\"key\") | clearAll()" }
            }
        }
    }
}

#[component]
pub(super) fn FlowConfig(
    icon: String,
    config: Value,
    update_str: EventHandler<(String, String)>,
    input_cls: &'static str,
) -> Element {
    rsx! {
        match icon.as_str() {
            "git-branch" => rsx! {
                div { class: "flex flex-col gap-1.5",
                    label { class: "text-[11px] font-medium uppercase tracking-wide text-slate-500", "Condition Expression" }
                    textarea {
                        class: "resize-none rounded-md border border-slate-700 bg-slate-950 px-3 py-2 font-mono text-[11px] text-slate-100 outline-none transition-colors focus:border-indigo-500/50 focus:ring-1 focus:ring-indigo-500/30",
                        rows: "2",
                        placeholder: "user.verified === true",
                        value: "{get_str_val(&config, \"conditionExpression\")}",
                        oninput: move |e| update_str.call(("conditionExpression".to_string(), e.value()))
                    }
                }
            },
            "repeat" => rsx! {
                div { class: "flex flex-col gap-1.5",
                    label { class: "text-[11px] font-medium uppercase tracking-wide text-slate-500", "Loop Iterator" }
                    input {
                        class: "{input_cls}",
                        placeholder: "items, userIds, tasks",
                        value: "{get_str_val(&config, \"loopIterator\")}",
                        oninput: move |e| update_str.call(("loopIterator".to_string(), e.value()))
                    }
                    span { class: "font-mono text-[10px] text-slate-500", "for (const item of items) {{ ... }}" }
                }
            },
            "undo" => rsx! {
                div { class: "flex flex-col gap-1.5",
                    label { class: "text-[11px] font-medium uppercase tracking-wide text-slate-500", "Compensation Handler" }
                    input {
                        class: "{input_cls}",
                        placeholder: "refundPayment",
                        value: "{get_str_val(&config, \"compensationHandler\")}",
                        oninput: move |e| update_str.call(("compensationHandler".to_string(), e.value()))
                    }
                    span { class: "text-[10px] text-slate-500", "Saga rollback logic" }
                }
            },
            _ => rsx! {},
        }
    }
}

#[component]
pub(super) fn TimingConfig(
    icon: String,
    config: Value,
    update_str: EventHandler<(String, String)>,
    update_u64: EventHandler<(String, u64)>,
    input_cls: &'static str,
) -> Element {
    rsx! {
        match icon.as_str() {
            "timer" => rsx! {
                div { class: "flex flex-col gap-1.5",
                    label { class: "text-[11px] font-medium uppercase tracking-wide text-slate-500", "Sleep Duration" }
                    input {
                        class: "{input_cls}",
                        placeholder: "\"5m\", \"1h\", \"30s\"",
                        value: "{get_str_val(&config, \"sleepDuration\")}",
                        oninput: move |e| update_str.call(("sleepDuration".to_string(), e.value()))
                    }
                    span { class: "font-mono text-[10px] text-slate-500", "await ctx.sleep(duration)" }
                }
            },
            "alarm" => rsx! {
                div { class: "flex flex-col gap-1.5",
                    label { class: "text-[11px] font-medium uppercase tracking-wide text-slate-500", "Timeout (ms)" }
                    input {
                        r#type: "number",
                        class: "{input_cls}",
                        placeholder: "30000",
                        value: "{get_u64_val(&config, \"timeoutMs\").map_or_else(|| 30000, |v| v)}",
                        oninput: move |e| {
                            if let Ok(val) = e.value().parse::<u64>() {
                                update_u64.call(("timeoutMs".to_string(), val));
                            }
                        }
                    }
                    span { class: "font-mono text-[10px] text-slate-500", "promise.orTimeout(ms)" }
                }
            },
            _ => rsx! {},
        }
    }
}

#[component]
pub(super) fn SignalConfig(
    icon: String,
    config: Value,
    update_str: EventHandler<(String, String)>,
    input_cls: &'static str,
) -> Element {
    rsx! {
        if icon == "sparkles" || icon == "bell" {
            div { class: "flex flex-col gap-1.5",
                label { class: "text-[11px] font-medium uppercase tracking-wide text-slate-500", "Promise Name" }
                input {
                    class: "{input_cls}",
                    placeholder: "\"payment-completed\"",
                    value: "{get_str_val(&config, \"promiseName\")}",
                    oninput: move |e| update_str.call(("promiseName".to_string(), e.value()))
                }
                span { class: "font-mono text-[10px] text-slate-500",
                    if icon == "sparkles" { "await ctx.promise<T>(\"name\")" }
                    else { "const {{ id, promise }} = ctx.awakeable<T>()" }
                }
            }
        } else if icon == "check-circle" {
            div { class: "flex flex-col gap-1.5",
                label { class: "text-[11px] font-medium uppercase tracking-wide text-slate-500", "Promise to Resolve" }
                input {
                    class: "{input_cls}",
                    placeholder: "\"payment-completed\"",
                    value: "{get_str_val(&config, \"promiseName\")}",
                    oninput: move |e| update_str.call(("promiseName".to_string(), e.value()))
                }
                span { class: "font-mono text-[10px] text-slate-500", "ctx.promiseManager().resolve(\"name\", val)" }
            }
        }
        div { class: "rounded-lg border border-dashed border-blue-500/30 bg-blue-500/5 p-2",
            p { class: "text-[10px] leading-relaxed text-slate-400", "Durable promises suspend execution until resolved externally via HTTP or SDK." }
        }
    }
}
