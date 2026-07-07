use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogSelectBlitzTargetParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogSelectBlitzTargetParameter;

impl IDialogParameter for DialogSelectBlitzTargetParameter {
    fn get_id(&self) -> DialogId { DialogId::SELECT_BLITZ_TARGET }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_select_blitz_target() {
        assert_eq!(DialogSelectBlitzTargetParameter.get_id(), DialogId::SELECT_BLITZ_TARGET);
    }
    #[test]
    fn transform_preserves_id() {
        assert_eq!(DialogSelectBlitzTargetParameter.transform().get_id(), DialogId::SELECT_BLITZ_TARGET);
    }

    #[test]
    fn default_is_sensible() {
        let _p = DialogSelectBlitzTargetParameter::default();
        // Unit struct — default constructs without panic
    }

    #[test]
    fn clone_has_same_id() {
        let p = DialogSelectBlitzTargetParameter;
        assert_eq!(p.clone().get_id(), DialogId::SELECT_BLITZ_TARGET);
    }

    #[test]
    fn serde_round_trip() {
        let p = DialogSelectBlitzTargetParameter;
        let json = serde_json::to_string(&p).unwrap();
        let back: DialogSelectBlitzTargetParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_id(), DialogId::SELECT_BLITZ_TARGET);
    }
}
