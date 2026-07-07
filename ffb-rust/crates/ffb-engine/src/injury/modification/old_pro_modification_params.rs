/// Translation of com.fumbbl.ffb.server.injury.modification.OldProModificationParams.
///
/// Extends ModificationParams with OldPro-specific tracking fields used during
/// the single-die replacement logic.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::injury::InjuryContext;
use crate::injury::modification::ModificationParams;

pub struct OldProModificationParams<'a> {
    pub base: ModificationParams<'a>,
    /// Java: selfInflicted — the damage is self-inflicted or hits a team-mate.
    pub self_inflicted: bool,
    /// Java: spottedFoul — both armor dice were equal during a foul.
    pub spotted_foul: bool,
    /// Java: oldValue — the original value of the die that will be replaced.
    pub old_value: i32,
    /// Java: replaceIndex — which armor die (0 or 1) to replace.
    pub replace_index: usize,
}

impl<'a> OldProModificationParams<'a> {
    pub fn new(game: &'a Game, rng: &'a mut GameRng, new_context: InjuryContext, injury_type_name: &'static str) -> Self {
        Self {
            base: ModificationParams::new(game, rng, new_context, injury_type_name),
            self_inflicted: false,
            spotted_foul: false,
            old_value: 0,
            replace_index: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{ApothecaryMode, Rules};
    use ffb_model::util::rng::GameRng;
    use crate::step::framework::test_team;

    #[test]
    fn old_pro_params_defaults() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut rng = GameRng::new(1);
        let ctx = InjuryContext::new(ApothecaryMode::Defender);
        let params = OldProModificationParams::new(&game, &mut rng, ctx, "Block");
        assert!(!params.self_inflicted);
        assert!(!params.spotted_foul);
        assert_eq!(params.old_value, 0);
        assert_eq!(params.replace_index, 0);
    }

    #[test]
    fn old_pro_spotted_foul_detection() {
        // spotted foul: both armor dice equal + foul injury type
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.armor_roll = Some([3, 3]); // equal dice
        let mut params = OldProModificationParams::new(&game, &mut rng, ctx, "Foul");
        // Spotted foul: both dice equal AND injury type is foul
        let is_foul = params.base.injury_type_name.contains("Foul");
        let both_equal = params.base.new_context.armor_roll
            .map(|r| r[0] == r[1]).unwrap_or(false);
        params.spotted_foul = is_foul && both_equal;
        assert!(params.spotted_foul);
    }

    #[test]
    fn self_inflicted_can_be_set() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut rng = GameRng::new(0);
        let ctx = InjuryContext::new(ApothecaryMode::Defender);
        let mut params = OldProModificationParams::new(&game, &mut rng, ctx, "Block");
        assert!(!params.self_inflicted);
        params.self_inflicted = true;
        assert!(params.self_inflicted);
    }

    #[test]
    fn replace_index_can_be_set() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut rng = GameRng::new(0);
        let ctx = InjuryContext::new(ApothecaryMode::Defender);
        let mut params = OldProModificationParams::new(&game, &mut rng, ctx, "Block");
        assert_eq!(params.replace_index, 0);
        params.replace_index = 1;
        assert_eq!(params.replace_index, 1);
    }
}
