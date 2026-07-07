use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogCoinChoiceParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogCoinChoiceParameter;

impl IDialogParameter for DialogCoinChoiceParameter {
    fn get_id(&self) -> DialogId { DialogId::COIN_CHOICE }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_coin_choice() {
        assert_eq!(DialogCoinChoiceParameter.get_id(), DialogId::COIN_CHOICE);
    }
    #[test]
    fn transform_preserves_id() {
        assert_eq!(DialogCoinChoiceParameter.transform().get_id(), DialogId::COIN_CHOICE);
    }

    #[test]
    fn default_is_sensible() {
        // Unit struct — default() and direct construction are identical
        let p = DialogCoinChoiceParameter::default();
        assert_eq!(p.get_id(), DialogId::COIN_CHOICE);
    }

    #[test]
    fn serde_round_trip() {
        let p = DialogCoinChoiceParameter;
        let json = serde_json::to_string(&p).unwrap();
        let back: DialogCoinChoiceParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_id(), DialogId::COIN_CHOICE);
    }

    #[test]
    fn clone_preserves_id() {
        let p = DialogCoinChoiceParameter;
        let cloned = p.clone();
        assert_eq!(cloned.get_id(), DialogId::COIN_CHOICE);
    }
}
