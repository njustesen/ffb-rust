/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::FanFavourite.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct FanFavourite {
    pub base: Skill,
}

impl FanFavourite {
    pub fn new() -> Self {
        let base = Skill::new("Fan Favourite", SkillCategory::Extraordinary);
        Self { base }
    }
}

impl Default for FanFavourite {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for FanFavourite {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    #[test]
    fn name_is_fan_favourite() {
        assert_eq!(FanFavourite::new().get_name(), "Fan Favourite");
    }

    #[test]
    fn category_is_extraordinary() {
        assert_eq!(FanFavourite::new().get_category(), SkillCategory::Extraordinary);
    }

    #[test]
    fn skill_properties_are_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()) — the live Rust property
        // table always yields a slice; assert every entry is a non-empty key.
        assert!(SkillId::FanFavourite.properties().iter().all(|p| !p.is_empty()));
    }
}
