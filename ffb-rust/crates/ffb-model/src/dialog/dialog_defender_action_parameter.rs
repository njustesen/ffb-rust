use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogDefenderActionParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogDefenderActionParameter;

impl IDialogParameter for DialogDefenderActionParameter {
    fn get_id(&self) -> DialogId { DialogId::DEFENDER_ACTION }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_defender_action() {
        assert_eq!(DialogDefenderActionParameter.get_id(), DialogId::DEFENDER_ACTION);
    }
    #[test]
    fn transform_preserves_id() {
        assert_eq!(DialogDefenderActionParameter.transform().get_id(), DialogId::DEFENDER_ACTION);
    }

    #[test]
    fn default_is_sensible() {
        let p = DialogDefenderActionParameter::default();
        assert_eq!(p.get_id(), DialogId::DEFENDER_ACTION);
    }

    #[test]
    fn serde_round_trip() {
        let p = DialogDefenderActionParameter;
        let json = serde_json::to_string(&p).unwrap();
        let back: DialogDefenderActionParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_id(), DialogId::DEFENDER_ACTION);
    }

    #[test]
    fn clone_preserves_id() {
        let p = DialogDefenderActionParameter;
        let cloned = p.clone();
        assert_eq!(cloned.get_id(), DialogId::DEFENDER_ACTION);
    }
}
