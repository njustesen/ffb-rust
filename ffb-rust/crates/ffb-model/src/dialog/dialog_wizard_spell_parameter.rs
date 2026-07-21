use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogWizardSpellParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogWizardSpellParameter {
    pub team_id: Option<String>,
}

impl DialogWizardSpellParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
}

impl IDialogParameter for DialogWizardSpellParameter {
    fn get_id(&self) -> DialogId { DialogId::WIZARD_SPELL }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
