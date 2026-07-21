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
    fn serde_round_trip() {
        let p = DialogFollowupChoiceParameter;
        let json = serde_json::to_string(&p).unwrap();
        let back: DialogFollowupChoiceParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_id(), DialogId::FOLLOWUP_CHOICE);
    }

}
