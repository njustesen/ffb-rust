/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::Timmmber.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Timmmber {
    pub base: Skill,
}

impl Timmmber {
    pub fn new() -> Self {
        let base = Skill::new("Timmm-ber!", SkillCategory::Extraordinary);
        Self { base }
    }
}

impl Default for Timmmber {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Timmmber {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
// Mirrors ffb-java/ffb-server/src/test/java/com/fumbbl/ffb/server/skill tests.
mod tests {
    use super::*;
    use crate::enums::SkillId;

    #[test]
    fn name_is_timmm_ber() {
        assert_eq!(Timmmber::new().get_name(), "Timmm-ber!");
    }

    #[test]
    fn category_is_extraordinary() {
        assert_eq!(Timmmber::new().get_category(), SkillCategory::Extraordinary);
    }

    #[test]
    fn has_allow_stand_up_assists_property() {
        assert!(SkillId::Timmmber.properties().contains(&"allowStandUpAssists"));
    }
}
