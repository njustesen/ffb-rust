/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::ASneakyPair.
// NOTE: Java postConstruct registers a StaticInjuryModifierAttacker(+1, foul/stab when a partner marks the
// defender) via registerModifier(). There is no per-skill dynamic injury-modifier registration mechanism in the
// Rust codebase (the mechanics-crate modifier factories only cover a fixed global pool: Bomb/Fireball/Lightning),
// so this behavior is not yet wired up. Left as a gap pending that infrastructure.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct ASneakyPair {
    pub base: Skill,
}

impl ASneakyPair {
    pub fn new() -> Self {
        let base = Skill::new("A Sneaky Pair", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for ASneakyPair {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for ASneakyPair {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    #[test]
    fn name_is_a_sneaky_pair() {
        assert_eq!(ASneakyPair::new().get_name(), "A Sneaky Pair");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(ASneakyPair::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn skill_properties_are_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()); the live Rust property table
        // is SkillId::ASneakyPair.properties(), which always returns a valid slice.
        assert!(SkillId::ASneakyPair.properties().iter().all(|p| !p.is_empty()));
    }
}
