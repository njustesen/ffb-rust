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

    #[test]
    fn default_is_sensible() {
        let p = DialogInformationOkayParameter::default();
        assert!(p.get_title().is_none());
        assert!(p.get_messages().is_empty());
        assert!(!p.is_confirm());
    }

    #[test]
    fn accessor_methods_with_non_default_values() {
        let p = DialogInformationOkayParameter {
            title: Some("Alert".into()),
            messages: vec!["line1".into(), "line2".into()],
            confirm: true,
        };
        assert_eq!(p.get_title(), Some("Alert"));
        assert_eq!(p.get_messages().len(), 2);
        assert_eq!(p.get_messages()[1], "line2");
        assert!(p.is_confirm());
    }

    #[test]
    fn none_title_and_empty_messages_is_edge_case() {
        let p = DialogInformationOkayParameter {
            title: None,
            messages: vec![],
            confirm: false,
        };
        assert!(p.get_title().is_none());
        assert!(p.get_messages().is_empty());
        assert!(!p.is_confirm());
    }
}
