use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogJourneymenParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogJourneymenParameter {
    pub team_id: Option<String>,
    pub nr_of_slots: i32,
    pub position_ids: Vec<String>,
}

impl DialogJourneymenParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_nr_of_slots(&self) -> i32 { self.nr_of_slots }
    pub fn get_position_ids(&self) -> &[String] { &self.position_ids }
    pub fn add_position_id(&mut self, id: impl Into<String>) {
        let s = id.into();
        if !s.is_empty() { self.position_ids.push(s); }
    }
}

impl IDialogParameter for DialogJourneymenParameter {
    fn get_id(&self) -> DialogId { DialogId::JOURNEYMEN }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_position_id_appends_nonempty_strings() {
        let mut p = DialogJourneymenParameter::default();
        p.add_position_id("pos1");
        p.add_position_id("");
        assert_eq!(p.get_position_ids(), &["pos1"]);
    }

    #[test]
    fn dialog_id_is_journeymen() {
        assert_eq!(DialogJourneymenParameter::default().get_id(), DialogId::JOURNEYMEN);
    }

    #[test]
    fn stores_nr_of_slots() {
        let p = DialogJourneymenParameter { nr_of_slots: 3, ..Default::default() };
        assert_eq!(p.get_nr_of_slots(), 3);
    }

    #[test]
    fn team_id_stored() {
        let p = DialogJourneymenParameter { team_id: Some("home".into()), ..Default::default() };
        assert_eq!(p.get_team_id(), Some("home"));
    }

    #[test]
    fn transform_preserves_id() {
        let t = DialogJourneymenParameter::default().transform();
        assert_eq!(t.get_id(), DialogId::JOURNEYMEN);
    }
}
