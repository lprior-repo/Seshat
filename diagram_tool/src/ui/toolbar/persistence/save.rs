#![allow(dead_code)]
use crate::ui::editor::ToolMode;
use crate::ui::toast::{ToastApi, ToastIntent, ToastOptions, ToastQueue};
#[cfg(not(target_arch = "wasm32"))]
use diagram_models::canonical_json::to_canonical_pretty_json;
use diagram_models::document::{ArrowType, DiagramDocument, EdgeStyle};
use dioxus::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;

#[cfg(not(target_arch = "wasm32"))]
use super::common::{update_load_save_error, update_load_save_success};

pub fn save_workspace(
    doc_signal: Signal<DiagramDocument>,
    _tool_signal: Signal<ToolMode>,
    _edge_style_signal: Signal<EdgeStyle>,
    _arrow_type_signal: Signal<ArrowType>,
    toasts: Signal<ToastQueue>,
) {
    let toast_api = ToastApi::from_signal(toasts);
    let toast_handle = toast_api.toast(
        ToastOptions::new(ToastIntent::Info, "Saving workspace").with_detail("Preparing data..."),
    );
    #[cfg(target_arch = "wasm32")]
    {
        let _ = doc_signal;
        let _ = toast_handle.dismiss();
        let toast_api = ToastApi::from_signal(toasts);
        let _ = toast_api.toast(
            ToastOptions::new(ToastIntent::Error, "Save not available")
                .with_detail("Backend has been decommissioned"),
        );
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let doc_snapshot = doc_signal.read().clone();
        spawn(async move {
            let path = FileDialog::new()
                .add_filter("Seshat Diagram", &["json"])
                .set_file_name("diagram.json")
                .save_file();
            match path {
                None => {
                    let _ = toast_handle.dismiss();
                }
                Some(p) => match to_canonical_pretty_json(&doc_snapshot) {
                    Ok(json_str) => match fs::write(&p, json_str.as_bytes()) {
                        Ok(()) => {
                            update_load_save_success(
                                toast_handle,
                                "Workspace saved",
                                format!("Saved to {}", p.display()),
                            );
                        }
                        Err(e) => update_load_save_error(
                            toast_handle,
                            "Save failed",
                            format!("Save error: {e}"),
                        ),
                    },
                    Err(e) => update_load_save_error(
                        toast_handle,
                        "Save failed",
                        format!("Serialize error: {e}"),
                    ),
                },
            }
        });
    }
}
