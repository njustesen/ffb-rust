use serde::{Deserialize, Serialize};
use crate::enums::PlayerState;
use crate::enums::SeriousInjuryKind;
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogApothecaryChoiceParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogApothecaryChoiceParameter {
    pub player_id: Option<String>,
    pub player_state_old: Option<PlayerState>,
    pub serious_injury_old: Option<SeriousInjuryKind>,
    pub player_state_new: Option<PlayerState>,
    pub serious_injury_new: Option<SeriousInjuryKind>,
}

impl DialogApothecaryChoiceParameter {
    pub fn new() -> Self { Self::default() }

    pub fn new_with(
        player_id: impl Into<String>,
        player_state_old: Option<PlayerState>,
        serious_injury_old: Option<SeriousInjuryKind>,
        player_state_new: Option<PlayerState>,
        serious_injury_new: Option<SeriousInjuryKind>,
    ) -> Self {
        DialogApothecaryChoiceParameter {
            player_id: Some(player_id.into()),
            player_state_old,
            serious_injury_old,
            player_state_new,
            serious_injury_new,
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_player_state_old(&self) -> Option<PlayerState> { self.player_state_old }
    pub fn get_serious_injury_old(&self) -> Option<SeriousInjuryKind> { self.serious_injury_old }
    pub fn get_player_state_new(&self) -> Option<PlayerState> { self.player_state_new }
    pub fn get_serious_injury_new(&self) -> Option<SeriousInjuryKind> { self.serious_injury_new }
}

impl IDialogParameter for DialogApothecaryChoiceParameter {
    fn get_id(&self) -> DialogId { DialogId::APOTHECARY_CHOICE }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::{PS_BADLY_HURT, PS_SERIOUS_INJURY};

    #[test]
    fn new_with_stores_player_id_and_states() {
        let p = DialogApothecaryChoiceParameter::new_with(
            "player1",
            Some(PlayerState::new(PS_BADLY_HURT)),
            None,
            Some(PlayerState::new(PS_SERIOUS_INJURY)),
            None,
        );
        assert_eq!(p.get_player_id(), Some("player1"));
        assert_eq!(p.get_player_state_old().map(|s| s.base()), Some(PS_BADLY_HURT));
        assert_eq!(p.get_player_state_new().map(|s| s.base()), Some(PS_SERIOUS_INJURY));
    }

    #[test]
    fn dialog_id_is_apothecary_choice() {
        assert_eq!(DialogApothecaryChoiceParameter::new().get_id(), DialogId::APOTHECARY_CHOICE);
    }

    #[test]
    fn default_has_no_player_id() {
        assert!(DialogApothecaryChoiceParameter::new().get_player_id().is_none());
    }

    #[test]
    fn transform_preserves_id() {
        let p = DialogApothecaryChoiceParameter::new();
        let t = p.transform();
        assert_eq!(t.get_id(), DialogId::APOTHECARY_CHOICE);
    }

    #[test]
    fn serde_round_trip() {
        let p = DialogApothecaryChoiceParameter::new_with("p42", None, None, None, None);
        let json = serde_json::to_string(&p).unwrap();
        let back: DialogApothecaryChoiceParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_player_id(), Some("p42"));
    }
}
