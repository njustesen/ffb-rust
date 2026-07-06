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
    fn valid_type_is_block() {
        let m = KrumpAndSmashModification::new();
        assert!(m.is_valid_type("Block"));
        assert!(!m.is_valid_type("Stab"));
    }

    #[test]
    fn skill_use_is_reroll_armour() {
        assert_eq!(KrumpAndSmashModification::new().skill_use(), SkillUse::RE_ROLL_ARMOUR);
    }

    #[test]
    fn try_armour_false_when_broken() {
        let game = make_game();
        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.armor_broken = true;
        let params = ModificationParams::new(&game, &mut rng, ctx, "Block");
        assert!(!KrumpAndSmashModification::new().try_armour_roll_modification(&params));
    }

    #[test]
    fn try_armour_true_when_not_broken() {
        let game = make_game();
        let mut rng = GameRng::new(1);
        let ctx = InjuryContext::new(ApothecaryMode::Defender);
        let params = ModificationParams::new(&game, &mut rng, ctx, "Block");
        assert!(KrumpAndSmashModification::new().try_armour_roll_modification(&params));
    }

    #[test]
    fn modify_armour_internal_returns_true() {
        let game = make_game();
        let mut rng = GameRng::new(1);
        let ctx = InjuryContext::new(ApothecaryMode::Defender);
        let mut params = ModificationParams::new(&game, &mut rng, ctx, "Block");
        assert!(KrumpAndSmashModification::new().modify_armour_internal(&mut params));
        let [d1, d2] = params.new_context.armor_roll.unwrap();
        assert!((1..=6).contains(&d1));
        assert!((1..=6).contains(&d2));
    }
}
