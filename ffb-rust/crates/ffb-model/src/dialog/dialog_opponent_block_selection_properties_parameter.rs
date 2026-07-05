use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogOpponentBlockSelectionPropertiesParameter.
/// Note: BlockRollProperties stored as JSON values (stub not yet translated).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogOpponentBlockSelectionPropertiesParameter {
    pub team_id: Option<String>,
    pub block_rolls: Vec<serde_json::Value>,
}

impl DialogOpponentBlockSelectionPropertiesParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_block_rolls(&self) -> &[serde_json::Value] { &self.block_rolls }
}

impl IDialogParameter for DialogOpponentBlockSelectionPropertiesParameter {
    fn get_id(&self) -> DialogId { DialogId::OPPONENT_BLOCK_SELECTION_PROPERTIES }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_opponent_block_selection_properties() {
        assert_eq!(DialogOpponentBlockSelectionPropertiesParameter::default().get_id(), DialogId::OPPONENT_BLOCK_SELECTION_PROPERTIES);
    }
    #[test]
    fn stores_team_id() {
        let p = DialogOpponentBlockSelectionPropertiesParameter { team_id: Some("away".into()), ..Default::default() };
        assert_eq!(p.get_team_id(), Some("away"));
    }
}
