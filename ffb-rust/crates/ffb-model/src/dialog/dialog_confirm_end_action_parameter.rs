use serde::{Deserialize, Serialize};
use crate::enums::PlayerAction;
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogConfirmEndActionParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogConfirmEndActionParameter {
    pub team_id: Option<String>,
    pub player_action: Option<PlayerAction>,
}

impl DialogConfirmEndActionParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn set_team_id(&mut self, team_id: impl Into<String>) { self.team_id = Some(team_id.into()); }
    pub fn get_player_action(&self) -> Option<PlayerAction> { self.player_action }
    pub fn set_player_action(&mut self, action: PlayerAction) { self.player_action = Some(action); }
}

impl IDialogParameter for DialogConfirmEndActionParameter {
    fn get_id(&self) -> DialogId { DialogId::CONFIRM_END_ACTION }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::PlayerAction;

    #[test]
    fn dialog_id_is_confirm_end_action() {
        assert_eq!(DialogConfirmEndActionParameter::default().get_id(), DialogId::CONFIRM_END_ACTION);
    }

    #[test]
    fn set_team_id_and_player_action() {
        let mut p = DialogConfirmEndActionParameter::default();
        p.set_team_id("team1");
        p.set_player_action(PlayerAction::Move);
        assert_eq!(p.get_team_id(), Some("team1"));
        assert_eq!(p.get_player_action(), Some(PlayerAction::Move));
    }

    #[test]
    fn default_is_sensible() {
        let p = DialogConfirmEndActionParameter::default();
        assert!(p.get_team_id().is_none());
        assert!(p.get_player_action().is_none());
    }

    #[test]
    fn accessor_methods_with_non_default_values() {
        let p = DialogConfirmEndActionParameter {
            team_id: Some("away".into()),
            player_action: Some(PlayerAction::Block),
        };
        assert_eq!(p.get_team_id(), Some("away"));
        assert_eq!(p.get_player_action(), Some(PlayerAction::Block));
    }

    #[test]
    fn none_player_action_is_edge_case() {
        let p = DialogConfirmEndActionParameter {
            team_id: Some("home".into()),
            player_action: None,
        };
        assert_eq!(p.get_team_id(), Some("home"));
        assert!(p.get_player_action().is_none());
    }
}
