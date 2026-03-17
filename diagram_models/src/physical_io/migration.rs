use serde_json::Value;

use super::Error;

pub fn migrate_schema(raw_json: serde_json::Value) -> Result<serde_json::Value, Error> {
    let mut obj = match raw_json {
        Value::Object(map) => map,
        _ => return Err(Error::ParseError("Root is not an object".into())),
    };

    let version = obj
        .get("version")
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| Error::MissingField("version".into()))?;

    if version < 0.9 {
        return Err(Error::UnsupportedVersion(version.to_string()));
    } else if version == 0.9 || version == 1.0 {
        obj.insert("version".into(), Value::Number(serde_json::Number::from(2)));
        if !obj.contains_key("revision") {
            obj.insert(
                "revision".into(),
                Value::Number(serde_json::Number::from(0)),
            );
        }
    } else if version > 2.0 {
        return Err(Error::UnsupportedVersion(version.to_string()));
    }

    Ok(Value::Object(obj))
}
