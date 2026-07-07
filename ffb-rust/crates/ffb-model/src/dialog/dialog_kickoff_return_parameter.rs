use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogKickoffReturnParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogKickoffReturnParameter;

impl IDialogParameter for DialogKickoffReturnParameter {
    fn get_id(&self) -> DialogId { DialogId::KICKOFF_RETURN }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_kickoff_return() {
        assert_eq!(DialogKickoffReturnParameter.get_id(), DialogId::KICKOFF_RETURN);
    }
    #[test]
    fn transform_preserves_id() {
        assert_eq!(DialogKickoffReturnParameter.transform().get_id(), DialogId::KICKOFF_RETURN);
    }

    #[test]
    fn default_is_sensible() {
        let _p = DialogKickoffReturnParameter::default();
        // Unit struct — default constructs without panic
    }

    #[test]
    fn clone_has_same_id() {
        let p = DialogKickoffReturnParameter;
        assert_eq!(p.clone().get_id(), DialogId::KICKOFF_RETURN);
    }

    #[test]
    fn serde_round_trip() {
        let p = DialogKickoffReturnParameter;
        let json = serde_json::to_string(&p).unwrap();
        let back: DialogKickoffReturnParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_id(), DialogId::KICKOFF_RETURN);
    }
}
