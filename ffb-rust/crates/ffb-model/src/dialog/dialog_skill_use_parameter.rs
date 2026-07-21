use serde::{Deserialize, Serialize};
use crate::enums::SkillId;
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogSkillUseParameter.
/// Note: SkillUse/CommonProperty serialized as String (stubs not yet translated).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogSkillUseParameter {
    pub player_id: Option<String>,
    pub default_value_key: Option<String>,
    pub skill: Option<SkillId>,
    pub modifying_skill: Option<SkillId>,
    pub minimum_roll: i32,
    pub show_never_use: bool,
    /// SkillUse serialized by name.
    pub skill_use: Option<String>,
    /// CommonProperty serialized by key.
    pub menu_property: Option<String>,
}

impl DialogSkillUseParameter {
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_default_value_key(&self) -> Option<&str> { self.default_value_key.as_deref() }
    pub fn get_skill(&self) -> Option<SkillId> { self.skill }
    pub fn get_modifying_skill(&self) -> Option<SkillId> { self.modifying_skill }
    pub fn get_minimum_roll(&self) -> i32 { self.minimum_roll }
    pub fn is_show_never_use(&self) -> bool { self.show_never_use }
    pub fn get_skill_use(&self) -> Option<&str> { self.skill_use.as_deref() }
    pub fn get_menu_property(&self) -> Option<&str> { self.menu_property.as_deref() }
}

impl IDialogParameter for DialogSkillUseParameter {
    fn get_id(&self) -> DialogId { DialogId::SKILL_USE }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
