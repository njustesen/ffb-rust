/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::Animosity.
// NOTE: Java overrides evaluator() with an AnimosityValueEvaluator that formats display strings
// from the player's roster/position data. `AnimosityValueEvaluator`/`Skill::evaluator()` have no
// live caller anywhere in this workspace (dead UI-display infra), so it is not implemented here.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Animosity {
    pub base: Skill,
}

impl Animosity {
    pub fn new() -> Self {
        let base = Skill::new("Animosity", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for Animosity {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Animosity {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Animosity::new().get_name(), "Animosity");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(Animosity::new().get_category(), SkillCategory::Trait);
    }
}
