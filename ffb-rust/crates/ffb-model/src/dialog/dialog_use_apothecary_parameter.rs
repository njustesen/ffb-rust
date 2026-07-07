use serde::{Deserialize, Serialize};
use crate::enums::{PlayerState, SeriousInjuryKind, ApothecaryType};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogUseApothecaryParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogUseApothecaryParameter {
    pub player_id: Option<String>,
    pub player_state: Option<PlayerState>,
    pub serious_injury: Option<SeriousInjuryKind>,
    pub apothecary_types: Vec<ApothecaryType>,
}

impl DialogUseApothecaryParameter {
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_player_state(&self) -> Option<PlayerState> { self.player_state }
    pub fn get_serious_injury(&self) -> Option<SeriousInjuryKind> { self.serious_injury }
    pub fn get_apothecary_types(&self) -> &[ApothecaryType] { &self.apothecary_types }
}

impl IDialogParameter for DialogUseApothecaryParameter {
    fn get_id(&self) -> DialogId { DialogId::USE_APOTHECARY }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_use_apothecary() {
        assert_eq!(DialogUseApothecaryParameter::default().get_id(), DialogId::USE_APOTHECARY);
    }
    #[test]
    fn stores_player_id() {
        let p = DialogUseApothecaryParameter { player_id: Some("p42".into()), ..Default::default() };
        assert_eq!(p.get_player_id(), Some("p42"));
    }
    #[test]
    fn default_is_sensible() {
        let p = DialogUseApothecaryParameter::default();
        assert!(p.get_player_id().is_none());
        assert!(p.get_player_state().is_none());
        assert!(p.get_serious_injury().is_none());
        assert!(p.get_apothecary_types().is_empty());
    }
    #[test]
    fn stores_serious_injury_and_apothecary_types() {
        let p = DialogUseApothecaryParameter {
            serious_injury: Some(SeriousInjuryKind::Dead),
            apothecary_types: vec![ApothecaryType::Team, ApothecaryType::Wandering],
            ..Default::default()
        };
        assert_eq!(p.get_serious_injury(), Some(SeriousInjuryKind::Dead));
        assert_eq!(p.get_apothecary_types().len(), 2);
    }
    #[test]
    fn player_state_none_when_unset() {
        let p = DialogUseApothecaryParameter { player_state: None, ..Default::default() };
        assert!(p.get_player_state().is_none());
    }
}
