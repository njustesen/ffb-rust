use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogPuntToCrowdParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogPuntToCrowdParameter;

impl IDialogParameter for DialogPuntToCrowdParameter {
    fn get_id(&self) -> DialogId { DialogId::PUNT_TO_CROWD }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
