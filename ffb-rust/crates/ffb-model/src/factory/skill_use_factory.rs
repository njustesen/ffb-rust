use crate::model::SkillUse;

/// 1:1 translation of com.fumbbl.ffb.factory.SkillUseFactory.
pub struct SkillUseFactory;

impl Default for SkillUseFactory {
    fn default() -> Self { SkillUseFactory }
}

impl SkillUseFactory {
    pub fn for_name(&self, name: &str) -> Option<SkillUse> {
        SkillUse::for_name(name)
    }

    pub fn initialize(&mut self) {}
}
