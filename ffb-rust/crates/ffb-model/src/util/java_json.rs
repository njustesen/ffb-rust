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

/// `JsonStringArrayOption.getFrom`: absent or JSON null -> null; a JSON null element -> a null
/// String; any other non-string element throws (`asString`).
pub fn get_string_array(obj: &JsonObject, key: &str) -> Result<Option<Vec<Option<String>>>, String> {
    match obj.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Array(a)) => a
            .iter()
            .map(|v| match v {
                Value::Null => Ok(None),
                Value::String(s) => Ok(Some(s.clone())),
                other => Err(format!("{key}: asString on {other}")),
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Some),
        Some(v) => Err(format!("{key}: asArray on {v}")),
    }
}

/// Java `String.equalsIgnoreCase` (`regionMatches(true, ..)`): per character, equal, or equal
/// after `Character.toUpperCase`, or equal after `toLowerCase(toUpperCase(..))`. Java's
/// `Character` mappings are the SIMPLE (one char) case mappings; a char whose Rust full mapping
/// is not a single char keeps itself, as its simple mapping does.
pub fn java_equals_ignore_case(a: &str, b: &str) -> bool {
    fn upper(c: char) -> char {
        let mut it = c.to_uppercase();
        match (it.next(), it.next()) {
            (Some(u), None) => u,
            _ => c,
        }
    }
    fn lower(c: char) -> char {
        let mut it = c.to_lowercase();
        match (it.next(), it.next()) {
            (Some(l), None) => l,
            _ => c,
        }
    }
    if a.encode_utf16().count() != b.encode_utf16().count() {
        return false;
    }
    a.chars().zip(b.chars()).all(|(x, y)| {
        if x == y {
            return true;
        }
        let (ux, uy) = (upper(x), upper(y));
        ux == uy || lower(ux) == lower(uy)
    })
}

/// `JsonFieldCoordinateOption.getFrom` = `UtilJson.toFieldCoordinate`: absent or null -> null; an
/// array of exactly two ints -> the coordinate; anything else throws.
pub fn get_field_coordinate(obj: &JsonObject, key: &str) -> Result<Option<crate::types::FieldCoordinate>, String> {
    match obj.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Array(a)) if a.len() == 2 => {
            Ok(Some(crate::types::FieldCoordinate::new(as_int(key, &a[0])?, as_int(key, &a[1])?)))
        }
        Some(v) => Err(format!("{key}: JsonValue is not a valid FieldCoordinate object: {v}")),
    }
}

/// `UtilJson.toJsonValue(FieldCoordinate)`: null -> JSON null, else `[x, y]`.
pub fn field_coordinate_or_null(coordinate: Option<crate::types::FieldCoordinate>) -> Value {
    coordinate.map_or(Value::Null, |c| Value::Array(vec![Value::from(c.x), Value::from(c.y)]))
}

/// `JsonObjectOption.getFrom`: absent or null -> null; not an object -> `asObject` throws.
pub fn get_object<'a>(obj: &'a JsonObject, key: &str) -> Result<Option<&'a JsonObject>, String> {
    match obj.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Object(o)) => Ok(Some(o)),
        Some(v) => Err(format!("{key}: asObject on {v}")),
    }
}

/// `UtilJson.toPlayerState`, as a Rust `PlayerState` (whose id is unsigned: a negative id `Err`s).
pub fn get_player_state(obj: &JsonObject, key: &str) -> Result<Option<crate::enums::PlayerState>, String> {
    get_player_state_id(obj, key)?
        .map(|id| u32::try_from(id).map(crate::enums::PlayerState).map_err(|_| format!("{key}: player state {id}")))
        .transpose()
}

/// `JsonValue.valueOf(String)` / `UtilJson.toJsonValue(INamedObject)`: null -> JSON null.
pub fn string_or_null(value: Option<&str>) -> Value {
    value.map_or(Value::Null, |s| Value::String(s.to_string()))
}

/// `UtilJson.toJsonValue(PlayerState)`: null -> JSON null, else the id.
pub fn player_state_or_null(id: Option<i32>) -> Value {
    id.map_or(Value::Null, Value::from)
}

/// `JsonArrayOption.getFrom`: absent or JSON null -> a NEW EMPTY array (not null!); a non-array
/// throws (`asArray`). Contrast `get_array`, which keeps absent/null as `None`.
pub fn get_json_array_option(obj: &JsonObject, key: &str) -> Result<Vec<Value>, String> {
    Ok(get_array(obj, key)?.cloned().unwrap_or_default())
}

/// `JsonIntArrayOption.getFrom`: absent or JSON null -> null; every element `asInt()` (throws on
/// a non-int, incl. JSON null).
pub fn get_int_array(obj: &JsonObject, key: &str) -> Result<Option<Vec<i32>>, String> {
    match get_array(obj, key)? {
        None => Ok(None),
        Some(a) => a.iter().map(|v| as_int(key, v)).collect::<Result<Vec<_>, _>>().map(Some),
    }
}

/// `JsonIntArrayOption.addTo` value: null -> JSON null, else `[ints]`.
pub fn int_array_or_null(values: Option<&[i32]>) -> Value {
    values.map_or(Value::Null, |a| Value::Array(a.iter().map(|&i| Value::from(i)).collect()))
}

