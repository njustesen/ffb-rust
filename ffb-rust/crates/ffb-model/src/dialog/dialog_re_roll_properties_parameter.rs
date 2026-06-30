use serde::{Deserialize, Serialize};
use crate::enums::ReRollProperty;
use crate::enums::SkillId;
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogReRollPropertiesParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogReRollPropertiesParameter {
    pub player_id: Option<String>,
    pub default_value_key: Option<String>,
    /// ReRolledAction serialized by name.
    pub re_rolled_action: Option<String>,
    pub minimum_roll: i32,
    pub fumble: bool,
    pub re_roll_skill: Option<SkillId>,
    pub modifying_skill: Option<SkillId>,
    pub messages: Vec<String>,
    pub re_roll_properties: Vec<ReRollProperty>,
    /// CommonProperty serialized by key.
    pub menu_property: Option<String>,
}

impl DialogReRollPropertiesParameter {
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_default_value_key(&self) -> Option<&str> { self.default_value_key.as_deref() }
    pub fn get_re_rolled_action(&self) -> Option<&str> { self.re_rolled_action.as_deref() }
    pub fn get_minimum_roll(&self) -> i32 { self.minimum_roll }
    pub fn is_fumble(&self) -> bool { self.fumble }
    pub fn get_messages(&self) -> &[String] { &self.messages }
    pub fn get_re_roll_properties(&self) -> &[ReRollProperty] { &self.re_roll_properties }
    pub fn get_menu_property(&self) -> Option<&str> { self.menu_property.as_deref() }
}

impl IDialogParameter for DialogReRollPropertiesParameter {
    fn get_id(&self) -> DialogId { DialogId::RE_ROLL_PROPERTIES }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
