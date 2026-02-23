use crate::ui::workflow_nodes::schema::{RouterBranch, RouterConfig};
use dioxus::prelude::*;

#[component]
pub fn RouterForm(config: Signal<RouterConfig>) -> Element {
    let branches = config.read().branches.clone();
    let branch_count = branches.len();

    rsx! {
        div {
            class: "space-y-4",

            div {
                class: "bg-pink-50 border border-pink-200 rounded-lg p-3 mb-4",
                p {
                    class: "text-sm text-pink-800",
                    "🔀 ",
                    strong { "Split" },
                    " - Go to different steps based on a condition."
                }
            }

            for (idx, branch) in branches.iter().enumerate() {
                div {
                    key: "{idx}",
                    class: "border border-gray-200 rounded-lg p-3 mb-2",

                    div {
                        class: "flex items-center justify-between mb-2",
                        h4 {
                            class: "font-medium text-gray-700",
                            "Path {idx + 1}"
                        }
                        button {
                            class: "text-red-500 text-sm hover:underline",
                            onclick: move |_| {
                                let mut new_branches = config.read().branches.clone();
                                if new_branches.len() > 1 {
                                    new_branches.remove(idx);
                                    config.write().branches = new_branches;
                                }
                            },
                            "Remove"
                        }
                    }

                    div {
                        class: "form-field",
                        label {
                            class: "block text-sm font-medium text-gray-700 mb-1",
                            "Name this path"
                        }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-pink-500",
                            placeholder: "e.g., If approved, If rejected",
                            value: "{branch.name}",
                            oninput: move |e| {
                                config.write().branches[idx].name = e.value().clone();
                            }
                        }
                    }

                    div {
                        class: "form-field",
                        label {
                            class: "block text-sm font-medium text-gray-700 mb-1",
                            "When to take this path?"
                        }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-pink-500",
                            placeholder: "{{ steps.approve.status }} == 'approved'",
                            value: "{branch.condition}",
                            oninput: move |e| {
                                config.write().branches[idx].condition = e.value().clone();
                            }
                        }
                        p {
                            class: "text-xs text-gray-500 mt-1",
                            "Use {{ step.field }} to check values"
                        }
                    }
                }
            }

            button {
                class: "w-full py-2 border-2 border-dashed border-gray-300 rounded-lg text-gray-500 hover:border-gray-400 hover:text-gray-600",
                onclick: move |_| {
                    let mut branches = config.read().branches.clone();
                    branches.push(RouterBranch {
                        name: format!("Path {}", branches.len() + 1),
                        condition: String::new(),
                        next_node_id: None,
                    });
                    config.write().branches = branches;
                },
                "+ Add Another Path"
            }

            if branch_count == 0 {
                div {
                    class: "bg-gray-50 p-3 rounded-lg mt-2",
                    p {
                        class: "text-sm text-gray-600",
                        "💡 Use this to check conditions and go different directions, like an if/else."
                    }
                }
            }
        }
    }
}

#[component]
pub fn RouterNodeCard() -> Element {
    rsx! {
        div {
            class: "flex items-center gap-3 p-3 bg-white border border-gray-200 rounded-lg shadow-sm hover:shadow-md transition-shadow",

            div {
                class: "w-10 h-10 bg-pink-100 rounded-full flex items-center justify-center",
                span {
                    class: "text-xl",
                    "🔀"
                }
            },

            div {
                class: "flex-1",
                h3 {
                    class: "font-medium text-gray-900",
                    "Router"
                }
                p {
                    class: "text-sm text-gray-500",
                    "Go different ways based on conditions"
                }
            }
        }
    }
}
