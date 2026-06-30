use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogInformationOkayParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogInformationOkayParameter {
    pub title: Option<String>,
    pub messages: Vec<String>,
    pub confirm: bool,
}

impl DialogInformationOkayParameter {
    pub fn get_title(&self) -> Option<&str> { self.title.as_deref() }
    pub fn get_messages(&self) -> &[String] { &self.messages }
    pub fn is_confirm(&self) -> bool { self.confirm }
}

impl IDialogParameter for DialogInformationOkayParameter {
    fn get_id(&self) -> DialogId { DialogId::INFORMATION_OKAY }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
