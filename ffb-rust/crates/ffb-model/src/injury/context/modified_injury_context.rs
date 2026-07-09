use crate::injury::context::injury_modification::InjuryModification;

/// 1:1 translation of `com.fumbbl.ffb.injury.context.ModifiedInjuryContext`.
///
/// Extends InjuryContext with skill/apothecary modification tracking.
/// Used when a skill or apothecary changes the injury outcome.
#[derive(Debug, Default)]
pub struct ModifiedInjuryContext {
    /// Java: usedSkill — the skill id that caused this modification (String representation).
    pub used_skill: Option<String>,
    /// Java: skillUse — the usage type (stored as a string matching Java enum name).
    pub skill_use: Option<String>,
    /// Java: modification — which phase of injury was modified.
    pub modification: InjuryModification,
}

impl ModifiedInjuryContext {
    /// Java: `new ModifiedInjuryContext()`
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `getUsedSkill()`
    pub fn get_used_skill(&self) -> Option<&str> {
        self.used_skill.as_deref()
    }

    /// Java: `setUsedSkill(Skill)`
    pub fn set_used_skill(&mut self, skill_id: impl Into<String>) {
        self.used_skill = Some(skill_id.into());
    }

    /// Java: `getSkillUse()`
    pub fn get_skill_use(&self) -> Option<&str> {
        self.skill_use.as_deref()
    }

    /// Java: `setSkillUse(SkillUse)`
    pub fn set_skill_use(&mut self, skill_use: impl Into<String>) {
        self.skill_use = Some(skill_use.into());
    }

    /// Java: `getModification()`
    pub fn get_modification(&self) -> InjuryModification {
        self.modification
    }

    /// Java: `setModification(InjuryModification)`
    pub fn set_modification(&mut self, modification: InjuryModification) {
        self.modification = modification;
    }

    /// Java: `getModifiedInjuryContext()` returns null for ModifiedInjuryContext — no nesting.
    pub fn get_modified_injury_context(&self) -> Option<&ModifiedInjuryContext> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_has_none_modification() {
        let ctx = ModifiedInjuryContext::new();
        assert_eq!(ctx.get_modification(), InjuryModification::NONE);
        assert!(ctx.get_used_skill().is_none());
    }

    #[test]
    fn set_and_get_modification() {
        let mut ctx = ModifiedInjuryContext::new();
        ctx.set_modification(InjuryModification::ARMOUR);
        assert_eq!(ctx.get_modification(), InjuryModification::ARMOUR);
    }

    #[test]
    fn set_used_skill() {
        let mut ctx = ModifiedInjuryContext::new();
        ctx.set_used_skill("mightyBlow");
        assert_eq!(ctx.get_used_skill(), Some("mightyBlow"));
    }

    #[test]
    fn no_nested_modified_context() {
        let ctx = ModifiedInjuryContext::new();
        assert!(ctx.get_modified_injury_context().is_none());
    }
}
