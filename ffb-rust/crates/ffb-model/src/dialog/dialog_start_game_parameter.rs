use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogStartGameParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogStartGameParameter;

impl IDialogParameter for DialogStartGameParameter {
    fn get_id(&self) -> DialogId { DialogId::START_GAME }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
