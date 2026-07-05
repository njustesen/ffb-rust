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
}
