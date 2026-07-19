/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::SideStep.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct SideStep {
    pub base: Skill,
}

impl SideStep {
    pub fn new() -> Self {
        let base = Skill::new("Side Step", SkillCategory::Agility);
        Self { base }
    }

    /// Java `getSkillUseDescription()`.
    pub fn get_skill_use_description(&self) -> Option<Vec<String>> {
        Some(vec!["Using SideStep will allow you to chose the square you are pushed to.".to_string()])
    }
}

impl Default for SideStep {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for SideStep {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(SideStep::new().get_name(), "Side Step");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(SideStep::new().get_category(), SkillCategory::Agility);
    }
}
