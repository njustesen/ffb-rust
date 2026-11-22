use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogUseIgorParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogUseIgorParameter {
    pub player_id: Option<String>,
}

impl DialogUseIgorParameter {
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
}

impl IDialogParameter for DialogUseIgorParameter {
    fn get_id(&self) -> DialogId { DialogId::USE_IGOR }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogUseIgorParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, _rules: crate::enums::Rules, _json: &serde_json::Value) -> Result<(), String> {
        Err("DialogUseIgorParameter.initFrom: not ported yet".to_string())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        Err("DialogUseIgorParameter.toJsonValue: not ported yet".to_string())
    }
}
