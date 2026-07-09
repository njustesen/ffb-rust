/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::WeepingDagger.
use crate::model::skill::Skill;
use crate::enums::SkillCategory;

pub struct WeepingDagger {
    pub base: Skill,
}

impl WeepingDagger {
    pub fn new() -> Self {
        let base = Skill::new("Weeping Dagger", SkillCategory::Extraordinary);
        Self { base }
    }
}

impl Default for WeepingDagger {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for WeepingDagger {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(WeepingDagger::new().get_name(), "Weeping Dagger");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(WeepingDagger::new().get_category(), SkillCategory::Extraordinary);
    }
}
