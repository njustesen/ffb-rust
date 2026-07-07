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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_skill_use() {
        assert_eq!(DialogSkillUseParameter::default().get_id(), DialogId::SKILL_USE);
    }
    #[test]
    fn stores_player_id_and_minimum_roll() {
        let p = DialogSkillUseParameter { player_id: Some("p1".into()), minimum_roll: 4, ..Default::default() };
        assert_eq!(p.get_player_id(), Some("p1"));
        assert_eq!(p.get_minimum_roll(), 4);
    }
    #[test]
    fn default_is_sensible() {
        let p = DialogSkillUseParameter::default();
        assert!(p.get_player_id().is_none());
        assert!(p.get_skill().is_none());
        assert!(p.get_modifying_skill().is_none());
        assert_eq!(p.get_minimum_roll(), 0);
        assert!(!p.is_show_never_use());
        assert!(p.get_skill_use().is_none());
        assert!(p.get_menu_property().is_none());
    }
    #[test]
    fn stores_skill_and_show_never_use() {
        let p = DialogSkillUseParameter {
            skill: Some(SkillId::Block),
            modifying_skill: Some(SkillId::Dodge),
            show_never_use: true,
            ..Default::default()
        };
        assert_eq!(p.get_skill(), Some(SkillId::Block));
        assert_eq!(p.get_modifying_skill(), Some(SkillId::Dodge));
        assert!(p.is_show_never_use());
    }
    #[test]
    fn optional_strings_none_when_unset() {
        let p = DialogSkillUseParameter { default_value_key: None, skill_use: None, menu_property: None, ..Default::default() };
        assert!(p.get_default_value_key().is_none());
        assert!(p.get_skill_use().is_none());
        assert!(p.get_menu_property().is_none());
    }
}
