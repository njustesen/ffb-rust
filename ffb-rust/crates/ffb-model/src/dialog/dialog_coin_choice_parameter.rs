use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogCoinChoiceParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogCoinChoiceParameter;

impl IDialogParameter for DialogCoinChoiceParameter {
    fn get_id(&self) -> DialogId { DialogId::COIN_CHOICE }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
