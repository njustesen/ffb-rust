use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogWinningsReRollParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogWinningsReRollParameter {
    pub team_id: Option<String>,
    pub old_roll: i32,
}

impl DialogWinningsReRollParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_old_roll(&self) -> i32 { self.old_roll }
}

impl IDialogParameter for DialogWinningsReRollParameter {
    fn get_id(&self) -> DialogId { DialogId::WINNINGS_RE_ROLL }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
