use crate::ui::workflow_nodes::schema::LoadFromMemoryConfig;
use dioxus::prelude::*;

fn json_to_display(value: &serde_json::Value) -> String {
    if let Ok(value) = serde_json::to_string_pretty(value) {
        value
    } else {
        String::new()
    }
}

fn optional_json_to_display(value: Option<&serde_json::Value>) -> String {
    if let Some(value) = value {
        json_to_display(value)
    } else {
        String::new()
    }
}

#[component]
pub fn LoadFromMemoryForm(config: Signal<LoadFromMemoryConfig>) -> Element {
    let pretty_default = optional_json_to_display(config.read().default.as_ref());

    rsx! {
        div {
            class: "space-y-4",

            div {
                class: "bg-cyan-50 border border-cyan-200 rounded-lg p-3 mb-4",
                p {
                    class: "text-sm text-cyan-800",
                    "📂 ",
                    strong { "Load Data" },
                    " - Get a value you saved earlier."
                }
            }

            div {
                class: "form-field",
                label {
                    class: "block text-sm font-medium text-gray-700 mb-1",
                    "Which saved item?"
                }
                input {
                    r#type: "text",
                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-cyan-500",
                    placeholder: "e.g., order_total, user_email",
                    value: "{config.read().key}",
                    oninput: move |e| {
                        config.write().key = e.value().clone();
                    }
                }
                p {
                    class: "text-xs text-gray-500 mt-1",
                    "Name you used when saving"
                }
            }

            div {
                class: "form-field",
                label {
                    class: "block text-sm font-medium text-gray-700 mb-1",
                    "Default value (if not found)"
                }
                textarea {
                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-cyan-500 font-mono text-sm",
                    rows: 2,
                    placeholder: "null",
                    value: "{pretty_default}",
                    oninput: move |e| {
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&e.value()) {
                            config.write().default = Some(v);
                        }
                    }
                }
                p {
                    class: "text-xs text-gray-500 mt-1",
                    "Invalid JSON is ignored to preserve last valid value"
                }
            }
        }
    }
}

#[component]
pub fn LoadFromMemoryNodeCard() -> Element {
    rsx! {
        div {
            class: "flex items-center gap-3 p-3 bg-white border border-gray-200 rounded-lg shadow-sm hover:shadow-md transition-shadow",

            div {
                class: "w-10 h-10 bg-cyan-100 rounded-full flex items-center justify-center",
                span {
                    class: "text-xl",
                    "📂"
                }
            },

            div {
                class: "flex-1",
                h3 {
                    class: "font-medium text-gray-900",
                    "Load from Memory"
                }
                p {
                    class: "text-sm text-gray-500",
                    "Get a saved value"
                }
            }
        }
    }
}
