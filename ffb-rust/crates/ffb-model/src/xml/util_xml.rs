/// 1:1 translation of the read-side of com.fumbbl.ffb.xml.UtilXml.
/// The write side (`addToXml`/`TransformerHandler`) is out of scope — this crate only
/// needs to read the standalone-server's disk roster/team XML, not produce it.
use std::collections::HashMap;

/// Java: `Attributes` — collected once per start-tag event into a plain map since
/// quick-xml attribute iterators are tied to the event's lifetime.
pub type XmlAttributes = HashMap<String, String>;

/// Java: `UtilXml.getStringAttribute(Attributes, String)`.
pub fn get_string_attribute(atts: &XmlAttributes, attribute: &str) -> Option<String> {
    atts.get(attribute).map(|v| v.trim().to_string())
}

/// Java: `UtilXml.getIntAttribute(Attributes, String, int)`.
pub fn get_int_attribute_or(atts: &XmlAttributes, attribute: &str, default_value: i32) -> i32 {
    match get_string_attribute(atts, attribute) {
        Some(v) => v.parse().unwrap_or(default_value),
        None => default_value,
    }
}

/// Java: `UtilXml.getIntAttribute(Attributes, String)`.
pub fn get_int_attribute(atts: &XmlAttributes, attribute: &str) -> i32 {
    get_int_attribute_or(atts, attribute, 0)
}

/// Java: `UtilXml.getBooleanAttribute(Attributes, String)`.
pub fn get_boolean_attribute(atts: &XmlAttributes, attribute: &str) -> bool {
    get_string_attribute(atts, attribute)
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn atts(pairs: &[(&str, &str)]) -> XmlAttributes {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    #[test]
    fn get_string_attribute_trims_and_returns() {
        let a = atts(&[("id", "  42  ")]);
        assert_eq!(get_string_attribute(&a, "id"), Some("42".to_string()));
    }

    #[test]
    fn get_string_attribute_missing_returns_none() {
        let a = atts(&[]);
        assert_eq!(get_string_attribute(&a, "id"), None);
    }

    #[test]
    fn get_int_attribute_or_parses_value() {
        let a = atts(&[("size", "3")]);
        assert_eq!(get_int_attribute_or(&a, "size", -1), 3);
    }

    #[test]
    fn get_int_attribute_or_missing_returns_default() {
        let a = atts(&[]);
        assert_eq!(get_int_attribute_or(&a, "size", -1), -1);
    }

    #[test]
    fn get_int_attribute_missing_returns_zero() {
        let a = atts(&[]);
        assert_eq!(get_int_attribute(&a, "size"), 0);
    }

    #[test]
    fn get_boolean_attribute_true() {
        let a = atts(&[("recovering", "true")]);
        assert!(get_boolean_attribute(&a, "recovering"));
    }

    #[test]
    fn get_boolean_attribute_missing_is_false() {
        let a = atts(&[]);
        assert!(!get_boolean_attribute(&a, "recovering"));
    }

    #[test]
    fn get_boolean_attribute_case_insensitive() {
        let a = atts(&[("recovering", "TRUE")]);
        assert!(get_boolean_attribute(&a, "recovering"));
    }
}
