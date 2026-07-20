/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::RightStuff.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct RightStuff {
    pub base: Skill,
}

impl RightStuff {
    pub fn new() -> Self {
        let base = Skill::new("Right Stuff", SkillCategory::Extraordinary);
        Self { base }
    }
}

impl Default for RightStuff {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for RightStuff {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    #[test]
    fn name_is_right_stuff() {
        assert_eq!(RightStuff::new().get_name(), "Right Stuff");
    }

    #[test]
    fn category_is_extraordinary() {
        assert_eq!(RightStuff::new().get_category(), SkillCategory::Extraordinary);
    }

    #[test]
    fn has_can_be_thrown_property() {
        assert!(SkillId::RightStuff.properties().contains(&"canBeThrown"));
    }
}
