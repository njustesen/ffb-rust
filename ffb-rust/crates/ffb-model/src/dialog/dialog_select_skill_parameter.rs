use serde::{Deserialize, Serialize};
use crate::enums::SkillId;
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogSelectSkillParameter.
/// Note: SkillChoiceMode serialized as String (stub not yet translated).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogSelectSkillParameter {
    pub skills: Vec<SkillId>,
    pub player_id: Option<String>,
    pub skill_choice_mode: Option<String>,
}

impl DialogSelectSkillParameter {
    pub fn get_skills(&self) -> &[SkillId] { &self.skills }
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_skill_choice_mode(&self) -> Option<&str> { self.skill_choice_mode.as_deref() }
    pub fn add_skill(&mut self, skill: SkillId) { self.skills.push(skill); }
}

impl IDialogParameter for DialogSelectSkillParameter {
    fn get_id(&self) -> DialogId { DialogId::SELECT_SKILL }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_skill_and_serde() {
        let mut p = DialogSelectSkillParameter {
            player_id: Some("player1".into()),
            ..Default::default()
        };
        p.add_skill(SkillId::Dodge);
        p.add_skill(SkillId::Block);
        let json = serde_json::to_string(&p).unwrap();
        let back: DialogSelectSkillParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_skills(), &[SkillId::Dodge, SkillId::Block]);
        assert_eq!(back.get_player_id(), Some("player1"));
    }

    #[test]
    fn get_id_is_select_skill() {
        assert_eq!(DialogSelectSkillParameter::default().get_id(), DialogId::SELECT_SKILL);
    }
}
