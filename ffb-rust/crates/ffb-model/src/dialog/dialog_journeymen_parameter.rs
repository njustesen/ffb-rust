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
