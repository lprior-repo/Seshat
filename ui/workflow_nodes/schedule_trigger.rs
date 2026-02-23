use crate::ui::workflow_nodes::schema::ScheduleTriggerConfig;
use dioxus::prelude::*;

#[derive(Clone)]
pub struct ScheduleTriggerNode {
    pub config: Signal<ScheduleTriggerConfig>,
}

impl ScheduleTriggerNode {
    pub fn new() -> Self {
        Self {
            config: use_signal(|| ScheduleTriggerConfig {
                cron_expression: String::new(),
                timezone: "UTC".to_string(),
            }),
        }
    }
}

#[component]
pub fn ScheduleTriggerForm(config: Signal<ScheduleTriggerConfig>) -> Element {
    rsx! {
        div {
            class: "space-y-4",

            div {
                class: "bg-blue-50 border border-blue-200 rounded-lg p-3 mb-4",
                p {
                    class: "text-sm text-blue-800",
                    "🕐 ",
                    strong { "Starts here" },
                    " - This workflow runs on a schedule."
                }
            }

            div {
                class: "form-field",
                label {
                    class: "block text-sm font-medium text-gray-700 mb-1",
                    "When should this run?"
                }
                div {
                    class: "grid grid-cols-3 gap-2 mb-2",
                    button {
                        class: "px-3 py-2 text-sm border rounded-md hover:bg-gray-50",
                        onclick: move |_| {
                            config.write().cron_expression = "0 * * * *".to_string();
                        },
                        "Every hour"
                    }
                    button {
                        class: "px-3 py-2 text-sm border rounded-md hover:bg-gray-50",
                        onclick: move |_| {
                            config.write().cron_expression = "0 0 * * *".to_string();
                        },
                        "Daily"
                    }
                    button {
                        class: "px-3 py-2 text-sm border rounded-md hover:bg-gray-50",
                        onclick: move |_| {
                            config.write().cron_expression = "0 0 * * 0".to_string();
                        },
                        "Weekly"
                    }
                }
                input {
                    r#type: "text",
                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                    placeholder: "0 * * * *",
                    value: "{config.read().cron_expression}",
                    oninput: move |e| {
                        config.write().cron_expression = e.value().clone();
                    }
                }
                p {
                    class: "text-xs text-gray-500 mt-1",
                    "Cron expression: minute hour day month weekday"
                }
            }

            div {
                class: "form-field",
                label {
                    class: "block text-sm font-medium text-gray-700 mb-1",
                    "Timezone"
                }
                select {
                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                    value: "{config.read().timezone}",
                    onchange: move |e| {
                        config.write().timezone = e.value().clone();
                    },
                    option { value: "UTC", "UTC" }
                    option { value: "America/New_York", "Eastern Time" }
                    option { value: "America/Los_Angeles", "Pacific Time" }
                    option { value: "Europe/London", "London" }
                    option { value: "Europe/Paris", "Paris" }
                    option { value: "Asia/Tokyo", "Tokyo" }
                }
            }
        }
    }
}

#[component]
pub fn ScheduleTriggerNodeCard() -> Element {
    rsx! {
        div {
            class: "flex items-center gap-3 p-3 bg-white border border-gray-200 rounded-lg shadow-sm hover:shadow-md transition-shadow",

            div {
                class: "w-10 h-10 bg-purple-100 rounded-full flex items-center justify-center",
                span {
                    class: "text-xl",
                    "🕐"
                }
            }

            div {
                class: "flex-1",
                h3 {
                    class: "font-medium text-gray-900",
                    "Schedule"
                }
                p {
                    class: "text-sm text-gray-500",
                    "Runs automatically on a schedule"
                }
            }
        }
    }
}
