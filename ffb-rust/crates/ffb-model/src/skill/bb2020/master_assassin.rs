/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::MasterAssassin.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct MasterAssassin {
    pub base: Skill,
}

impl MasterAssassin {
    pub fn new() -> Self {
        let base = Skill::new("Master Assassin", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for MasterAssassin {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for MasterAssassin {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(MasterAssassin::new().get_name(), "Master Assassin");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(MasterAssassin::new().get_category(), SkillCategory::Trait);
    }
}
