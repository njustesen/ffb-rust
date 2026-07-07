use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogPassBlockParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogPassBlockParameter;

impl IDialogParameter for DialogPassBlockParameter {
    fn get_id(&self) -> DialogId { DialogId::PASS_BLOCK }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_pass_block() {
        assert_eq!(DialogPassBlockParameter.get_id(), DialogId::PASS_BLOCK);
    }
    #[test]
    fn transform_preserves_id() {
        assert_eq!(DialogPassBlockParameter.transform().get_id(), DialogId::PASS_BLOCK);
    }

    #[test]
    fn default_is_sensible() {
        let _p = DialogPassBlockParameter::default();
        // Unit struct — default constructs without panic
    }

    #[test]
    fn clone_has_same_id() {
        let p = DialogPassBlockParameter;
        assert_eq!(p.clone().get_id(), DialogId::PASS_BLOCK);
    }

    #[test]
    fn serde_round_trip() {
        let p = DialogPassBlockParameter;
        let json = serde_json::to_string(&p).unwrap();
        let back: DialogPassBlockParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_id(), DialogId::PASS_BLOCK);
    }
}
