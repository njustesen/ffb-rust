use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.model.FieldModelChangeEvent.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FieldModelChangeEvent {
    pub change_type: String,
}

impl FieldModelChangeEvent {
    pub fn new(change_type: String) -> Self { Self { change_type } }
    pub fn get_change_type(&self) -> &str { &self.change_type }
}
