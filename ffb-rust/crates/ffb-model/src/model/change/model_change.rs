use serde::{Deserialize, Serialize};
use crate::enums::{ModelChangeId, ModelChangeDataType};

/// 1:1 translation of com.fumbbl.ffb.model.change.ModelChange.
///
/// `value` is the change's JSON value as it travels on the wire (Java `MODEL_CHANGE_VALUE`);
/// `ModelChangeDataType::from_json_value` decodes it into the typed `ModelChangeValue` that
/// `ModelChangeProcessor::apply` works on (Java decodes in `initFrom`; the replay path decodes the
/// whole list before applying any of it, as Java does).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelChange {
    pub change_id: ModelChangeId,
    pub data_type: ModelChangeDataType,
    pub value: serde_json::Value,
    /// Java `fKey` (a player id, `home`/`away`, or null).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
}

impl ModelChange {
    /// Java `ModelChange.HOME`.
    pub const HOME: &'static str = "home";
    /// Java `ModelChange.AWAY`.
    pub const AWAY: &'static str = "away";

    pub fn new(change_id: ModelChangeId, data_type: ModelChangeDataType, value: serde_json::Value) -> Self {
        Self { change_id, data_type, value, key: None }
    }

    /// Java `new ModelChange(changeId, key, value)` (value in its JSON form).
    pub fn with_key(change_id: ModelChangeId, key: Option<String>, value: serde_json::Value) -> Self {
        Self { change_id, data_type: change_id.data_type(), value, key }
    }

    /// Java `getKey()`.
    pub fn get_key(&self) -> Option<&str> {
        self.key.as_deref()
    }

    /// Java `ModelChange.initFrom`: `modelChangeId` (ModelChangeIdFactory -- an unknown id is
    /// null, and Java then NPEs on `fChangeId.fromJsonValue`, so `Err` here), `modelChangeKey`
    /// (string or null), `modelChangeValue` (kept as JSON; decoded by the data type).
    pub fn init_from(json: &serde_json::Value) -> Result<Self, String> {
        let obj = json.as_object().ok_or_else(|| "modelChange: not an object".to_string())?;
        let id_name = match obj.get("modelChangeId") {
            None | Some(serde_json::Value::Null) => None,
            Some(serde_json::Value::String(s)) => Some(s.as_str()),
            Some(other) => return Err(format!("modelChangeId: not a string: {other}")),
        };
        let change_id = id_name
            .and_then(ModelChangeId::for_name)
            .ok_or_else(|| format!("modelChangeId {id_name:?} unknown (Java NPE)"))?;
        let key = match obj.get("modelChangeKey") {
            None | Some(serde_json::Value::Null) => None,
            Some(serde_json::Value::String(s)) => Some(s.clone()),
            Some(other) => return Err(format!("modelChangeKey: not a string: {other}")),
        };
        let value = obj.get("modelChangeValue").cloned().unwrap_or(serde_json::Value::Null);
        Ok(Self::with_key(change_id, key, value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::{ModelChangeId, ModelChangeDataType};

    #[test]
    fn new_sets_change_id() {
        let mc = ModelChange::new(ModelChangeId::GameSetHomePlaying, ModelChangeDataType::String, serde_json::Value::Null);
        assert_eq!(mc.change_id, ModelChangeId::GameSetHomePlaying);
    }

    #[test]
    fn data_type_stored() {
        let mc = ModelChange::new(ModelChangeId::GameSetHomePlaying, ModelChangeDataType::Boolean, serde_json::Value::Null);
        assert_eq!(mc.data_type, ModelChangeDataType::Boolean);
    }

    #[test]
    fn init_from_reads_id_key_value() {
        let mc = ModelChange::init_from(&serde_json::json!({
            "netCommandId": "x", "modelChangeId": "fieldModelSetPlayerState",
            "modelChangeKey": "12345", "modelChangeValue": 1})).unwrap();
        assert_eq!(mc.change_id, ModelChangeId::FieldModelSetPlayerState);
        assert_eq!(mc.data_type, ModelChangeDataType::PlayerState);
        assert_eq!(mc.get_key(), Some("12345"));
        assert_eq!(mc.value, serde_json::json!(1));
        let mc = ModelChange::init_from(&serde_json::json!({"modelChangeId": "gameSetHalf", "modelChangeValue": 2})).unwrap();
        assert_eq!(mc.get_key(), None);
        assert!(ModelChange::init_from(&serde_json::json!({"modelChangeId": "turnDataSetCheeringFansBlockAssist"})).is_err());
    }
}
