/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::SneakyGit.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct SneakyGit {
    pub base: Skill,
}

impl SneakyGit {
    pub fn new() -> Self {
        let base = Skill::new("Sneaky Git", SkillCategory::Agility);
        Self { base }
    }
}

impl Default for SneakyGit {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for SneakyGit {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(SneakyGit::new().get_name(), "Sneaky Git");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(SneakyGit::new().get_category(), SkillCategory::Agility);
    }
}
