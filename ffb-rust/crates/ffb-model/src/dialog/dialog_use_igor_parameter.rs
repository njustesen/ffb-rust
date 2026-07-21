use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogUseIgorParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogUseIgorParameter {
    pub player_id: Option<String>,
}

impl DialogUseIgorParameter {
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
}

impl IDialogParameter for DialogUseIgorParameter {
    fn get_id(&self) -> DialogId { DialogId::USE_IGOR }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
