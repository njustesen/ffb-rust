use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogReRollBlockForTargetsPropertiesParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogReRollBlockForTargetsPropertiesParameter {
    pub player_id: Option<String>,
    pub block_rolls: Vec<serde_json::Value>,
}

impl DialogReRollBlockForTargetsPropertiesParameter {
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_block_rolls(&self) -> &[serde_json::Value] { &self.block_rolls }
}

impl IDialogParameter for DialogReRollBlockForTargetsPropertiesParameter {
    fn get_id(&self) -> DialogId { DialogId::RE_ROLL_BLOCK_FOR_TARGETS_PROPERTIES }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
