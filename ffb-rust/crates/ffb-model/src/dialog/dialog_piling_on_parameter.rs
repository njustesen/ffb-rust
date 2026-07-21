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
