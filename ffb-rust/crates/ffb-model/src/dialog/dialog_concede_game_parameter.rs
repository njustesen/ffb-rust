use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogConcedeGameParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogConcedeGameParameter;

impl IDialogParameter for DialogConcedeGameParameter {
    fn get_id(&self) -> DialogId { DialogId::CONCEDE_GAME }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
