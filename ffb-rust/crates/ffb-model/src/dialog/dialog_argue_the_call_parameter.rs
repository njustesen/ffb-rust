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
