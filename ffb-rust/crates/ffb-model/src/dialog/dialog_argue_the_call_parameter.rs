use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogArgueTheCallParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogArgueTheCallParameter {
    pub team_id: Option<String>,
    pub player_ids: Vec<String>,
    pub stay_on_pitch: bool,
    pub friends_with_the_ref: bool,
    pub biased_refs: i32,
}

impl DialogArgueTheCallParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_player_ids(&self) -> &[String] { &self.player_ids }
    pub fn is_stay_on_pitch(&self) -> bool { self.stay_on_pitch }
    pub fn is_friends_with_the_ref(&self) -> bool { self.friends_with_the_ref }
    pub fn get_biased_refs(&self) -> i32 { self.biased_refs }
    pub fn add_player_id(&mut self, id: impl Into<String>) {
        let s = id.into();
        if !s.is_empty() { self.player_ids.push(s); }
    }
}

impl IDialogParameter for DialogArgueTheCallParameter {
    fn get_id(&self) -> DialogId { DialogId::ARGUE_THE_CALL }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_player_id_appends_nonempty_strings() {
        let mut p = DialogArgueTheCallParameter::default();
        p.add_player_id("p1");
        p.add_player_id("p2");
        assert_eq!(p.get_player_ids(), &["p1", "p2"]);
    }

    #[test]
    fn add_player_id_ignores_empty_string() {
        let mut p = DialogArgueTheCallParameter::default();
        p.add_player_id("");
        assert!(p.get_player_ids().is_empty());
    }

    #[test]
    fn dialog_id_is_argue_the_call() {
        assert_eq!(DialogArgueTheCallParameter::default().get_id(), DialogId::ARGUE_THE_CALL);
    }

    #[test]
    fn stay_on_pitch_flag() {
        let p = DialogArgueTheCallParameter { stay_on_pitch: true, ..Default::default() };
        assert!(p.is_stay_on_pitch());
    }

    #[test]
    fn biased_refs_stored() {
        let p = DialogArgueTheCallParameter { biased_refs: 2, ..Default::default() };
        assert_eq!(p.get_biased_refs(), 2);
    }
}
