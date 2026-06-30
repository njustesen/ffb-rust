use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::enums::ReRollSource;
use crate::enums::SkillId;
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogReRollForTargetsParameter.
/// Note: ReRolledAction serialized as String name (stub not yet translated).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogReRollForTargetsParameter {
    pub player_id: Option<String>,
    pub target_ids: Vec<String>,
    pub minimum_rolls: HashMap<String, i32>,
    /// ReRolledAction serialized by name.
    pub re_rolled_action: Option<String>,
    pub re_roll_available_against: Vec<String>,
    pub pro_re_roll_available: bool,
    pub team_re_roll_available: bool,
    pub consummate_available: bool,
    pub re_roll_skill: Option<SkillId>,
    pub single_use_re_roll_source: Option<ReRollSource>,
}

impl DialogReRollForTargetsParameter {
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_target_ids(&self) -> &[String] { &self.target_ids }
    pub fn get_minimum_rolls(&self) -> &HashMap<String, i32> { &self.minimum_rolls }
    pub fn get_re_rolled_action(&self) -> Option<&str> { self.re_rolled_action.as_deref() }
    pub fn is_pro_re_roll_available(&self) -> bool { self.pro_re_roll_available }
    pub fn is_team_re_roll_available(&self) -> bool { self.team_re_roll_available }
    pub fn is_consummate_available(&self) -> bool { self.consummate_available }
}

impl IDialogParameter for DialogReRollForTargetsParameter {
    fn get_id(&self) -> DialogId { DialogId::RE_ROLL_FOR_TARGETS }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
