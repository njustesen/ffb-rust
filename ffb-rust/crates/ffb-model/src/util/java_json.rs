//! The read/write semantics of Java's `com.fumbbl.ffb.json.*Option` classes and `UtilJson`
//! (minimal-json), for the 1:1 `init_from` / `to_json_value` ports of the model classes (BACKLOG §O).
//!
//! Where Java would THROW (a primitive assigned from a missing `Boolean`, `asBoolean()` on JSON
//! null, `asInt()` on a string ...) these return `Err`, so a Rust load never silently diverges.

use serde_json::{Map, Value};

pub type JsonObject = Map<String, Value>;

/// `UtilJson.toJsonObject(JsonValue)`: not an object -> IllegalArgumentException.
pub fn to_json_object(value: &Value) -> Result<&JsonObject, String> {
    value.as_object().ok_or_else(|| "JsonValue is not an object.".to_string())
}

/// `JsonAbstractOption.isDefinedIn`: the key is present (a JSON `null` value counts as defined).
pub fn is_defined_in(obj: &JsonObject, key: &str) -> bool {
    obj.contains_key(key)
}

/// minimal-json `JsonValue.asBoolean()`.
fn as_boolean(key: &str, value: &Value) -> Result<bool, String> {
    value.as_bool().ok_or_else(|| format!("{key}: asBoolean on {value}"))
}

/// minimal-json `JsonValue.asInt()`.
fn as_int(key: &str, value: &Value) -> Result<i32, String> {
    value
        .as_i64()
        .and_then(|v| i32::try_from(v).ok())
        .ok_or_else(|| format!("{key}: asInt on {value}"))
}

/// `JsonBooleanOption.getFrom` assigned to a primitive `boolean` field: a missing key is a
/// `null` Boolean and unboxing it throws.
pub fn get_boolean(obj: &JsonObject, key: &str) -> Result<bool, String> {
    match obj.get(key) {
        Some(v) => as_boolean(key, v),
        None => Err(format!("{key}: missing (null Boolean unboxed)")),
    }
}

/// `JsonBooleanOption.getFrom` -> `Boolean` (null when the key is absent).
pub fn get_boolean_object(obj: &JsonObject, key: &str) -> Result<Option<bool>, String> {
    obj.get(key).map(|v| as_boolean(key, v)).transpose()
}

/// `JsonIntOption.getFrom`: absent or JSON null -> 0.
pub fn get_int(obj: &JsonObject, key: &str) -> Result<i32, String> {
    match obj.get(key) {
        None | Some(Value::Null) => Ok(0),
        Some(v) => as_int(key, v),
    }
}

/// `JsonStringOption.getFrom`: absent or JSON null -> null.
pub fn get_string(obj: &JsonObject, key: &str) -> Result<Option<String>, String> {
    match obj.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.clone())),
        Some(v) => Err(format!("{key}: asString on {v}")),
    }
}

/// `UtilJson.toPlayerState` (via `JsonPlayerStateOption.getFrom`): absent or null -> null, else
/// `new PlayerState(asInt())`.
pub fn get_player_state_id(obj: &JsonObject, key: &str) -> Result<Option<i32>, String> {
    match obj.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => as_int(key, v).map(Some),
    }
}

/// `UtilJson.toEnumWithName(factory, value)`: absent or null -> null, else
/// `factory.forName(asString())` (which may itself be null for an unknown name).
pub fn get_enum_name<'a>(obj: &'a JsonObject, key: &str) -> Result<Option<&'a str>, String> {
    match obj.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.as_str())),
        Some(v) => Err(format!("{key}: asString on {v}")),
    }
}

/// `JsonArrayOption.getFrom`: absent or null -> null.
pub fn get_array<'a>(obj: &'a JsonObject, key: &str) -> Result<Option<&'a Vec<Value>>, String> {
    match obj.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Array(a)) => Ok(Some(a)),
        Some(v) => Err(format!("{key}: asArray on {v}")),
    }
}

/// `JsonValue.valueOf(String)` / `UtilJson.toJsonValue(INamedObject)`: null -> JSON null.
pub fn string_or_null(value: Option<&str>) -> Value {
    value.map_or(Value::Null, |s| Value::String(s.to_string()))
}

/// `UtilJson.toJsonValue(PlayerState)`: null -> JSON null, else the id.
pub fn player_state_or_null(id: Option<i32>) -> Value {
    id.map_or(Value::Null, Value::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn obj(v: Value) -> JsonObject {
        v.as_object().unwrap().clone()
    }

    #[test]
    fn option_semantics_match_java() {
        let o = obj(json!({ "b": true, "n": null, "i": 7, "s": "x" }));
        assert_eq!(get_boolean(&o, "b"), Ok(true));
        assert!(get_boolean(&o, "missing").is_err()); // null Boolean unboxed -> NPE
        assert!(get_boolean(&o, "n").is_err()); // asBoolean on JSON null throws
        assert_eq!(get_boolean_object(&o, "missing"), Ok(None));
        assert!(is_defined_in(&o, "n")); // JsonValue.NULL is a non-null reference
        assert_eq!(get_int(&o, "missing"), Ok(0));
        assert_eq!(get_int(&o, "n"), Ok(0));
        assert_eq!(get_int(&o, "i"), Ok(7));
        assert!(get_int(&o, "s").is_err());
        assert_eq!(get_string(&o, "n"), Ok(None));
        assert_eq!(get_string(&o, "s"), Ok(Some("x".to_string())));
        assert_eq!(get_player_state_id(&o, "i"), Ok(Some(7)));
    }
}
