/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::BallAndChain.
// NOTE: Java also calls registerConflictingProperty(...) for 6 properties (forceSecondBlock,
// canAttemptToTackleDodgingPlayer, canLeap, canBlockTwoAtOnce, canMoveDuringKickOffScatter,
// canFollowPlayerLeavingTacklezones), consumed via Skill.canBeAssignedTo()/conflictsWithAnySkill().
// `Skill::conflicting_properties`/`register_conflicting_property` exist on the Rust Skill struct
// but have no live caller anywhere in this workspace (dead skill-assignment-validation infra), so
// this is not modeled here; the granted properties (registerProperty/CancelSkillProperty) are all
// present in skill_id.rs's properties() table, which IS the live has_skill_property() mechanism.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct BallAndChain {
    pub base: Skill,
}

impl BallAndChain {
    pub fn new() -> Self {
        let base = Skill::new("Ball and Chain", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for BallAndChain {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for BallAndChain {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    // bb2020/BallAndChain is SkillCategory.TRAIT (the bb2016 test's EXTRAORDINARY is bb2016-only).

    #[test]
    fn name_is_ball_and_chain() {
        assert_eq!(BallAndChain::new().get_name(), "Ball and Chain");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(BallAndChain::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_moves_randomly_property() {
        // Java bb2016 test asserts forceFullMovement, which only the bb2016 edition
        // registers; bb2020/BallAndChain.postConstruct registers movesRandomly instead.
        assert!(SkillId::BallAndChain.properties().contains(&"movesRandomly"));
    }
}
