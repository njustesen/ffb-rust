/// 1:1 translation of com.fumbbl.ffb.skill.bb2025.special::WisdomOfTheWhiteDwarf.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct WisdomOfTheWhiteDwarf {
    pub base: Skill,
}

impl WisdomOfTheWhiteDwarf {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Wisdom of the White Dwarf", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }

    /// Java `enhancementSourceName()` override — returns a fixed description instead of
    /// the base class's default (which returns the skill's own name). This is an inherent
    /// method so it shadows `Skill::enhancement_source_name` via method resolution (Rust has
    /// no virtual dispatch through `Deref`).
    pub fn enhancement_source_name(&self) -> &str {
        "Granted by Wisdom of the White Dwarf"
    }
}

impl Default for WisdomOfTheWhiteDwarf {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for WisdomOfTheWhiteDwarf {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(WisdomOfTheWhiteDwarf::new().get_name(), "Wisdom of the White Dwarf");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(WisdomOfTheWhiteDwarf::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn enhancement_source_name_is_overridden() {
        // Java WisdomOfTheWhiteDwarf.enhancementSourceName() returns a fixed string,
        // not the base Skill.enhancementSourceName() (which returns the skill's name).
        // Before the fix this would have deref'd to Skill's impl and returned
        // "Wisdom of the White Dwarf" instead.
        assert_eq!(
            WisdomOfTheWhiteDwarf::new().enhancement_source_name(),
            "Granted by Wisdom of the White Dwarf"
        );
    }
}
