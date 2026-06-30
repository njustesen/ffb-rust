use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogBriberyAndCorruptionParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogBriberyAndCorruptionParameter {
    pub team_id: Option<String>,
}

impl DialogBriberyAndCorruptionParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
}

impl IDialogParameter for DialogBriberyAndCorruptionParameter {
    fn get_id(&self) -> DialogId { DialogId::BRIBERY_AND_CORRUPTION_RE_ROLL }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
