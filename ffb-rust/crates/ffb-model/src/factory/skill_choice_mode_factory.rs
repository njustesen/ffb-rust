use crate::model::SkillChoiceMode;

/// 1:1 translation of com.fumbbl.ffb.factory.SkillChoiceModeFactory (if exists).
pub struct SkillChoiceModeFactory;

impl Default for SkillChoiceModeFactory {
    fn default() -> Self { SkillChoiceModeFactory }
}

impl SkillChoiceModeFactory {
    pub fn for_name(&self, name: &str) -> Option<SkillChoiceMode> {
        SkillChoiceMode::for_name(name)
    }

    pub fn initialize(&mut self) {}
}
