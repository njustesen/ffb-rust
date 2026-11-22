use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogWithoutParameter.
/// Abstract base for dialogs that carry no data beyond their ID.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogWithoutParameter;

impl DialogWithoutParameter {
    /// Java `DialogWithoutParameter.toJsonValue()`: `{"dialogId": getId()}` (the subclass's id).
    pub fn to_json_value(dialog_id: super::dialog_id::DialogId) -> serde_json::Value {
        let mut obj = serde_json::Map::new();
        obj.insert("dialogId".to_string(), serde_json::Value::String(dialog_id.get_name().to_string()));
        serde_json::Value::Object(obj)
    }
}
