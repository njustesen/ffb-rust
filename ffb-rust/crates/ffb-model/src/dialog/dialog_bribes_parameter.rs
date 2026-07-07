use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogBribesParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogBribesParameter {
    pub team_id: Option<String>,
    pub max_nr_of_bribes: i32,
    pub player_ids: Vec<String>,
}

impl DialogBribesParameter {
    pub fn new() -> Self { Self::default() }

    pub fn new_with(team_id: impl Into<String>, max_nr_of_bribes: i32) -> Self {
        DialogBribesParameter { team_id: Some(team_id.into()), max_nr_of_bribes, player_ids: Vec::new() }
    }

    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_max_nr_of_bribes(&self) -> i32 { self.max_nr_of_bribes }
    pub fn get_player_ids(&self) -> &[String] { &self.player_ids }

    pub fn add_player_id(&mut self, player_id: impl Into<String>) {
        let id = player_id.into();
        if !id.is_empty() { self.player_ids.push(id); }
    }
}

impl IDialogParameter for DialogBribesParameter {
    fn get_id(&self) -> DialogId { DialogId::BRIBES }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_with_stores_team_id_and_max_bribes() {
        let p = DialogBribesParameter::new_with("team1", 3);
        assert_eq!(p.get_team_id(), Some("team1"));
        assert_eq!(p.get_max_nr_of_bribes(), 3);
    }

    #[test]
    fn add_player_id_appends_nonempty_strings() {
        let mut p = DialogBribesParameter::new_with("t", 2);
        p.add_player_id("p1");
        p.add_player_id("");
        assert_eq!(p.get_player_ids(), &["p1"]);
    }

    #[test]
    fn dialog_id_is_bribes() {
        assert_eq!(DialogBribesParameter::default().get_id(), DialogId::BRIBES);
    }

    #[test]
    fn transform_preserves_id() {
        let p = DialogBribesParameter::default();
        let t = p.transform();
        assert_eq!(t.get_id(), DialogId::BRIBES);
    }

    #[test]
    fn serde_round_trip() {
        let p = DialogBribesParameter::new_with("team2", 5);
        let json = serde_json::to_string(&p).unwrap();
        let back: DialogBribesParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_max_nr_of_bribes(), 5);
        assert_eq!(back.get_team_id(), Some("team2"));
    }
}
