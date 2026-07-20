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

    // Mirrors ffb-java SecretWeaponSkillTest (written against the bb2016 class,
    // where the category is EXTRAORDINARY); category adapted to the mixed
    // edition's constructor (TRAIT). Property assertion verified against the
    // mixed edition's postConstruct, which registers only getsSentOffAtEndOfDrive.
    // The Java `is_bb2016_edition` test is skipped (edition annotations are
    // covered elsewhere).

    #[test]
    fn name_is_secret_weapon() {
        assert_eq!(SecretWeapon::new().get_name(), "Secret Weapon");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(SecretWeapon::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_gets_sent_off_at_end_of_drive_property() {
        assert!(crate::enums::SkillId::SecretWeapon.properties().contains(&"getsSentOffAtEndOfDrive"));
    }

    // Additional Rust-side logic test beyond the Java test class.
    #[test]
    fn evaluator_is_roll() {
        assert_eq!(SecretWeapon::new().evaluator(), SkillValueEvaluator::Roll);
    }
}
