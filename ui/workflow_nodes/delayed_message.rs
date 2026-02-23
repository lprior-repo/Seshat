use crate::ui::workflow_nodes::schema::DelayedMessageConfig;
use dioxus::prelude::*;

#[component]
pub fn DelayedMessageForm(config: Signal<DelayedMessageConfig>) -> Element {
    let pretty_input = if let Ok(value) = serde_json::to_string_pretty(&*config.read().input) {
        value
    } else {
        String::new()
    };

    rsx! {
        div {
            class: "space-y-4",

            div {
                class: "bg-orange-50 border border-orange-200 rounded-lg p-3 mb-4",
                p {
                    class: "text-sm text-orange-800",
                    "⏰ ",
                    strong { "Schedule for Later" },
                    " - Send a message at a specific time."
                }
            }

            div {
                class: "form-field",
                label {
                    class: "block text-sm font-medium text-gray-700 mb-1",
                    "Service Name"
                }
                input {
                    r#type: "text",
                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-orange-500",
                    placeholder: "e.g., reminder_service",
                    value: "{config.read().service_name}",
                    oninput: move |e| {
                        config.write().service_name = e.value().clone();
                    }
                }
            }

            div {
                class: "form-field",
                label {
                    class: "block text-sm font-medium text-gray-700 mb-1",
                    "Handler"
                }
                input {
                    r#type: "text",
                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-orange-500",
                    placeholder: "e.g., send_reminder",
                    value: "{config.read().handler_name}",
                    oninput: move |e| {
                        config.write().handler_name = e.value().clone();
                    }
                }
            }

            div {
                class: "form-field",
                label {
                    class: "block text-sm font-medium text-gray-700 mb-1",
                    "How long to wait?"
                }
                div {
                    class: "grid grid-cols-4 gap-2 mb-2",
                    button {
                        class: "px-2 py-2 text-sm border rounded-md hover:bg-gray-50",
                        onclick: move |_| {
                            config.write().delay_ms = 60_000;
                        },
                        "1 min"
                    }
                    button {
                        class: "px-2 py-2 text-sm border rounded-md hover:bg-gray-50",
                        onclick: move |_| {
                            config.write().delay_ms = 3600_000;
                        },
                        "1 hour"
                    }
                    button {
                        class: "px-2 py-2 text-sm border rounded-md hover:bg-gray-50",
                        onclick: move |_| {
                            config.write().delay_ms = 86400_000;
                        },
                        "1 day"
                    }
                    button {
                        class: "px-2 py-2 text-sm border rounded-md hover:bg-gray-50",
                        onclick: move |_| {
                            config.write().delay_ms = 604800_000;
                        },
                        "1 week"
                    }
                }
                input {
                    r#type: "number",
                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-orange-500",
                    placeholder: "Or enter milliseconds",
                    value: "{config.read().delay_ms}",
                    oninput: move |e| {
                        if let Ok(v) = e.value().parse::<u64>() {
                            config.write().delay_ms = v;
                        }
                    }
                }
                p {
                    class: "text-xs text-gray-500 mt-1",
                    "How long to wait before sending"
                }
            }

            div {
                class: "form-field",
                label {
                    class: "block text-sm font-medium text-gray-700 mb-1",
                    "Message (JSON)"
                }
                textarea {
                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-orange-500 font-mono text-sm",
                    rows: 3,
                    value: "{pretty_input}",
                    oninput: move |e| {
                        if let Ok(v) = serde_json::from_str(&e.value()) {
                            config.write().input = v;
                        }
                    },
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
pub fn DelayedMessageNodeCard() -> Element {
    rsx! {
        div {
            class: "flex items-center gap-3 p-3 bg-white border border-gray-200 rounded-lg shadow-sm hover:shadow-md transition-shadow",

            div {
                class: "w-10 h-10 bg-orange-100 rounded-full flex items-center justify-center",
                span {
                    class: "text-xl",
                    "⏰"
                }
            },

            div {
                class: "flex-1",
                h3 {
                    class: "font-medium text-gray-900",
                    "Delayed Message"
                }
                p {
                    class: "text-sm text-gray-500",
                    "Send after a delay"
                }
            }
        }
    }
}
