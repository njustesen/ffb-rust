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
}
