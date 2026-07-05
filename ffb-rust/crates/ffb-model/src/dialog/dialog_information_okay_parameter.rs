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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialog_id_is_information_okay() {
        assert_eq!(DialogInformationOkayParameter::default().get_id(), DialogId::INFORMATION_OKAY);
    }

    #[test]
    fn stores_title_messages_and_confirm() {
        let p = DialogInformationOkayParameter {
            title: Some("Info".into()),
            messages: vec!["msg1".into()],
            confirm: true,
        };
        assert_eq!(p.get_title(), Some("Info"));
        assert_eq!(p.get_messages(), &["msg1"]);
        assert!(p.is_confirm());
    }
}
