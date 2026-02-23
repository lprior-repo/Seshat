use crate::graph::{Node, NodeCategory};
use dioxus::prelude::*;
use serde_json::Value;

mod common;
mod config_sections;
mod execution;

use common::CommonConfig;
use config_sections::{
    DurableConfig, EntryConfig, FlowConfig, SignalConfig, StateConfig, TimingConfig,
};
use execution::ExecutionTab;

const INPUT_CLASS: &str =
    "h-8 w-full rounded-md border border-slate-700 bg-slate-950 px-3 font-mono text-[12px] text-slate-100 outline-none transition-colors focus:border-indigo-500/50 focus:ring-1 focus:ring-indigo-500/30";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Config,
    Execution,
}

#[component]
pub fn NodeConfigEditor(
    node: Node,
    input_payloads: Vec<Value>,
    on_change: EventHandler<Value>,
) -> Element {
    let mut tab = use_signal(|| Tab::Config);
    let config = node.config.clone();

    let tab_val = *tab.read();

    let config_tab_class = if tab_val == Tab::Config {
        "flex-1 border-b-2 border-indigo-500 py-2 text-[11px] font-medium capitalize text-slate-100 transition-colors"
    } else {
        "flex-1 border-b-2 border-transparent py-2 text-[11px] font-medium capitalize text-slate-500 transition-colors hover:text-slate-300"
    };

    let exec_tab_class = if tab_val == Tab::Execution {
        "flex-1 border-b-2 border-indigo-500 py-2 text-[11px] font-medium capitalize text-slate-100 transition-colors"
    } else {
        "flex-1 border-b-2 border-transparent py-2 text-[11px] font-medium capitalize text-slate-500 transition-colors hover:text-slate-300"
    };

    rsx! {
        div { class: "flex flex-col gap-0",
            div { class: "flex border-b border-slate-800",
                button {
                    class: "{config_tab_class}",
                    onclick: move |_| tab.set(Tab::Config),
                    "Configuration"
                }
                button {
                    class: "{exec_tab_class}",
                    onclick: move |_| tab.set(Tab::Execution),
                    "Execution"
                }
            }

            div { class: "pt-4",
                match tab_val {
                    Tab::Config => rsx! {
                        ConfigTab { node, on_change }
                    },
                    Tab::Execution => rsx! {
                        ExecutionTab {
                            config: config.clone(),
                            last_output: node.last_output,
                            input_payloads,
                            on_pin_sample: EventHandler::new({
                                move |payload: Option<Value>| {
                                    let mut new_config = config.clone();
                                    if let Some(obj) = new_config.as_object_mut() {
                                        match payload {
                                            Some(value) => {
                                                obj.insert("pinnedOutputSample".to_string(), value);
                                            }
                                            None => {
                                                obj.remove("pinnedOutputSample");
                                            }
                                        }
                                        on_change.call(new_config);
                                    }
                                }
                            })
                        }
                    },
                }
            }
        }
    }
}

#[allow(clippy::redundant_clone)]
#[component]
fn ConfigTab(node: Node, on_change: EventHandler<Value>) -> Element {
    let config = node.config.clone();
    let category_config = config.clone();

    let update_str = EventHandler::new({
        let config = config.clone();
        move |(key, val): (String, String)| {
            let mut new_config = config.clone();
            if let Some(obj) = new_config.as_object_mut() {
                obj.insert(key, Value::String(val));
                on_change.call(new_config);
            }
        }
    });

    let update_u64 = EventHandler::new({
        let config = config.clone();
        move |(key, val): (String, u64)| {
            let mut new_config = config.clone();
            if let Some(obj) = new_config.as_object_mut() {
                obj.insert(key, Value::Number(val.into()));
                on_change.call(new_config);
            }
        }
    });

    rsx! {
        div { class: "flex flex-col gap-4",
            match node.category {
                NodeCategory::Entry => rsx! { EntryConfig { icon: node.icon.clone(), config: category_config.clone(), update_str, input_cls: INPUT_CLASS } },
                NodeCategory::Durable => rsx! { DurableConfig { icon: node.icon.clone(), config: category_config.clone(), update_str, input_cls: INPUT_CLASS } },
                NodeCategory::State => rsx! { StateConfig { icon: node.icon.clone(), config: category_config.clone(), update_str, input_cls: INPUT_CLASS } },
                NodeCategory::Flow => rsx! { FlowConfig { icon: node.icon.clone(), config: category_config.clone(), update_str, input_cls: INPUT_CLASS } },
                NodeCategory::Timing => rsx! { TimingConfig { icon: node.icon.clone(), config: category_config.clone(), update_str, update_u64, input_cls: INPUT_CLASS } },
                NodeCategory::Signal => rsx! { SignalConfig { icon: node.icon.clone(), config: category_config.clone(), update_str, input_cls: INPUT_CLASS } },
            }

            div { class: "h-px bg-slate-800" }

            CommonConfig { config, update_u64, update_str, input_cls: INPUT_CLASS }
        }
    }
}

pub(crate) fn get_str_val(config: &Value, key: &str) -> String {
    config
        .get(key)
        .and_then(Value::as_str)
        .map_or("", |value| value)
        .to_string()
}

pub(crate) fn get_u64_val(config: &Value, key: &str) -> Option<u64> {
    config.get(key).and_then(Value::as_u64)
}
