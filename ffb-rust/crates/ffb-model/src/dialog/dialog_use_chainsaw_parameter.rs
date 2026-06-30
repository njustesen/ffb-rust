use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogUseChainsawParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogUseChainsawParameter {
    pub team_id: Option<String>,
}

impl DialogUseChainsawParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
}

impl IDialogParameter for DialogUseChainsawParameter {
    fn get_id(&self) -> DialogId { DialogId::USE_CHAINSAW }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
