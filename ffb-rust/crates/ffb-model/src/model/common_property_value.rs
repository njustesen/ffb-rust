use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.model.CommonPropertyValue.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommonPropertyValue {
    pub value: String,
}

impl CommonPropertyValue {
    pub fn new(value: String) -> Self { Self { value } }
    pub fn get_value(&self) -> &str { &self.value }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_empty() {
        assert!(CommonPropertyValue::default().value.is_empty());
    }

    #[test]
    fn get_value_returns_value() {
        let v = CommonPropertyValue::new("true".to_string());
        assert_eq!(v.get_value(), "true");
    }
}
