/// Translation of com.fumbbl.ffb.server.injury.modification.MasterAssassinModification.
///
/// Valid for Stab. Replaces the entire injury roll with a fresh re-roll.
/// Gate: injury is not already a casualty (RE_ROLL_INJURY).
use ffb_model::model::game::Game;
use ffb_model::model::SkillUse;
use ffb_model::util::rng::GameRng;
use crate::injury::InjuryContext;
use crate::injury::modification::{InjuryContextModification, ModificationParams};

pub struct MasterAssassinModification {
    skill_id: Option<u16>,
}

impl MasterAssassinModification {
    pub fn new() -> Self { Self { skill_id: None } }
}

impl Default for MasterAssassinModification {
    fn default() -> Self { Self::new() }
}

impl InjuryContextModification for MasterAssassinModification {
    fn skill_use(&self) -> SkillUse { SkillUse::RE_ROLL_INJURY }
    fn valid_types(&self) -> &'static [&'static str] { &["Stab"] }
    fn skill_id(&self) -> Option<u16> { self.skill_id }
    fn set_skill_id(&mut self, id: u16) { self.skill_id = Some(id); }

    fn try_injury_modification(&self, _game: &Game, ctx: &InjuryContext, _injury_type_name: &str) -> bool {
        !ctx.is_casualty()
    }

    /// Java: re-roll injury dice, re-interpret, always return true.
    fn modify_injury_internal(&self, game: &Game, rng: &mut GameRng, ctx: &mut InjuryContext) -> bool {
        let d1 = rng.d6();
        let d2 = rng.d6();
        ctx.set_injury_roll([d1, d2]);
        ctx.injury = self.interpret_injury(game, ctx);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{ApothecaryMode, Rules, PS_SERIOUS_INJURY, PlayerState};
    use crate::step::framework::test_team;

    #[test]
    fn valid_type_is_stab() {
        assert!(MasterAssassinModification::new().is_valid_type("Stab"));
        assert!(!MasterAssassinModification::new().is_valid_type("Block"));
    }

    #[test]
    fn try_injury_false_when_casualty() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.injury = Some(PlayerState::new(PS_SERIOUS_INJURY));
        assert!(!MasterAssassinModification::new().try_injury_modification(&game, &ctx, "Stab"));
    }

    #[test]
    fn try_injury_true_when_not_casualty() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        use ffb_model::enums::PS_STUNNED;
        ctx.injury = Some(PlayerState::new(PS_STUNNED));
        assert!(MasterAssassinModification::new().try_injury_modification(&game, &ctx, "Stab"));
    }
}
