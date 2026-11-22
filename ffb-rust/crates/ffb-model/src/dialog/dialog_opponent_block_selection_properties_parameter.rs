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

// JSON serialization (BACKLOG §O).
impl DialogOpponentBlockSelectionPropertiesParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, _rules: crate::enums::Rules, _json: &serde_json::Value) -> Result<(), String> {
        Err("DialogOpponentBlockSelectionPropertiesParameter.initFrom: not ported yet".to_string())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        Err("DialogOpponentBlockSelectionPropertiesParameter.toJsonValue: not ported yet".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn block_rolls_stored_correctly() {
        let roll = serde_json::json!({"result": "pushed"});
        let p = DialogOpponentBlockSelectionPropertiesParameter { team_id: None, block_rolls: vec![roll.clone()] };
        assert_eq!(p.get_block_rolls().len(), 1);
        assert_eq!(p.get_block_rolls()[0], roll);
    }

}
