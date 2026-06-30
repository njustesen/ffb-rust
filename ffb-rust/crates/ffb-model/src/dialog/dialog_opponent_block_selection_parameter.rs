use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogOpponentBlockSelectionParameter.
/// Note: BlockRoll stored as serialized JSON strings (BlockRoll stub not yet translated).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogOpponentBlockSelectionParameter {
    pub team_id: Option<String>,
    /// Serialized BlockRoll entries.
    pub block_rolls: Vec<serde_json::Value>,
}

impl DialogOpponentBlockSelectionParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_block_rolls(&self) -> &[serde_json::Value] { &self.block_rolls }
}

impl IDialogParameter for DialogOpponentBlockSelectionParameter {
    fn get_id(&self) -> DialogId { DialogId::OPPONENT_BLOCK_SELECTION }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
