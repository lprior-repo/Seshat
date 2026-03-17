use dioxus::prelude::*;

use diagram_models::document::DiagramDocument;
use crate::ui::toolbar::auto_save;

#[cfg(target_arch = "wasm32")]
fn use_load_auto_save(
    doc_signal: Signal<DiagramDocument>,
    tool_signal: Signal<ToolMode>,
    edge_style_signal: Signal<EdgeStyle>,
    arrow_type_signal: Signal<ArrowType>,
    last_saved_revision: Signal<diagram_models::document::Revision>,
) {
    use crate::ui::toolbar::auto_save::AUTO_SAVE_KEY;

    use_effect(move || {
        let mut eval = document::eval(&format!(
            r#"
            (() => {{
                const key = "{AUTO_SAVE_KEY}";
                let data = null;
                try {{
                    data = localStorage.getItem(key);
                }} catch (_) {{}}
                dioxus.send({{ data }});
            }})();
            "#
        ));

        let mut doc_signal = doc_signal;
        let mut tool_signal = tool_signal;
        let mut edge_style_signal = edge_style_signal;
        let mut arrow_type_signal = arrow_type_signal;
        let mut last_saved_revision = last_saved_revision;

        spawn(async move {
            if let Ok(msg) = eval.recv::<serde_json::Value>().await {
                if let Some(data) = msg["data"]
                    .as_str()
                    .and_then(|s| if s.is_empty() { None } else { Some(s) })
                {
                    if let Ok(saved) = auto_save::deserialize_diagram(data) {
                        let mut doc = doc_signal.write();
                        *doc = saved.document;
                        last_saved_revision.set(doc.revision);

                        if let Some(mode) = ToolMode::from_persisted_key(&saved.tool_mode) {
                            *tool_signal.write() = mode;
                        }

                        *edge_style_signal.write() = saved.edge_style;
                        *arrow_type_signal.write() = saved.arrow_type;
                    }
                }
            }
        });
    });
}

#[cfg(target_arch = "wasm32")]
fn use_persist_auto_save(
    doc_signal: Signal<DiagramDocument>,
    tool_signal: Signal<ToolMode>,
    edge_style_signal: Signal<EdgeStyle>,
    arrow_type_signal: Signal<ArrowType>,
    last_saved_revision: Signal<diagram_models::document::Revision>,
) {
    use crate::ui::toolbar::auto_save::AUTO_SAVE_KEY;

    let mut last_saved_revision = last_saved_revision;

    use_effect(move || {
        let doc = doc_signal.read();
        let current_revision = doc.revision;

        if auto_save::has_revision_changed(current_revision, Some(*last_saved_revision.read())) {
            let saved = auto_save::AutoSavedDiagram::new(
                &doc,
                &tool_signal.read(),
                *edge_style_signal.read(),
                *arrow_type_signal.read(),
            );

            if let Ok(json) = auto_save::serialize_diagram(&saved) {
                if let Ok(payload_literal) = serde_json::to_string(&json) {
                    let _eval = document::eval(&format!(
                        r#"
                        (() => {{
                            try {{
                                localStorage.setItem("{AUTO_SAVE_KEY}", {payload_literal});
                            }} catch (_) {{}}
                        }})();
                        "#
                    ));
                }
            }

            last_saved_revision.set(current_revision);
        }
    });
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn use_auto_save(doc_signal: Signal<DiagramDocument>) {
    let tool_signal = use_context::<Signal<ToolMode>>();
    let edge_style_signal = use_context::<Signal<EdgeStyle>>();
    let arrow_type_signal = use_context::<Signal<ArrowType>>();
    let last_saved_revision = use_signal(auto_save::default_revision);

    use_load_auto_save(
        doc_signal,
        tool_signal,
        edge_style_signal,
        arrow_type_signal,
        last_saved_revision,
    );
    use_persist_auto_save(
        doc_signal,
        tool_signal,
        edge_style_signal,
        arrow_type_signal,
        last_saved_revision,
    );
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn use_auto_save(_doc_signal: Signal<DiagramDocument>) {
    let _ = auto_save::default_revision();
}
