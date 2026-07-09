/// 1:1 translation of com.fumbbl.ffb.skill.mixed::AlwaysHungry.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct AlwaysHungry {
    pub base: Skill,
}

impl AlwaysHungry {
    pub fn new() -> Self {
        let base = Skill::new("Always Hungry", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for AlwaysHungry {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for AlwaysHungry {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(AlwaysHungry::new().get_name(), "Always Hungry"); }
    #[test]
    fn category_is_correct() { assert_eq!(AlwaysHungry::new().get_category(), SkillCategory::Trait); }
}
