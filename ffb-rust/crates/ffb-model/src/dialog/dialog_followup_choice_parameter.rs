use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogFollowupChoiceParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogFollowupChoiceParameter;

impl IDialogParameter for DialogFollowupChoiceParameter {
    fn get_id(&self) -> DialogId { DialogId::FOLLOWUP_CHOICE }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_followup_choice() {
        assert_eq!(DialogFollowupChoiceParameter.get_id(), DialogId::FOLLOWUP_CHOICE);
    }
    #[test]
    fn transform_preserves_id() {
        assert_eq!(DialogFollowupChoiceParameter.transform().get_id(), DialogId::FOLLOWUP_CHOICE);
    }

    #[test]
    fn default_is_sensible() {
        let p = DialogFollowupChoiceParameter::default();
        assert_eq!(p.get_id(), DialogId::FOLLOWUP_CHOICE);
    }

    #[test]
    fn serde_round_trip() {
        let p = DialogFollowupChoiceParameter;
        let json = serde_json::to_string(&p).unwrap();
        let back: DialogFollowupChoiceParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_id(), DialogId::FOLLOWUP_CHOICE);
    }

    #[test]
    fn clone_preserves_id() {
        let p = DialogFollowupChoiceParameter;
        let cloned = p.clone();
        assert_eq!(cloned.get_id(), DialogId::FOLLOWUP_CHOICE);
    }
}
