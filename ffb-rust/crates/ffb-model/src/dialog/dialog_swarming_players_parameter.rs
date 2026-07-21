use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogSwarmingPlayersParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogSwarmingPlayersParameter {
    pub amount: i32,
    pub restrict_placement: bool,
}

impl DialogSwarmingPlayersParameter {
    pub fn get_amount(&self) -> i32 { self.amount }
    pub fn is_restrict_placement(&self) -> bool { self.restrict_placement }
}

impl IDialogParameter for DialogSwarmingPlayersParameter {
    fn get_id(&self) -> DialogId { DialogId::SWARMING }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
