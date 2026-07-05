/// Translation of com.fumbbl.ffb.server.injury.modification.bb2025.KrumpAndSmashModification.
///
/// Extends RerollArmourModification with valid types = {Block}.
use ffb_model::model::SkillUse;
use crate::injury::modification::{InjuryContextModification, ModificationParams};
use crate::injury::modification::bb2025::reroll_armour_modification::RerollArmourModification;

pub struct KrumpAndSmashModification {
    inner: RerollArmourModification,
}

impl KrumpAndSmashModification {
    pub fn new() -> Self { Self { inner: RerollArmourModification::with_types(&["Block"]) } }
}

impl Default for KrumpAndSmashModification {
    fn default() -> Self { Self::new() }
}

impl InjuryContextModification for KrumpAndSmashModification {
    fn skill_use(&self) -> SkillUse { self.inner.skill_use() }
    fn valid_types(&self) -> &'static [&'static str] { &["Block"] }
    fn skill_id(&self) -> Option<u16> { self.inner.skill_id() }
    fn set_skill_id(&mut self, id: u16) { self.inner.set_skill_id(id); }

    fn try_armour_roll_modification(&self, params: &ModificationParams) -> bool {
        self.inner.try_armour_roll_modification(params)
    }

    fn modify_armour_internal(&self, params: &mut ModificationParams) -> bool {
        self.inner.modify_armour_internal(params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_type_is_block() {
        let m = KrumpAndSmashModification::new();
        assert!(m.is_valid_type("Block"));
        assert!(!m.is_valid_type("Stab"));
    }

    #[test]
    fn skill_use_is_reroll_armour() {
        assert_eq!(KrumpAndSmashModification::new().skill_use(), SkillUse::RE_ROLL_ARMOUR);
    }
}
