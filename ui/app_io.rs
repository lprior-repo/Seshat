#[cfg(target_arch = "wasm32")]
use crate::graph::Workflow;

#[cfg(target_arch = "wasm32")]
pub fn canvas_rect_size() -> Option<(f32, f32)> {
    use web_sys::window;

    let document = window().and_then(|win| win.document())?;
    let element = document.query_selector("main").ok().flatten()?;
    let rect = element.get_bounding_client_rect();
    #[allow(clippy::cast_possible_truncation)]
    let width = rect.width() as f32;
    #[allow(clippy::cast_possible_truncation)]
    let height = rect.height() as f32;
    Some((width, height))
}

#[cfg(target_arch = "wasm32")]
pub fn canvas_origin() -> Option<(f32, f32)> {
    use web_sys::window;

    let document = window().and_then(|win| win.document())?;
    let element = document.query_selector("main").ok().flatten()?;
    let rect = element.get_bounding_client_rect();
    #[allow(clippy::cast_possible_truncation)]
    let left = rect.left() as f32;
    #[allow(clippy::cast_possible_truncation)]
    let top = rect.top() as f32;
    Some((left, top))
}

#[cfg(not(target_arch = "wasm32"))]
#[must_use]
pub const fn canvas_rect_size() -> Option<(f32, f32)> {
    None
}

#[cfg(not(target_arch = "wasm32"))]
#[must_use]
pub const fn canvas_origin() -> Option<(f32, f32)> {
    None
}

#[cfg(target_arch = "wasm32")]
pub fn download_workflow_json(name: &str, workflow: &Workflow) {
    use js_sys::Array;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{window, Blob, HtmlAnchorElement, Url};

    let json = match serde_json::to_string_pretty(workflow) {
        Ok(value) => value,
        Err(_) => return,
    };

    let chunks = Array::new();
    chunks.push(&JsValue::from_str(&json));

    let blob = match Blob::new_with_str_sequence(&chunks) {
        Ok(value) => value,
        Err(_) => return,
    };

    let url = match Url::create_object_url_with_blob(&blob) {
        Ok(value) => value,
        Err(_) => return,
    };

    let document = match window().and_then(|win| win.document()) {
        Some(value) => value,
        None => return,
    };

    let element = match document.create_element("a") {
        Ok(value) => value,
        Err(_) => {
            let _ = Url::revoke_object_url(&url);
            return;
        }
    };

    let anchor = match element.dyn_into::<HtmlAnchorElement>() {
        Ok(value) => value,
        Err(_) => {
            let _ = Url::revoke_object_url(&url);
            return;
        }
    };

    let filename = format!(
        "{}.json",
        name.trim()
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                    ch
                } else {
                    '_'
                }
            })
            .collect::<String>()
    );

    anchor.set_href(&url);
    anchor.set_download(&filename);
    anchor.click();
    let _ = Url::revoke_object_url(&url);
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::{canvas_origin, canvas_rect_size};

    #[test]
    fn given_non_wasm_target_when_reading_canvas_rect_size_then_none_is_returned() {
        assert_eq!(canvas_rect_size(), None);
    }

    #[test]
    fn given_non_wasm_target_when_reading_canvas_origin_then_none_is_returned() {
        assert_eq!(canvas_origin(), None);
    }
}
