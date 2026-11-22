use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogUseChainsawParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogUseChainsawParameter {
    pub team_id: Option<String>,
}

impl DialogUseChainsawParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
}

impl IDialogParameter for DialogUseChainsawParameter {
    fn get_id(&self) -> DialogId { DialogId::USE_CHAINSAW }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogUseChainsawParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, _rules: crate::enums::Rules, _json: &serde_json::Value) -> Result<(), String> {
        Err("DialogUseChainsawParameter.initFrom: not ported yet".to_string())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        Err("DialogUseChainsawParameter.toJsonValue: not ported yet".to_string())
    }
}
