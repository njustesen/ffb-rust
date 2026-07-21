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
