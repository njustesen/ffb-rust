use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogSelectBlitzTargetParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogSelectBlitzTargetParameter;

impl IDialogParameter for DialogSelectBlitzTargetParameter {
    fn get_id(&self) -> DialogId { DialogId::SELECT_BLITZ_TARGET }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
