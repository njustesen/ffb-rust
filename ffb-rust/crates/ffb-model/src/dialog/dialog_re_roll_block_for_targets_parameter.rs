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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_re_roll_block_for_targets() {
        assert_eq!(DialogReRollBlockForTargetsParameter::default().get_id(), DialogId::RE_ROLL_BLOCK_FOR_TARGETS);
    }
    #[test]
    fn stores_player_id() {
        let p = DialogReRollBlockForTargetsParameter { player_id: Some("p1".into()), ..Default::default() };
        assert_eq!(p.get_player_id(), Some("p1"));
    }

    #[test]
    fn default_is_sensible() {
        let p = DialogReRollBlockForTargetsParameter::default();
        assert!(p.get_player_id().is_none());
        assert!(p.get_block_rolls().is_empty());
    }

    #[test]
    fn block_rolls_stored_correctly() {
        let roll = serde_json::json!({"result": "both_down"});
        let p = DialogReRollBlockForTargetsParameter { player_id: None, block_rolls: vec![roll.clone()] };
        assert_eq!(p.get_block_rolls().len(), 1);
        assert_eq!(p.get_block_rolls()[0], roll);
    }

    #[test]
    fn none_player_id_edge_case() {
        let p = DialogReRollBlockForTargetsParameter { player_id: None, ..Default::default() };
        assert_eq!(p.get_player_id(), None);
    }
}
