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
    fn dialog_id_is_pick_up_choice() {
        assert_eq!(DialogPickUpChoiceParameter.get_id(), DialogId::PICK_UP_CHOICE);
    }
    #[test]
    fn transform_preserves_id() {
        assert_eq!(DialogPickUpChoiceParameter.transform().get_id(), DialogId::PICK_UP_CHOICE);
    }
}
