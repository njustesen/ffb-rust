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
}
