use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogKickoffReturnParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogKickoffReturnParameter;

impl IDialogParameter for DialogKickoffReturnParameter {
    fn get_id(&self) -> DialogId { DialogId::KICKOFF_RETURN }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
