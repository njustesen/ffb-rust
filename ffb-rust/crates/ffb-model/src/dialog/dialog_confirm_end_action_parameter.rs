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
