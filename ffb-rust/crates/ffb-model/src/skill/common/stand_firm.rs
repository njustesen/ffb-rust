/// 1:1 translation of com.fumbbl.ffb.skill.common::StandFirm.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;
use crate::model::property::{NamedProperties, NamedProperty};

pub struct StandFirm {
    pub base: Skill,
}

impl StandFirm {
    pub fn new() -> Self {
        let mut base = Skill::new("Stand Firm", SkillCategory::Strength);
        // Java postConstruct(): registerProperty(NamedProperties.canRefuseToBePushed);
        base.register_property(Box::new(NamedProperty::new(NamedProperties::CAN_REFUSE_TO_BE_PUSHED)));
        Self { base }
    }
}

impl Default for StandFirm {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for StandFirm {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_stand_firm() {
        assert_eq!(StandFirm::new().get_name(), "Stand Firm");
    }

    #[test]
    fn category_is_strength() {
        assert_eq!(StandFirm::new().get_category(), SkillCategory::Strength);
    }

    #[test]
    fn has_can_refuse_to_be_pushed_property() {
        assert!(crate::enums::SkillId::StandFirm.properties().contains(&"canRefuseToBePushed"));
    }
}
