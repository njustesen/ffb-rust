use serde::{Deserialize, Serialize};
use crate::enums::{ModelChangeId, ModelChangeDataType};

/// 1:1 translation of com.fumbbl.ffb.model.change.ModelChange.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelChange {
    pub change_id: ModelChangeId,
    pub data_type: ModelChangeDataType,
    pub value: serde_json::Value,
}

impl ModelChange {
    pub fn new(change_id: ModelChangeId, data_type: ModelChangeDataType, value: serde_json::Value) -> Self {
        Self { change_id, data_type, value }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::{ModelChangeId, ModelChangeDataType};

    #[test]
    fn new_sets_change_id() {
        let mc = ModelChange::new(ModelChangeId::GameSetActingTeam, ModelChangeDataType::String, serde_json::Value::Null);
        assert_eq!(mc.change_id, ModelChangeId::GameSetActingTeam);
    }

    #[test]
    fn data_type_stored() {
        let mc = ModelChange::new(ModelChangeId::GameSetActingTeam, ModelChangeDataType::Boolean, serde_json::Value::Null);
        assert_eq!(mc.data_type, ModelChangeDataType::Boolean);
    }
}
