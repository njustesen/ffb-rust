use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogPettyCashParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogPettyCashParameter {
    pub team_id: Option<String>,
    pub treasury: i32,
    pub team_value: i32,
    pub opponent_team_value: i32,
}

impl DialogPettyCashParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_treasury(&self) -> i32 { self.treasury }
    pub fn get_team_value(&self) -> i32 { self.team_value }
    pub fn get_opponent_team_value(&self) -> i32 { self.opponent_team_value }
}

impl IDialogParameter for DialogPettyCashParameter {
    fn get_id(&self) -> DialogId { DialogId::PETTY_CASH }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