/// `JsonBooleanArrayOption.getFrom`: `getValueFrom(..).asArray()` with no null check -- absent
/// (NPE) or JSON null (`asArray` throws) is an error; every element `asBoolean()`.
pub fn get_boolean_array(obj: &JsonObject, key: &str) -> Result<Vec<bool>, String> {
    match obj.get(key) {
        Some(Value::Array(a)) => a.iter().map(|v| as_boolean(key, v)).collect(),
        Some(v) => Err(format!("{key}: asArray on {v}")),
        None => Err(format!("{key}: NullPointerException (missing boolean array)")),
    }
}

/// `JsonBooleanArrayOption.addTo` value: null -> JSON null, else `[booleans]`.
pub fn boolean_array_or_null(values: Option<&[bool]>) -> Value {
    values.map_or(Value::Null, |a| Value::Array(a.iter().map(|&b| Value::Bool(b)).collect()))
}

/// `JsonStringArrayOption.addTo` value: null -> JSON null, else the strings (a null element ->
/// JSON null).
pub fn string_array_or_null(values: Option<&[Option<String>]>) -> Value {
    values.map_or(Value::Null, |a| Value::Array(a.iter().map(|s| string_or_null(s.as_deref())).collect()))
}

/// `JsonStringArrayOption.addTo` of a non-null collection of non-null strings.
pub fn string_array(values: &[String]) -> Value {
    Value::Array(values.iter().map(|s| Value::String(s.clone())).collect())
}

/// `StringTool.isProvided(Object)`: non-null and `toString().length() > 0`.
pub fn is_provided(value: Option<&str>) -> bool {
    value.is_some_and(|s| !s.is_empty())
}

/// `JsonIntegerMapOption.getFrom`: a NEW map (Java `HashMap`); filled only when the value is a
/// JSON object, each member `asInt()` (throws on a non-int). Absent, null or a non-object value ->
/// empty. Returned in input member order; the Java map is a HashMap (its `addTo` order is the
/// HashMap's, irrelevant under canonical key sorting). A duplicate member name: later wins.
pub fn get_integer_map(obj: &JsonObject, key: &str) -> Result<Vec<(String, i32)>, String> {
    let mut out: Vec<(String, i32)> = Vec::new();
    if let Some(Value::Object(o)) = obj.get(key) {
        for (name, v) in o {
            let value = as_int(key, v)?;
            match out.iter_mut().find(|(n, _)| n == name) {
                Some(e) => e.1 = value,
                None => out.push((name.clone(), value)),
            }
        }
    }
    Ok(out)
}

/// `JsonIntegerMapOption.addTo`: `{name: int}` (always an object, possibly empty).
pub fn integer_map(map: &[(String, i32)]) -> Value {
    Value::Object(map.iter().map(|(k, v)| (k.clone(), Value::from(*v))).collect())
}

/// `JsonStringMapOption.getFrom`: a NEW map, filled only when the value is a JSON object; each
/// member read as `JsonStringOption` (absent/null -> null, a non-string throws `asString`) and
/// kept only if `StringTool.isProvided`, else mapped to null.
pub fn get_string_map(obj: &JsonObject, key: &str) -> Result<Vec<(String, Option<String>)>, String> {
    let mut out: Vec<(String, Option<String>)> = Vec::new();
    if let Some(Value::Object(o)) = obj.get(key) {
        for name in o.keys() {
            let value = get_string(o, name)?.filter(|s| is_provided(Some(s)));
            match out.iter_mut().find(|(n, _)| n == name) {
                Some(e) => e.1 = value,
                None => out.push((name.clone(), value)),
            }
        }
    }
    Ok(out)
}

/// `JsonStringMapOption.addTo`: `{name: string|null}` (always an object, possibly empty).
pub fn string_map(map: &[(String, Option<String>)]) -> Value {
    Value::Object(map.iter().map(|(k, v)| (k.clone(), string_or_null(v.as_deref()))).collect())
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

    #[test]
    fn field_coordinate_and_object_semantics_match_java() {
        use crate::types::FieldCoordinate;
        let o = obj(json!({ "c": [3, 4], "n": null, "bad": [1], "s": "x", "o": { "k": 1 } }));
        assert_eq!(get_field_coordinate(&o, "c"), Ok(Some(FieldCoordinate::new(3, 4))));
        assert_eq!(get_field_coordinate(&o, "n"), Ok(None));
        assert_eq!(get_field_coordinate(&o, "missing"), Ok(None));
        assert!(get_field_coordinate(&o, "bad").is_err()); // size != 2 -> IllegalArgumentException
        assert!(get_field_coordinate(&o, "s").is_err());
        assert_eq!(field_coordinate_or_null(Some(FieldCoordinate::new(3, 4))), json!([3, 4]));
        assert_eq!(field_coordinate_or_null(None), Value::Null);
        assert!(get_object(&o, "o").unwrap().is_some());
        assert_eq!(get_object(&o, "n"), Ok(None));
        assert!(get_object(&o, "s").is_err());
    }

    #[test]
    fn string_array_and_ignore_case_match_java() {
        let o = obj(json!({ "a": ["x", null], "n": null, "bad": [1] }));
        assert_eq!(get_string_array(&o, "a"), Ok(Some(vec![Some("x".to_string()), None])));
        assert_eq!(get_string_array(&o, "n"), Ok(None));
        assert_eq!(get_string_array(&o, "missing"), Ok(None));
        assert!(get_string_array(&o, "bad").is_err());
        assert!(java_equals_ignore_case("Iron Man", "iron MAN"));
        assert!(!java_equals_ignore_case("Iron Man", "Iron Ma"));
        // Java: Character.toUpperCase('ſ') == 'S', so the long s matches 's'.
        assert!(java_equals_ignore_case("\u{17f}", "s"));
    }
}
