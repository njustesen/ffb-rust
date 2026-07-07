use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogPileDriverParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogPileDriverParameter {
    pub knocked_down_players: Vec<String>,
    pub team_id: Option<String>,
}

impl DialogPileDriverParameter {
    pub fn get_knocked_down_players(&self) -> &[String] { &self.knocked_down_players }
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn add_knocked_down_player(&mut self, id: impl Into<String>) {
        let s = id.into();
        if !s.is_empty() { self.knocked_down_players.push(s); }
    }
}

impl IDialogParameter for DialogPileDriverParameter {
    fn get_id(&self) -> DialogId { DialogId::PILE_DRIVER }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_knocked_down_player_appends_nonempty() {
        let mut p = DialogPileDriverParameter::default();
        p.add_knocked_down_player("p1");
        p.add_knocked_down_player("");
        assert_eq!(p.get_knocked_down_players(), &["p1"]);
    }

    #[test]
    fn dialog_id_is_pile_driver() {
        assert_eq!(DialogPileDriverParameter::default().get_id(), DialogId::PILE_DRIVER);
    }

    #[test]
    fn default_is_sensible() {
        let p = DialogPileDriverParameter::default();
        assert!(p.get_knocked_down_players().is_empty());
        assert!(p.get_team_id().is_none());
    }

    #[test]
    fn team_id_accessor() {
        let p = DialogPileDriverParameter { knocked_down_players: vec![], team_id: Some("away".into()) };
        assert_eq!(p.get_team_id(), Some("away"));
    }

    #[test]
    fn add_empty_string_is_ignored() {
        let mut p = DialogPileDriverParameter::default();
        p.add_knocked_down_player("");
        assert!(p.get_knocked_down_players().is_empty());
    }
}
