use crate::ui::workflow_nodes::schema::SaveToMemoryConfig;
use dioxus::prelude::*;

fn json_to_display(value: &serde_json::Value) -> String {
    if let Ok(value) = serde_json::to_string_pretty(value) {
        value
    } else {
        String::new()
    }
}

#[component]
pub fn SaveToMemoryForm(config: Signal<SaveToMemoryConfig>) -> Element {
    let pretty_value = json_to_display(&*config.read().value);

    rsx! {
        div {
            class: "space-y-4",

            div {
                class: "bg-indigo-50 border border-indigo-200 rounded-lg p-3 mb-4",
                p {
                    class: "text-sm text-indigo-800",
                    "💾 ",
                    strong { "Save Data" },
                    " - Store a value you can use later."
                }
            }

            div {
                class: "form-field",
                label {
                    class: "block text-sm font-medium text-gray-700 mb-1",
                    "What name should this have?"
                }
                input {
                    r#type: "text",
                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500",
                    placeholder: "e.g., order_total, user_email, approval_status",
                    value: "{config.read().key}",
                    oninput: move |e| {
                        config.write().key = e.value().clone();
                    }
                }
                p {
                    class: "text-xs text-gray-500 mt-1",
                    "Use this name to load the data later"
                }
            }

            div {
                class: "form-field",
                label {
                    class: "block text-sm font-medium text-gray-700 mb-1",
                    "What to save (JSON)"
                }
                textarea {
                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500 font-mono text-sm",
                    rows: 4,
                    placeholder: r#"{"amount": 100, "currency": "USD"}"#,
                    value: "{pretty_value}",
                    oninput: move |e| {
                        if let Ok(v) = serde_json::from_str(&e.value()) {
                            config.write().value = v;
                        }
                    },
                }
                p {
                    class: "text-xs text-gray-500 mt-1",
                    "Invalid JSON is ignored to preserve last valid value"
                }
            }

            div {
                class: "bg-gray-50 p-3 rounded-lg",
                p {
                    class: "text-sm text-gray-600",
                    "💡 This is like saving a variable. The data persists until you overwrite it or the workflow ends."
                }
            }
        }
    }
}

#[component]
pub fn SaveToMemoryNodeCard() -> Element {
    rsx! {
        div {
            class: "flex items-center gap-3 p-3 bg-white border border-gray-200 rounded-lg shadow-sm hover:shadow-md transition-shadow",

            div {
                class: "w-10 h-10 bg-indigo-100 rounded-full flex items-center justify-center",
                span {
                    class: "text-xl",
                    "💾"
                }
            },

            div {
                class: "flex-1",
                h3 {
                    class: "font-medium text-gray-900",
                    "Save to Memory"
                }
                p {
                    class: "text-sm text-gray-500",
                    "Store a value for later"
                }
            }
        }
    }
}
