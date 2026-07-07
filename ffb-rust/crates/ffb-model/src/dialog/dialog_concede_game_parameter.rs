use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogConcedeGameParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogConcedeGameParameter;

impl IDialogParameter for DialogConcedeGameParameter {
    fn get_id(&self) -> DialogId { DialogId::CONCEDE_GAME }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_concede_game() {
        assert_eq!(DialogConcedeGameParameter.get_id(), DialogId::CONCEDE_GAME);
    }
    #[test]
    fn transform_preserves_id() {
        assert_eq!(DialogConcedeGameParameter.transform().get_id(), DialogId::CONCEDE_GAME);
    }

    #[test]
    fn default_is_sensible() {
        let p = DialogConcedeGameParameter::default();
        assert_eq!(p.get_id(), DialogId::CONCEDE_GAME);
    }

    #[test]
    fn serde_round_trip() {
        let p = DialogConcedeGameParameter;
        let json = serde_json::to_string(&p).unwrap();
        let back: DialogConcedeGameParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_id(), DialogId::CONCEDE_GAME);
    }

    #[test]
    fn clone_preserves_id() {
        let p = DialogConcedeGameParameter;
        let cloned = p.clone();
        assert_eq!(cloned.get_id(), DialogId::CONCEDE_GAME);
    }
}
