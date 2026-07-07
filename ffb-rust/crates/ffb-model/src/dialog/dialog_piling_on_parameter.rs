use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogPilingOnParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogPilingOnParameter {
    pub player_id: Option<String>,
    pub re_roll_injury: bool,
    pub uses_a_team_reroll: bool,
}

impl DialogPilingOnParameter {
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn is_re_roll_injury(&self) -> bool { self.re_roll_injury }
    pub fn is_uses_a_team_reroll(&self) -> bool { self.uses_a_team_reroll }
}

impl IDialogParameter for DialogPilingOnParameter {
    fn get_id(&self) -> DialogId { DialogId::PILING_ON }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialog_id_is_piling_on() {
        assert_eq!(DialogPilingOnParameter::default().get_id(), DialogId::PILING_ON);
    }

    #[test]
    fn stores_flags_and_player_id() {
        let p = DialogPilingOnParameter {
            player_id: Some("atk".into()),
            re_roll_injury: true,
            uses_a_team_reroll: false,
        };
        assert_eq!(p.get_player_id(), Some("atk"));
        assert!(p.is_re_roll_injury());
        assert!(!p.is_uses_a_team_reroll());
    }

    #[test]
    fn default_is_sensible() {
        let p = DialogPilingOnParameter::default();
        assert!(p.get_player_id().is_none());
        assert!(!p.is_re_roll_injury());
        assert!(!p.is_uses_a_team_reroll());
    }

    #[test]
    fn uses_a_team_reroll_true() {
        let p = DialogPilingOnParameter { uses_a_team_reroll: true, ..Default::default() };
        assert!(p.is_uses_a_team_reroll());
    }

    #[test]
    fn none_player_id_edge_case() {
        let p = DialogPilingOnParameter { player_id: None, ..Default::default() };
        assert_eq!(p.get_player_id(), None);
    }
}
