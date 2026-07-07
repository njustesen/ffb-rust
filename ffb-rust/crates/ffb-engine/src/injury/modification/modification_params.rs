/// Translation of com.fumbbl.ffb.server.injury.modification.ModificationParams.
///
/// Java: holds GameState (game + dice roller), ModifiedInjuryContext (the work copy),
/// and InjuryType (the class discriminant).
///
/// Rust: GameState is split into `game: &Game` (read-only) and `rng: &mut GameRng`
/// (dice source), matching the engine's standard parameter pattern.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::injury::InjuryContext;

pub struct ModificationParams<'a> {
    /// Java: getGameState().getGame()
    pub game: &'a Game,
    /// Java: getGameState().getDiceRoller()
    pub rng: &'a mut GameRng,
    /// Java: getNewContext() — working copy of the InjuryContext that the modification operates on.
    pub new_context: InjuryContext,
    /// Java: getInjuryType().getClass().getSimpleName() — used by isValidType().
    pub injury_type_name: &'static str,
}

impl<'a> ModificationParams<'a> {
    pub fn new(
        game: &'a Game,
        rng: &'a mut GameRng,
        new_context: InjuryContext,
        injury_type_name: &'static str,
    ) -> Self {
        Self { game, rng, new_context, injury_type_name }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{ApothecaryMode, Rules};
    use ffb_model::util::rng::GameRng;
    use crate::step::framework::test_team;

    #[test]
    fn modification_params_new_stores_fields() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut rng = GameRng::new(1);
        let ctx = InjuryContext::new(ApothecaryMode::Defender);
        let params = ModificationParams::new(&game, &mut rng, ctx, "Block");
        assert_eq!(params.injury_type_name, "Block");
        assert!(!params.new_context.armor_broken);
    }

    #[test]
    fn different_injury_type_name_stored() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut rng = GameRng::new(0);
        let ctx = InjuryContext::new(ApothecaryMode::Attacker);
        let params = ModificationParams::new(&game, &mut rng, ctx, "Stab");
        assert_eq!(params.injury_type_name, "Stab");
    }

    #[test]
    fn new_context_apo_mode_stored() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut rng = GameRng::new(0);
        let ctx = InjuryContext::new(ApothecaryMode::Attacker);
        let params = ModificationParams::new(&game, &mut rng, ctx, "Block");
        assert_eq!(params.new_context.apothecary_mode, ApothecaryMode::Attacker);
    }

    #[test]
    fn injury_type_name_is_accessible() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut rng = GameRng::new(0);
        let ctx = InjuryContext::new(ApothecaryMode::Defender);
        let params = ModificationParams::new(&game, &mut rng, ctx, "Crowd");
        assert!(!params.injury_type_name.is_empty());
        assert_eq!(params.injury_type_name, "Crowd");
    }
    #[test]
    fn new_context_starts_with_no_injury() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let game = ffb_model::model::game::Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut rng = ffb_model::util::rng::GameRng::new(0);
        let ctx = crate::injury::InjuryContext::new(ffb_model::enums::ApothecaryMode::Defender);
        let params = ModificationParams::new(&game, &mut rng, ctx, "Test");
        assert!(params.new_context.injury.is_none());
    }
}
