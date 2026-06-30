use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogReceiveChoiceParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogReceiveChoiceParameter {
    pub choosing_team_id: Option<String>,
}

impl DialogReceiveChoiceParameter {
    pub fn get_choosing_team_id(&self) -> Option<&str> { self.choosing_team_id.as_deref() }
}

impl IDialogParameter for DialogReceiveChoiceParameter {
    fn get_id(&self) -> DialogId { DialogId::RECEIVE_CHOICE }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
