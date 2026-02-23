use crate::ui::workflow_nodes::schema::{CodeLanguage, RunCodeConfig};
use dioxus::prelude::*;

#[component]
pub fn RunCodeForm(config: Signal<RunCodeConfig>) -> Element {
    rsx! {
        div {
            class: "space-y-4",

            div {
                class: "bg-red-50 border border-red-200 rounded-lg p-3 mb-4",
                p {
                    class: "text-sm text-red-800",
                    "⚡ ",
                    strong { "Run Code" },
                    " - Execute custom code. Results are saved for retries."
                }
            }

            div {
                class: "form-field",
                label {
                    class: "block text-sm font-medium text-gray-700 mb-1",
                    "Language"
                }
                select {
                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-red-500",
                    value: match &*config.read() {
                        RunCodeConfig { language: CodeLanguage::JavaScript, .. } => "JavaScript",
                        RunCodeConfig { language: CodeLanguage::Python, .. } => "Python",
                        RunCodeConfig { language: CodeLanguage::Expression, .. } => "Expression",
                    },
                    onchange: move |e| {
                        config.write().language = match e.value().as_str() {
                            "JavaScript" => CodeLanguage::JavaScript,
                            "Python" => CodeLanguage::Python,
                            "Expression" => CodeLanguage::Expression,
                            _ => CodeLanguage::Expression,
                        };
                    },
                    option { value: "Expression", "Expression (simple math/transform)" }
                    option { value: "JavaScript", "JavaScript" }
                    option { value: "Python", "Python" }
                }
            }

            match &*config.read() {
                RunCodeConfig { language: CodeLanguage::Expression, .. } => {
                    rsx! {
                        div {
                            class: "form-field",
                            label {
                                class: "block text-sm font-medium text-gray-700 mb-1",
                                "Expression"
                            }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-red-500 font-mono",
                                placeholder: "{{ steps.total.amount }} * 1.2",
                                value: "{config.read().code}",
                                oninput: move |e| {
                                    config.write().code = e.value().clone();
                                }
                            }
                            p {
                                class: "text-xs text-gray-500 mt-1",
                                "Use double braces like {{ and }} to render literals"
                            }
                        }
                    }
                }
                _ => {
                    rsx! {
                        div {
                            class: "form-field",
                            label {
                                class: "block text-sm font-medium text-gray-700 mb-1",
                                "Code"
                            }
                            textarea {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-red-500 font-mono text-sm",
                                rows: 8,
                                placeholder: "// Available: input (from previous step)\n// Return value is saved\n\nconst result = input.amount * 1.2;\nreturn { total: result };",
                                value: "{config.read().code}",
                                oninput: move |e| {
                                    config.write().code = e.value().clone();
                                }
                            }
                            p {
                                class: "text-xs text-gray-500 mt-1",
                                "Use double braces like {{ and }} to render literals"
                            }
                        }
                    }
                }
            }

            div {
                class: "bg-yellow-50 p-3 rounded-lg",
                p {
                    class: "text-sm text-yellow-800",
                    "⚠️ Code runs once. On retry, the saved result is used to ensure consistency."
                }
            }
        }
    }
}

#[component]
pub fn RunCodeNodeCard() -> Element {
    rsx! {
        div {
            class: "flex items-center gap-3 p-3 bg-white border border-gray-200 rounded-lg shadow-sm hover:shadow-md transition-shadow",

            div {
                class: "w-10 h-10 bg-red-100 rounded-full flex items-center justify-center",
                span {
                    class: "text-xl",
                    "⚡"
                }
            },

            div {
                class: "flex-1",
                h3 {
                    class: "font-medium text-gray-900",
                    "Run Code"
                }
                p {
                    class: "text-sm text-gray-500",
                    "Execute custom logic"
                }
            }
        }
    }
}
