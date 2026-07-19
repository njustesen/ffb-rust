/// 1:1 translation of com.fumbbl.ffb.skill.mixed::SecretWeapon.
use crate::model::skill::skill::Skill;
use crate::model::skill::skill_value_evaluator::SkillValueEvaluator;
use crate::enums::SkillCategory;

pub struct SecretWeapon {
    pub base: Skill,
}

impl SecretWeapon {
    pub fn new() -> Self {
        let base = Skill::new("Secret Weapon", SkillCategory::Trait);
        Self { base }
    }

    /// Java `evaluator()` override.
    pub fn evaluator(&self) -> SkillValueEvaluator {
        SkillValueEvaluator::Roll
    }
}

impl Default for SecretWeapon {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for SecretWeapon {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(SecretWeapon::new().get_name(), "Secret Weapon"); }
    #[test]
    fn category_is_correct() { assert_eq!(SecretWeapon::new().get_category(), SkillCategory::Trait); }
    #[test]
    fn evaluator_is_roll() {
        assert_eq!(SecretWeapon::new().evaluator(), SkillValueEvaluator::Roll);
    }
}
