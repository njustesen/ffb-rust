/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::TakeRoot.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct TakeRoot {
    pub base: Skill,
}

impl TakeRoot {
    pub fn new() -> Self {
        let base = Skill::new("Take Root", SkillCategory::Extraordinary);
        Self { base }
    }
}

impl Default for TakeRoot {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for TakeRoot {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    #[test]
    fn name_is_take_root() {
        assert_eq!(TakeRoot::new().get_name(), "Take Root");
    }

    #[test]
    fn category_is_extraordinary() {
        assert_eq!(TakeRoot::new().get_category(), SkillCategory::Extraordinary);
    }

    #[test]
    fn has_becomes_immovable_property() {
        assert!(SkillId::TakeRoot.properties().contains(&"becomesImmovable"));
    }
}
