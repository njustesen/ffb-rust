use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogPilingOnParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogPilingOnParameter {
    pub player_id: Option<String>,
    pub re_roll_injury: bool,
    pub uses_a_team_reroll: bool,
}

impl DialogPilingOnParameter {
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn is_re_roll_injury(&self) -> bool { self.re_roll_injury }
    pub fn is_uses_a_team_reroll(&self) -> bool { self.uses_a_team_reroll }
}

impl IDialogParameter for DialogPilingOnParameter {
    fn get_id(&self) -> DialogId { DialogId::PILING_ON }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogPilingOnParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, _rules: crate::enums::Rules, _json: &serde_json::Value) -> Result<(), String> {
        Err("DialogPilingOnParameter.initFrom: not ported yet".to_string())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        Err("DialogPilingOnParameter.toJsonValue: not ported yet".to_string())
    }
}
