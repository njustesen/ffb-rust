/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::Disposable.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Disposable {
    pub base: Skill,
}

impl Disposable {
    pub fn new() -> Self {
        let base = Skill::new("Disposable", SkillCategory::Extraordinary);
        Self { base }
    }
}

impl Default for Disposable {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Disposable {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    #[test]
    fn name_is_disposable() {
        assert_eq!(Disposable::new().get_name(), "Disposable");
    }

    #[test]
    fn category_is_extraordinary() {
        assert_eq!(Disposable::new().get_category(), SkillCategory::Extraordinary);
    }

    #[test]
    fn skill_properties_are_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()) — the live Rust property
        // table always yields a slice; assert every entry is a non-empty key.
        assert!(SkillId::Disposable.properties().iter().all(|p| !p.is_empty()));
    }
}
