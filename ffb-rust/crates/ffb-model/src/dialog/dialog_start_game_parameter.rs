use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogStartGameParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogStartGameParameter;

impl IDialogParameter for DialogStartGameParameter {
    fn get_id(&self) -> DialogId { DialogId::START_GAME }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_start_game() {
        assert_eq!(DialogStartGameParameter.get_id(), DialogId::START_GAME);
    }
    #[test]
    fn transform_preserves_id() {
        assert_eq!(DialogStartGameParameter.transform().get_id(), DialogId::START_GAME);
    }
    #[test]
    fn default_matches_unit_struct() {
        assert_eq!(DialogStartGameParameter::default().get_id(), DialogId::START_GAME);
    }
    #[test]
    fn double_transform_preserves_id() {
        let once = DialogStartGameParameter.transform();
        assert_eq!(once.transform().get_id(), DialogId::START_GAME);
    }
    #[test]
    fn id_name_is_start_game() {
        assert_eq!(DialogStartGameParameter.get_id().get_name(), "startGame");
    }
}
