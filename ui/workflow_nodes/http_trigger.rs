use crate::ui::workflow_nodes::schema::{HttpMethod, HttpTriggerConfig};
use dioxus::prelude::*;

#[derive(Clone)]
pub struct HttpTriggerNode {
    pub config: Signal<HttpTriggerConfig>,
}

impl HttpTriggerNode {
    pub fn new() -> Self {
        Self {
            config: use_signal(|| HttpTriggerConfig {
                handler_name: String::new(),
                method: HttpMethod::POST,
            }),
        }
    }

    pub fn from_config(config: HttpTriggerConfig) -> Self {
        Self {
            config: use_signal(|| config),
        }
    }
}

impl Default for HttpTriggerNode {
    fn default() -> Self {
        Self::new()
    }
}

#[component]
pub fn HttpTriggerForm(config: Signal<HttpTriggerConfig>) -> Element {
    let method_options = ["GET", "POST", "PUT", "DELETE", "PATCH"];

    rsx! {
        div {
            class: "space-y-4",

            div {
                class: "bg-blue-50 border border-blue-200 rounded-lg p-3 mb-4",
                p {
                    class: "text-sm text-blue-800",
                    "🚀 ",
                    strong { "Starts here" },
                    " - This workflow runs when an HTTP request comes in."
                }
            }

            div {
                class: "form-field",
                label {
                    class: "block text-sm font-medium text-gray-700 mb-1",
                    "Handler Name"
                }
                input {
                    r#type: "text",
                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                    placeholder: "e.g., process_order",
                    value: "{config.read().handler_name}",
                    oninput: move |e| {
                        config.write().handler_name = e.value().clone();
                    }
                }
                p {
                    class: "text-xs text-gray-500 mt-1",
                    "What your users call to start this workflow"
                }
            }

            div {
                class: "form-field",
                label {
                    class: "block text-sm font-medium text-gray-700 mb-1",
                    "HTTP Method"
                }
                select {
                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                    value: match &*config.read() {
                        HttpTriggerConfig { method: HttpMethod::GET, .. } => "GET",
                        HttpTriggerConfig { method: HttpMethod::POST, .. } => "POST",
                        HttpTriggerConfig { method: HttpMethod::PUT, .. } => "PUT",
                        HttpTriggerConfig { method: HttpMethod::DELETE, .. } => "DELETE",
                        HttpTriggerConfig { method: HttpMethod::PATCH, .. } => "PATCH",
                    },
                    onchange: move |e| {
                        config.write().method = match e.value().as_str() {
                            "GET" => HttpMethod::GET,
                            "POST" => HttpMethod::POST,
                            "PUT" => HttpMethod::PUT,
                            "DELETE" => HttpMethod::DELETE,
                            "PATCH" => HttpMethod::PATCH,
                            _ => HttpMethod::POST,
                        };
                    },
                    option { value: "GET", "GET" }
                    option { value: "POST", "POST" }
                    option { value: "PUT", "PUT" }
                    option { value: "DELETE", "DELETE" }
                    option { value: "PATCH", "PATCH" }
                }
                p {
                    class: "text-xs text-gray-500 mt-1",
                    "What kind of request this handler accepts"
                }
            }
        }
    }
}

#[component]
pub fn HttpTriggerNodeCard() -> Element {
    rsx! {
        div {
            class: "flex items-center gap-3 p-3 bg-white border border-gray-200 rounded-lg shadow-sm hover:shadow-md transition-shadow",

            div {
                class: "w-10 h-10 bg-blue-100 rounded-full flex items-center justify-center",
                span {
                    class: "text-xl",
                    "🌐"
                }
            }

            div {
                class: "flex-1",
                h3 {
                    class: "font-medium text-gray-900",
                    "HTTP Trigger"
                }
                p {
                    class: "text-sm text-gray-500",
                    "Starts when someone calls this URL"
                }
            }
        }
    }
}
