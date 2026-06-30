use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogSwarmingErrorParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogSwarmingErrorParameter {
    pub allowed: i32,
    pub actual: i32,
}

impl DialogSwarmingErrorParameter {
    pub fn get_allowed(&self) -> i32 { self.allowed }
    pub fn get_actual(&self) -> i32 { self.actual }
}

impl IDialogParameter for DialogSwarmingErrorParameter {
    fn get_id(&self) -> DialogId { DialogId::SWARMING_ERROR }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
