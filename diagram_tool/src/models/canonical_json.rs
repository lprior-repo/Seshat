#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use serde::Serialize;

/// Serialize a value to canonical pretty-printed JSON.
///
/// # Errors
/// Returns an error if serialization fails.
pub fn to_canonical_pretty_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    let mut json = serde_json::to_value(value)?;
    canonicalize_value(&mut json);
    serde_json::to_string_pretty(&json)
}

fn canonicalize_value(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            let mut keys = map.keys().cloned().collect::<Vec<_>>();
            keys.sort();

            let mut next = serde_json::Map::new();
            for key in keys {
                if let Some(mut child) = map.remove(&key) {
                    canonicalize_value(&mut child);
                    let _ = next.insert(key, child);
                }
            }
            *map = next;
        }
        serde_json::Value::Array(items) => {
            for item in items {
                canonicalize_value(item);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::to_canonical_pretty_json;
    use serde_json::json;

    #[test]
    fn given_equivalent_objects_when_serialized_then_output_is_deterministic() {
        let a = json!({"z": 1, "a": {"d": 4, "b": 2}});
        let b = json!({"a": {"b": 2, "d": 4}, "z": 1});

        let first = to_canonical_pretty_json(&a).unwrap();
        let second = to_canonical_pretty_json(&b).unwrap();

        assert_eq!(first, second);
    }
}
