use diagram_models::document::DiagramDocument;

#[derive(Debug, PartialEq, Clone)]
pub enum ExportError {
    SerializationFailed(String),
    IoError(String),
    BrowserInteropFailed(String),
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportError::SerializationFailed(e) => write!(f, "Serialization failed: {}", e),
            ExportError::IoError(e) => write!(f, "IO error: {}", e),
            ExportError::BrowserInteropFailed(e) => write!(f, "Browser interop failed: {}", e),
        }
    }
}

impl std::error::Error for ExportError {}

#[cfg(not(target_arch = "wasm32"))]
pub async fn export_png(doc: DiagramDocument) -> Result<(), ExportError> {
    crate::export::png::export_png(&doc, "diagram.png")
        .map_err(|e| ExportError::IoError(e.to_string()))
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn export_svg(doc: DiagramDocument) -> Result<(), ExportError> {
    let svg = crate::export::svg::generate_svg_string(&doc);
    std::fs::write("diagram.svg", svg).map_err(|e| ExportError::IoError(e.to_string()))
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn export_json(doc: DiagramDocument) -> Result<(), ExportError> {
    let json = diagram_models::canonical_json::to_canonical_pretty_json(&doc)
        .map_err(|e| ExportError::SerializationFailed(e.to_string()))?;
    std::fs::write("diagram.json", json).map_err(|e| ExportError::IoError(e.to_string()))
}

#[cfg(target_arch = "wasm32")]
async fn wasm_download_bytes(filename: &str, mime: &str, bytes: &[u8]) -> Result<(), ExportError> {
    use base64::{engine::general_purpose, Engine as _};
    let b64 = general_purpose::STANDARD.encode(bytes);
    let script = format!(
        "(function() {{ try {{ const raw = atob('{b64}'); const array = new Uint8Array(raw.length); for (let i = 0; i < raw.length; i++) array[i] = raw.charCodeAt(i); const blob = new Blob([array], {{ type: '{mime}' }}); const url = URL.createObjectURL(blob); const a = document.createElement('a'); a.href = url; a.download = '{filename}'; a.click(); setTimeout(() => URL.revokeObjectURL(url), 0); dioxus::eval::eval_send({{ ok: true }}); }} catch (error) {{ dioxus::eval::eval_send({{ ok: false, reason: String(error) }}); }} }})();"
    );
    let mut eval = dioxus::prelude::document::eval(&script);
    match eval.recv::<serde_json::Value>().await {
        Ok(msg) if msg["ok"].as_bool() == Some(true) => Ok(()),
        Ok(msg) => Err(ExportError::BrowserInteropFailed(
            msg["reason"].as_str().unwrap_or("unknown interop error").to_string()
        )),
        Err(e) => Err(ExportError::BrowserInteropFailed(e.to_string())),
    }
}

#[cfg(target_arch = "wasm32")]
pub async fn export_png(doc: DiagramDocument) -> Result<(), ExportError> {
    let svg = crate::export::svg::generate_svg_string(&doc);
    use base64::{engine::general_purpose, Engine as _};
    let svg_b64 = general_purpose::STANDARD.encode(svg.as_bytes());
    let script = format!(
        "(function() {{ try {{ const svgText = atob('{svg_b64}'); const svgBlob = new Blob([svgText], {{ type: 'image/svg+xml' }}); const svgUrl = URL.createObjectURL(svgBlob); const img = new Image(); img.onload = () => {{ const canvas = document.createElement('canvas'); canvas.width = Math.max(1, img.naturalWidth || img.width || 1); canvas.height = Math.max(1, img.naturalHeight || img.height || 1); const ctx = canvas.getContext('2d'); if (!ctx) {{ URL.revokeObjectURL(svgUrl); dioxus::eval::eval_send({{ ok: false, reason: 'no-canvas-context' }}); return; }} ctx.drawImage(img, 0, 0); canvas.toBlob((blob) => {{ URL.revokeObjectURL(svgUrl); if (!blob) {{ dioxus::eval::eval_send({{ ok: false, reason: 'png-encode-failed' }}); return; }} const out = URL.createObjectURL(blob); const a = document.createElement('a'); a.href = out; a.download = 'diagram.png'; a.click(); setTimeout(() => URL.revokeObjectURL(out), 0); dioxus::eval::eval_send({{ ok: true }}); }}, 'image/png'); }}; img.onerror = () => {{ URL.revokeObjectURL(svgUrl); dioxus::eval::eval_send({{ ok: false, reason: 'svg-image-load-failed' }}); }}; img.src = svgUrl; }} catch (error) {{ dioxus::eval::eval_send({{ ok: false, reason: String(error) }}); }} }})();"
    );
    let mut eval = dioxus::prelude::document::eval(&script);
    match eval.recv::<serde_json::Value>().await {
        Ok(msg) if msg["ok"].as_bool() == Some(true) => Ok(()),
        Ok(msg) => Err(ExportError::BrowserInteropFailed(
            msg["reason"].as_str().unwrap_or("unknown interop error").to_string()
        )),
        Err(e) => Err(ExportError::BrowserInteropFailed(e.to_string())),
    }
}

#[cfg(target_arch = "wasm32")]
pub async fn export_svg(doc: DiagramDocument) -> Result<(), ExportError> {
    let svg = crate::export::svg::generate_svg_string(&doc);
    wasm_download_bytes("diagram.svg", "image/svg+xml", svg.as_bytes()).await
}

#[cfg(target_arch = "wasm32")]
pub async fn export_json(doc: DiagramDocument) -> Result<(), ExportError> {
    let json = diagram_models::canonical_json::to_canonical_pretty_json(&doc)
        .map_err(|e| ExportError::SerializationFailed(e.to_string()))?;
    wasm_download_bytes("diagram.json", "application/json", json.as_bytes()).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use diagram_models::document::DiagramDocument;

    #[tokio::test]
    #[cfg(not(target_arch = "wasm32"))]
    async fn test_native_export_json_writes_verified_content_when_valid_document() -> Result<(), Box<dyn std::error::Error>> {
        let doc = DiagramDocument::default();
        export_json(doc).await?;
        let content = std::fs::read_to_string("diagram.json")?;
        assert!(content.contains("\"nodes\":"));
        std::fs::remove_file("diagram.json")?;
        Ok(())
    }

    #[tokio::test]
    #[cfg(not(target_arch = "wasm32"))]
    async fn test_native_export_svg_writes_verified_content() -> Result<(), Box<dyn std::error::Error>> {
        let doc = DiagramDocument::default();
        export_svg(doc).await?;
        let content = std::fs::read_to_string("diagram.svg")?;
        assert!(content.contains("<svg"));
        std::fs::remove_file("diagram.svg")?;
        Ok(())
    }

    #[tokio::test]
    #[cfg(not(target_arch = "wasm32"))]
    async fn test_native_export_png_writes_verified_content() -> Result<(), Box<dyn std::error::Error>> {
        let doc = DiagramDocument::default();
        export_png(doc).await?;
        let content = std::fs::read("diagram.png")?;
        assert_eq!(&content[0..4], &[0x89, b'P', b'N', b'G']); // PNG magic number
        std::fs::remove_file("diagram.png")?;
        Ok(())
    }
}
