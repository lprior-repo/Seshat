#[cfg(not(target_arch = "wasm32"))]
use crate::export::png::export_png as export_png_file;
use crate::export::svg::generate_svg_string;
use crate::ui::toast::ToastIntent;
use crate::ui::toast::ToastQueue;
#[cfg(target_arch = "wasm32")]
use base64::{engine::general_purpose, Engine as _};
use diagram_models::canonical_json::to_canonical_pretty_json;
use diagram_models::document::DiagramDocument;
use dioxus::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;
#[cfg(not(target_arch = "wasm32"))]
use std::io::Write;

#[cfg(target_arch = "wasm32")]
fn wasm_download_bytes(filename: &str, mime: &str, bytes: &[u8]) {
    let b64 = general_purpose::STANDARD.encode(bytes);
    let script = format!(
        "(function() {{ const raw = atob('{b64}'); const array = new Uint8Array(raw.length); for (let i = 0; i < raw.length; i++) array[i] = raw.charCodeAt(i); const blob = new Blob([array], {{ type: '{mime}' }}); const url = URL.createObjectURL(blob); const a = document.createElement('a'); a.href = url; a.download = '{filename}'; a.click(); setTimeout(() => URL.revokeObjectURL(url), 0); dioxus.send({{ ok: true }}); }})();"
    );
    let mut eval = document::eval(&script);
    spawn(async move {
        let _ = eval.recv::<serde_json::Value>().await;
    });
}

#[cfg(target_arch = "wasm32")]
fn wasm_export_png_from_svg(svg: String, filename: &str, mut toasts: Signal<ToastQueue>) {
    let svg_b64 = general_purpose::STANDARD.encode(svg.as_bytes());
    let script = format!(
        "(function() {{ try {{ const svgText = atob('{svg_b64}'); const svgBlob = new Blob([svgText], {{ type: 'image/svg+xml' }}); const svgUrl = URL.createObjectURL(svgBlob); const img = new Image(); img.onload = () => {{ const canvas = document.createElement('canvas'); canvas.width = Math.max(1, img.naturalWidth || img.width || 1); canvas.height = Math.max(1, img.naturalHeight || img.height || 1); const ctx = canvas.getContext('2d'); if (!ctx) {{ URL.revokeObjectURL(svgUrl); dioxus.send({{ ok: false, reason: 'no-canvas-context' }}); return; }} ctx.drawImage(img, 0, 0); canvas.toBlob((blob) => {{ URL.revokeObjectURL(svgUrl); if (!blob) {{ dioxus.send({{ ok: false, reason: 'png-encode-failed' }}); return; }} const out = URL.createObjectURL(blob); const a = document.createElement('a'); a.href = out; a.download = '{filename}'; a.click(); setTimeout(() => URL.revokeObjectURL(out), 0); dioxus.send({{ ok: true }}); }}, 'image/png'); }}; img.onerror = () => {{ URL.revokeObjectURL(svgUrl); dioxus.send({{ ok: false, reason: 'svg-image-load-failed' }}); }}; img.src = svgUrl; }} catch (error) {{ dioxus.send({{ ok: false, reason: String(error) }}); }} }})();"
    );
    let mut eval = document::eval(&script);
    spawn(async move {
        if let Ok(msg) = eval.recv::<serde_json::Value>().await {
            if msg["ok"].as_bool() != Some(true) {
                let reason = msg["reason"].as_str().map_or("unknown", |v| v);
                toasts.with_mut(|queue| {
                    let _ = queue.add(
                        ToastIntent::Error,
                        "PNG export failed",
                        Some(format!("Browser export error: {reason}")),
                    );
                });
            }
        }
    });
}

pub fn export_png(doc_signal: Signal<DiagramDocument>, toasts: Signal<ToastQueue>) {
    #[cfg(target_arch = "wasm32")]
    {
        let doc = doc_signal.read().clone();
        let svg = generate_svg_string(&doc);
        wasm_export_png_from_svg(svg, "diagram.png", toasts);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut toasts = toasts;
        let doc = doc_signal.read();
        if let Err(e) = export_png_file(&doc, "diagram.png") {
            toasts.with_mut(|queue| {
                let _ = queue.add(
                    ToastIntent::Error,
                    "PNG export failed",
                    Some(format!("Failed to export PNG: {e}")),
                );
            });
        }
    }
}

pub fn export_svg(doc_signal: Signal<DiagramDocument>, toasts: Signal<ToastQueue>) {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = &toasts;
        let doc = doc_signal.read().clone();
        let svg = generate_svg_string(&doc);
        wasm_download_bytes("diagram.svg", "image/svg+xml", svg.as_bytes());
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut toasts = toasts;
        let doc = doc_signal.read();
        let svg = generate_svg_string(&doc);
        match File::create("diagram.svg") {
            Ok(mut file) => {
                if let Err(e) = file.write_all(svg.as_bytes()) {
                    toasts.with_mut(|queue| {
                        let _ = queue.add(
                            ToastIntent::Error,
                            "SVG export failed",
                            Some(format!("Failed to write SVG: {e}")),
                        );
                    });
                }
            }
            Err(e) => {
                toasts.with_mut(|queue| {
                    let _ = queue.add(
                        ToastIntent::Error,
                        "SVG export failed",
                        Some(format!("Failed to create SVG file: {e}")),
                    );
                });
            }
        }
    }
}

pub fn export_json(doc_signal: Signal<DiagramDocument>, mut toasts: Signal<ToastQueue>) {
    let doc = doc_signal.read().clone();
    if let Ok(json) = to_canonical_pretty_json(&doc) {
        let bytes = json.into_bytes();
        #[cfg(target_arch = "wasm32")]
        {
            wasm_download_bytes("diagram.json", "application/json", &bytes);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            match File::create("diagram.json") {
                Ok(mut file) => {
                    if let Err(e) = file.write_all(&bytes) {
                        toasts.with_mut(|queue| {
                            let _ = queue.add(
                                ToastIntent::Error,
                                "JSON export failed",
                                Some(format!("Failed to write JSON: {e}")),
                            );
                        });
                    }
                }
                Err(e) => {
                    toasts.with_mut(|queue| {
                        let _ = queue.add(
                            ToastIntent::Error,
                            "JSON export failed",
                            Some(format!("Failed to create JSON file: {e}")),
                        );
                    });
                }
            }
        }
    } else {
        toasts.with_mut(|queue| {
            let _ = queue.add(
                ToastIntent::Error,
                "JSON export failed",
                Some("Failed to serialize document".to_string()),
            );
        });
    }
}
