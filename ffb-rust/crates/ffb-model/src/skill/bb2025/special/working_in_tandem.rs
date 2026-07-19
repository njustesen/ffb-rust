/// 1:1 translation of com.fumbbl.ffb.skill.bb2025.special::WorkingInTandem.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct WorkingInTandem {
    pub base: Skill,
}

impl WorkingInTandem {
    pub fn new() -> Self {
        let base = Skill::new("Working in Tandem", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for WorkingInTandem {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for WorkingInTandem {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

// DEFERRED: Java WorkingInTandem overrides `evaluator()` with a custom, Player-aware
// SkillValueEvaluator (nested `Evaluator` class reading `player.temporarySkillValues()` /
// `player.getSkillValueExcludingTemporaryOnes()` to build display strings, plus the
// `VARIANT_BLOCK`/`VARIANT_PASS` constants it formats). The Rust `SkillValueEvaluator`
// (crate::model::skill::skill_value_evaluator) is a fixed Default/Modifier/Roll enum with
// no support for arbitrary per-skill/per-player evaluator logic, and `Skill` has no
// `evaluator()` hook at all — porting this needs a new player-aware evaluator trait/hook
// on `Skill`, which is out of scope for this file-level audit. The rules-engine-relevant
// behavior (reroll on marked partner / no pass modifiers to partner) is driven separately
// via NamedProperties lookups in ffb-mechanics/ffb-engine, not through this evaluator.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(WorkingInTandem::new().get_name(), "Working in Tandem");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(WorkingInTandem::new().get_category(), SkillCategory::Trait);
    }
}
