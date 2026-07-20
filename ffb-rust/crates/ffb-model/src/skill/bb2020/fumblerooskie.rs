/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::Fumblerooskie.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Fumblerooskie {
    pub base: Skill,
}

impl Fumblerooskie {
    pub fn new() -> Self {
        let base = Skill::new("Fumblerooskie", SkillCategory::Passing);
        Self { base }
    }
}

impl Default for Fumblerooskie {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Fumblerooskie {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    #[test]
    fn name_is_fumblerooskie() {
        assert_eq!(Fumblerooskie::new().get_name(), "Fumblerooskie");
    }

    #[test]
    fn category_is_passing() {
        assert_eq!(Fumblerooskie::new().get_category(), SkillCategory::Passing);
    }

    #[test]
    fn has_can_drop_ball_property() {
        assert!(SkillId::Fumblerooskie.properties().contains(&"canDropBall"));
    }
}
