/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::Titchy.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Titchy {
    pub base: Skill,
}

impl Titchy {
    pub fn new() -> Self {
        let base = Skill::new("Titchy", SkillCategory::Extraordinary);
        Self { base }
    }
}

impl Default for Titchy {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Titchy {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    #[test]
    fn name_is_titchy() {
        assert_eq!(Titchy::new().get_name(), "Titchy");
    }

    #[test]
    fn category_is_extraordinary() {
        assert_eq!(Titchy::new().get_category(), SkillCategory::Extraordinary);
    }

    #[test]
    fn has_no_tacklezone_for_dodging_property() {
        assert!(SkillId::Titchy.properties().contains(&"hasNoTacklezoneForDodging"));
    }
}
