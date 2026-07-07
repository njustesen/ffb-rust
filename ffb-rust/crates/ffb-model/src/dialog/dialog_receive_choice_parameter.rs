use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogReceiveChoiceParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogReceiveChoiceParameter {
    pub choosing_team_id: Option<String>,
}

impl DialogReceiveChoiceParameter {
    pub fn get_choosing_team_id(&self) -> Option<&str> { self.choosing_team_id.as_deref() }
}

impl IDialogParameter for DialogReceiveChoiceParameter {
    fn get_id(&self) -> DialogId { DialogId::RECEIVE_CHOICE }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_receive_choice() {
        assert_eq!(DialogReceiveChoiceParameter::default().get_id(), DialogId::RECEIVE_CHOICE);
    }
    #[test]
    fn stores_choosing_team_id() {
        let p = DialogReceiveChoiceParameter { choosing_team_id: Some("home".into()) };
        assert_eq!(p.get_choosing_team_id(), Some("home"));
    }

    #[test]
    fn default_is_sensible() {
        let p = DialogReceiveChoiceParameter::default();
        assert!(p.get_choosing_team_id().is_none());
    }

    #[test]
    fn transform_preserves_id() {
        let p = DialogReceiveChoiceParameter::default();
        assert_eq!(p.transform().get_id(), DialogId::RECEIVE_CHOICE);
    }

    #[test]
    fn none_team_id_edge_case() {
        let p = DialogReceiveChoiceParameter { choosing_team_id: None };
        assert_eq!(p.get_choosing_team_id(), None);
    }
}
