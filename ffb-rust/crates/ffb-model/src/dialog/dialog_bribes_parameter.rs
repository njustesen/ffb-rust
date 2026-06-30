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
