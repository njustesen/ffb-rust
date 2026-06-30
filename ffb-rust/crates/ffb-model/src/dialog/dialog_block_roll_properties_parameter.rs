use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::enums::ReRollProperty;
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogBlockRollPropertiesParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogBlockRollPropertiesParameter {
    pub choosing_team_id: Option<String>,
    pub nr_of_dice: i32,
    pub block_roll: Vec<i32>,
    pub re_roll_properties: Vec<ReRollProperty>,
    /// Maps re-rolled action name to source name.
    pub rr_action_to_source: HashMap<String, String>,
}

impl DialogBlockRollPropertiesParameter {
    pub fn get_choosing_team_id(&self) -> Option<&str> { self.choosing_team_id.as_deref() }
    pub fn get_nr_of_dice(&self) -> i32 { self.nr_of_dice }
    pub fn get_block_roll(&self) -> &[i32] { &self.block_roll }
    pub fn get_re_roll_properties(&self) -> &[ReRollProperty] { &self.re_roll_properties }
}

impl IDialogParameter for DialogBlockRollPropertiesParameter {
    fn get_id(&self) -> DialogId { DialogId::BLOCK_ROLL_PROPERTIES }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
