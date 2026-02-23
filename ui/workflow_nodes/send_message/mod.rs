use crate::ui::workflow_nodes::schema::{SendMessageConfig, TargetType};
use dioxus::prelude::*;

#[component]
pub fn SendMessageForm(config: Signal<SendMessageConfig>) -> Element {
    let pretty_input = if let Ok(value) = serde_json::to_string_pretty(&*config.read().input) {
        value
    } else {
        String::new()
    };

    let key_value = match config.read().key.clone() {
        Some(value) => value,
        None => String::new(),
    };

    rsx! {
        div {
            class: "space-y-4",

            div {
                class: "bg-yellow-50 border border-yellow-200 rounded-lg p-3 mb-4",
                p {
                    class: "text-sm text-yellow-800",
                    "📤 ",
                    strong { "Fire & Forget" },
                    " - Send a message and continue without waiting."
                }
            }

            div {
                class: "form-field",
                label {
                    class: "block text-sm font-medium text-gray-700 mb-1",
                    "Send to"
                }
                select {
                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-yellow-500",
                    value: match &*config.read() {
                        SendMessageConfig { target_type: TargetType::Service, .. } => "Service",
                        SendMessageConfig { target_type: TargetType::VirtualObject, .. } => "Virtual Object",
                        SendMessageConfig { target_type: TargetType::Workflow, .. } => "Workflow",
                    },
                    onchange: move |e| {
                        config.write().target_type = match e.value().as_str() {
                            "Service" => TargetType::Service,
                            "Virtual Object" => TargetType::VirtualObject,
                            "Workflow" => TargetType::Workflow,
                            _ => TargetType::Service,
                        };
                    },
                    option { value: "Service", "Service" }
                    option { value: "Virtual Object", "Virtual Object" }
                    option { value: "Workflow", "Workflow" }
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
                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-yellow-500",
                    placeholder: "e.g., notification_service",
                    value: "{config.read().service_name}",
                    oninput: move |e| {
                        config.write().service_name = e.value().clone();
                    }
                }
            }

            div {
                class: "form-field",
                visible: matches!(config.read().target_type, TargetType::VirtualObject | TargetType::Workflow),
                label {
                    class: "block text-sm font-medium text-gray-700 mb-1",
                    "Key (for objects/workflows)"
                }
                input {
                    r#type: "text",
                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-yellow-500",
                    placeholder: "e.g., user-456",
                    value: "{key_value}",
                    oninput: move |e| {
                        config.write().key = Some(e.value().clone());
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
                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-yellow-500",
                    placeholder: "e.g., send_email",
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
                    "Message (JSON)"
                }
                textarea {
                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-yellow-500 font-mono text-sm",
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

            div {
                class: "bg-gray-50 p-3 rounded-lg",
                p {
                    class: "text-sm text-gray-600",
                    "💡 Use this when you don't need to wait for a response, like sending notifications."
                }
            }
        }
    }
}

#[component]
pub fn SendMessageNodeCard() -> Element {
    rsx! {
        div {
            class: "flex items-center gap-3 p-3 bg-white border border-gray-200 rounded-lg shadow-sm hover:shadow-md transition-shadow",

            div {
                class: "w-10 h-10 bg-yellow-100 rounded-full flex items-center justify-center",
                span {
                    class: "text-xl",
                    "📤"
                }
            }

            div {
                class: "flex-1",
                h3 {
                    class: "font-medium text-gray-900",
                    "Send Message"
                }
                p {
                    class: "text-sm text-gray-500",
                    "Send without waiting for response"
                }
            }
        }
    }
}
