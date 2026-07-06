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
    use ffb_model::enums::{ApothecaryMode, Rules};
    use ffb_model::model::game::Game;
    use ffb_model::util::rng::GameRng;
    use crate::injury::InjuryContext;
    use crate::injury::modification::ModificationParams;
    use crate::step::framework::test_team;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

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

    #[test]
    fn skill_use_is_reroll_armour() {
        assert_eq!(MasterAssassinModification::new().skill_use(), SkillUse::RE_ROLL_ARMOUR);
    }

    #[test]
    fn try_armour_false_when_broken() {
        let game = make_game();
        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.armor_broken = true;
        let params = ModificationParams::new(&game, &mut rng, ctx, "Stab");
        assert!(!MasterAssassinModification::new().try_armour_roll_modification(&params));
    }

    #[test]
    fn try_armour_true_when_not_broken() {
        let game = make_game();
        let mut rng = GameRng::new(1);
        let ctx = InjuryContext::new(ApothecaryMode::Defender);
        let params = ModificationParams::new(&game, &mut rng, ctx, "Stab");
        assert!(MasterAssassinModification::new().try_armour_roll_modification(&params));
    }
}
