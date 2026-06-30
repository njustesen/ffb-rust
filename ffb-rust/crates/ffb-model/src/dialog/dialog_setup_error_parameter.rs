use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogSetupErrorParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogSetupErrorParameter {
    pub team_id: Option<String>,
    pub setup_errors: Vec<String>,
}

impl DialogSetupErrorParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_setup_errors(&self) -> &[String] { &self.setup_errors }
    pub fn add_setup_error(&mut self, error: impl Into<String>) {
        let s = error.into();
        if !s.is_empty() { self.setup_errors.push(s); }
    }
}

impl IDialogParameter for DialogSetupErrorParameter {
    fn get_id(&self) -> DialogId { DialogId::SETUP_ERROR }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
