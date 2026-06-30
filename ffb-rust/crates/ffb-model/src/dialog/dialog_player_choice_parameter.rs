use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogPlayerChoiceParameter.
/// Note: PlayerChoiceMode serialized as String name (stub not yet translated).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogPlayerChoiceParameter {
    pub team_id: Option<String>,
    /// PlayerChoiceMode serialized by name.
    pub player_choice_mode: Option<String>,
    pub player_ids: Vec<String>,
    pub descriptions: Vec<String>,
    pub max_selects: i32,
    pub min_selects: i32,
}

impl DialogPlayerChoiceParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_player_choice_mode(&self) -> Option<&str> { self.player_choice_mode.as_deref() }
    pub fn get_player_ids(&self) -> &[String] { &self.player_ids }
    pub fn get_descriptions(&self) -> &[String] { &self.descriptions }
    pub fn get_max_selects(&self) -> i32 { self.max_selects }
    pub fn get_min_selects(&self) -> i32 { self.min_selects }
    pub fn add_player_id(&mut self, id: impl Into<String>) {
        let s = id.into();
        if !s.is_empty() { self.player_ids.push(s); }
    }
    pub fn add_description(&mut self, d: impl Into<String>) {
        let s = d.into();
        if !s.is_empty() { self.descriptions.push(s); }
    }
}

impl IDialogParameter for DialogPlayerChoiceParameter {
    fn get_id(&self) -> DialogId { DialogId::PLAYER_CHOICE }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
