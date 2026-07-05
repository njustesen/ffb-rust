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
}
