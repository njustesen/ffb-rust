use ffb_model::enums::ApothecaryMode;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use crate::injury::injuryType::injury_type_ball_and_chain::InjuryTypeBallAndChain;
use crate::step::framework::{DeferredCommand, DeferredCommandId, StepParameter};
use crate::step::util_server_injury::{drop_player, handle_injury};

/// Drops a player from play: runs the full injury sequence and publishes the resulting parameters.
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.command.DropPlayerCommand`.
pub struct DropPlayerCommand {
    pub player_id: String,
    pub apothecary_mode: ApothecaryMode,
    pub eligible_for_safe_pair_of_hands: bool,
}

impl DropPlayerCommand {
    pub fn new(player_id: String, apothecary_mode: ApothecaryMode, eligible_for_safe_pair_of_hands: bool) -> Self {
        Self { player_id, apothecary_mode, eligible_for_safe_pair_of_hands }
    }
}

impl DeferredCommand for DropPlayerCommand {
    fn id(&self) -> DeferredCommandId { DeferredCommandId::DropPlayer }

    fn execute(&self, game: &mut Game, rng: &mut GameRng) -> Vec<StepParameter> {
        // Java UtilServerInjury.dropPlayer:339-342: a player with placedProneCausesInjuryRoll
        // (Ball & Chain) is NOT placed prone — the chain injures it: handleInjury(new
        // InjuryTypeBallAndChain(), ...) → publish INJURY_RESULT. This deferred command executes in
        // StepSteadyFooting.fail (a B&C block-defender's fall is deferred here), which is why the
        // chain injury was missing when the drop happened via this path (goblin seed 2 i=16: a Troll
        // blitzes the away Fanatic, Skull → the fanatic's deferred drop must still roll 2d6 here).
        let placed_prone_causes_injury = game.player(&self.player_id)
            .map(|p| p.has_skill_property(NamedProperties::PLACED_PRONE_CAUSES_INJURY_ROLL))
            .unwrap_or(false);
        if placed_prone_causes_injury {
            let coord = game.field_model.player_coordinate(&self.player_id)
                .unwrap_or(FieldCoordinate::new(0, 0));
            let mut it = InjuryTypeBallAndChain::new();
            let res = handle_injury(
                game, rng, &mut it, None, &self.player_id, coord, None, None, self.apothecary_mode,
            );
            vec![StepParameter::InjuryResult(Box::new(res))]
        } else {
            // Java: UtilServerInjury.dropPlayer(step, player, apothecaryMode, eligibleForSafePairOfHands)
            drop_player(game, &self.player_id, self.eligible_for_safe_pair_of_hands)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::{Rules, ApothecaryMode};

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn execute_returns_empty_stub() {
        let mut game = make_game();
        let cmd = DropPlayerCommand::new("p1".into(), ApothecaryMode::Defender, true);
        let params = cmd.execute(&mut game, &mut ffb_model::util::rng::GameRng::new(0));
        assert!(params.is_empty());
    }

    #[test]
    fn stores_player_id_and_apothecary_mode() {
        let cmd = DropPlayerCommand::new("player42".into(), ApothecaryMode::Attacker, true);
        assert_eq!(cmd.player_id, "player42");
        assert_eq!(cmd.apothecary_mode, ApothecaryMode::Attacker);
        assert!(cmd.eligible_for_safe_pair_of_hands);
    }

    #[test]
    fn execute_with_sph_false_still_returns_empty_stub() {
        let mut game = make_game();
        let cmd = DropPlayerCommand::new("p2".into(), ApothecaryMode::Attacker, false);
        let params = cmd.execute(&mut game, &mut ffb_model::util::rng::GameRng::new(0));
        assert!(params.is_empty());
    }
}
