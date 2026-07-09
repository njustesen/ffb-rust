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

    #[test]
    fn name_is_correct() {
        assert_eq!(FanFavourite::new().get_name(), "Fan Favourite");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(FanFavourite::new().get_category(), SkillCategory::Extraordinary);
    }
}
