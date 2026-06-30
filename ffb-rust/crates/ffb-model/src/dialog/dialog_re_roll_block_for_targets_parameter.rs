use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogReRollBlockForTargetsParameter.
/// Note: BlockRoll stored as JSON values (stub not yet translated).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogReRollBlockForTargetsParameter {
    pub player_id: Option<String>,
    pub block_rolls: Vec<serde_json::Value>,
}

impl DialogReRollBlockForTargetsParameter {
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_block_rolls(&self) -> &[serde_json::Value] { &self.block_rolls }
}

impl IDialogParameter for DialogReRollBlockForTargetsParameter {
    fn get_id(&self) -> DialogId { DialogId::RE_ROLL_BLOCK_FOR_TARGETS }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
