#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        div { class: "min-h-screen bg-slate-100 flex items-center justify-center",
            h1 { class: "text-2xl font-bold text-slate-700", "Oya Frontend" }
            p { class: "text-slate-500", "Workflow editor loading..." }
        }
    }
}
