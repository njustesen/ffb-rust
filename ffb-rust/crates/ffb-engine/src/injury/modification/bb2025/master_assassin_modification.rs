/// Translation of com.fumbbl.ffb.server.injury.modification.bb2025.MasterAssassinModification.
///
/// Extends RerollArmourModification with valid types = {Stab}.
use ffb_model::model::SkillUse;
use crate::injury::modification::{InjuryContextModification, ModificationParams};
use crate::injury::modification::bb2025::reroll_armour_modification::RerollArmourModification;

pub struct MasterAssassinModification {
    inner: RerollArmourModification,
}

impl MasterAssassinModification {
    pub fn new() -> Self { Self { inner: RerollArmourModification::with_types(&["Stab"]) } }
}

impl Default for MasterAssassinModification {
    fn default() -> Self { Self::new() }
}

impl InjuryContextModification for MasterAssassinModification {
    fn skill_use(&self) -> SkillUse { self.inner.skill_use() }
    fn valid_types(&self) -> &'static [&'static str] { &["Stab"] }
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
    fn valid_type_is_stab() {
        let m = MasterAssassinModification::new();
        assert!(m.is_valid_type("Stab"));
        assert!(!m.is_valid_type("Block"));
    }

    #[test]
    fn valid_types_slice_contains_only_stab() {
        let m = MasterAssassinModification::new();
        let types = m.valid_types();
        assert_eq!(types, &["Stab"]);
    }

    #[test]
    fn rejects_chainsaw_and_projectile_vomit() {
        let m = MasterAssassinModification::new();
        assert!(!m.is_valid_type("Chainsaw"));
        assert!(!m.is_valid_type("ProjectileVomit"));
    }
}
