use serde::{Deserialize, Serialize};
use crate::enums::PlayerAction;
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogConfirmEndActionParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogConfirmEndActionParameter {
    pub team_id: Option<String>,
    pub player_action: Option<PlayerAction>,
}

impl DialogConfirmEndActionParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn set_team_id(&mut self, team_id: impl Into<String>) { self.team_id = Some(team_id.into()); }
    pub fn get_player_action(&self) -> Option<PlayerAction> { self.player_action }
    pub fn set_player_action(&mut self, action: PlayerAction) { self.player_action = Some(action); }
}

impl IDialogParameter for DialogConfirmEndActionParameter {
    fn get_id(&self) -> DialogId { DialogId::CONFIRM_END_ACTION }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogConfirmEndActionParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, _rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::CONFIRM_END_ACTION, obj)?;
        self.team_id = jj::get_string(obj, "teamId")?;
        // JsonEnumWithNameOption over PlayerActionFactory (unknown -> null), as in ActingPlayer.
        self.player_action = jj::get_enum_name(obj, "playerAction")?.and_then(PlayerAction::from_name);
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".into(), Value::from(DialogId::CONFIRM_END_ACTION.get_name()));
        o.insert("teamId".into(), jj::string_or_null(self.team_id.as_deref()));
        o.insert("playerAction".into(), jj::string_or_null(self.player_action.map(PlayerAction::name)));
        Ok(Value::Object(o))
    }
}
