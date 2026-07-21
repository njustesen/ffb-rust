use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogPickUpChoiceParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogPickUpChoiceParameter;

impl IDialogParameter for DialogPickUpChoiceParameter {
    fn get_id(&self) -> DialogId { DialogId::PICK_UP_CHOICE }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn serde_round_trip() {
        let p = DialogPickUpChoiceParameter;
        let json = serde_json::to_string(&p).unwrap();
        let back: DialogPickUpChoiceParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_id(), DialogId::PICK_UP_CHOICE);
    }
}
