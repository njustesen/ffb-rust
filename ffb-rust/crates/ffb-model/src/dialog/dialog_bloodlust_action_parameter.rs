use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogBloodlustActionParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogBloodlustActionParameter {
    pub change_to_move: bool,
}

impl DialogBloodlustActionParameter {
    pub fn is_change_to_move(&self) -> bool { self.change_to_move }
}

impl IDialogParameter for DialogBloodlustActionParameter {
    fn get_id(&self) -> DialogId { DialogId::BLOODLUST_ACTION }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
