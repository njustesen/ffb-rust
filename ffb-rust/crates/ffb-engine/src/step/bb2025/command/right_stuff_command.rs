use ffb_model::model::game::Game;
use crate::step::framework::{DeferredCommand, DeferredCommandId, StepParameter};
use ffb_model::enums::ApothecaryMode;
use crate::step::util_server_injury::drop_player_rng;

/// Handles a thrown player landing: runs injury, conditionally triggers a turnover (if carrying
/// the ball), and clears ThrownPlayerCoordinate to prevent an end-step reset.
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.command.RightStuffCommand`.
pub struct RightStuffCommand {
    pub player_id: String,
    pub has_ball: bool,
}

impl RightStuffCommand {
    pub fn new(player_id: String, has_ball: bool) -> Self {
        Self { player_id, has_ball }
    }
}

impl DeferredCommand for RightStuffCommand {
    fn id(&self) -> DeferredCommandId { DeferredCommandId::RightStuff }

    fn execute(&self, game: &mut Game, rng: &mut ffb_model::util::rng::GameRng) -> Vec<StepParameter> {
        // Java: UtilServerInjury.dropPlayer(step, thrownPlayer, ApothecaryMode.THROWN_PLAYER)
        //   (= dropPlayer(.., PRONE, THROWN_PLAYER, eligibleForSafePairOfHands=false))
        // Then: remove END_TURN from the result set; re-add it only if has_ball.
        // The mode MATTERS: dropPlayer deactivates the acting player unless the mode is
        // THROWN_PLAYER, and a swooped thrown player IS the acting player. `drop_player_no_sph`
        // assumed DEFENDER, so a swooped Doom Diver's failed landing left it inactive where Java
        // leaves it free to stand up and act (BACKLOG §M: goblin / goblin_sk bb2025 seed 16).
        let mut params = drop_player_rng(game, rng, &self.player_id, false, ApothecaryMode::ThrownPlayer);
        params.retain(|p| !matches!(p, StepParameter::EndTurn(_)));
        if self.has_ball {
            params.push(StepParameter::EndTurn(true));
        }
        params.push(StepParameter::ThrownPlayerCoordinate(None));
        params
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn with_ball_publishes_end_turn() {
        let mut game = make_game();
        let cmd = RightStuffCommand::new("p1".into(), true);
        let params = cmd.execute(&mut game, &mut ffb_model::util::rng::GameRng::new(0));
        assert!(params.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
        assert!(params.iter().any(|p| matches!(p, StepParameter::ThrownPlayerCoordinate(None))));
    }

    #[test]
    fn without_ball_no_end_turn() {
        let mut game = make_game();
        let cmd = RightStuffCommand::new("p1".into(), false);
        let params = cmd.execute(&mut game, &mut ffb_model::util::rng::GameRng::new(0));
        assert!(!params.iter().any(|p| matches!(p, StepParameter::EndTurn(_))));
        assert!(params.iter().any(|p| matches!(p, StepParameter::ThrownPlayerCoordinate(None))));
    }

    #[test]
    fn id_returns_right_stuff_variant() {
        let cmd = RightStuffCommand::new("p1".into(), false);
        assert_eq!(cmd.id(), DeferredCommandId::RightStuff);
    }

    #[test]
    fn always_clears_thrown_player_coordinate() {
        let mut game = make_game();
        // Test both has_ball=true and has_ball=false always clear coordinate
        for has_ball in [true, false] {
            let cmd = RightStuffCommand::new("p1".into(), has_ball);
            let params = cmd.execute(&mut game, &mut ffb_model::util::rng::GameRng::new(0));
            assert!(
                params.iter().any(|p| matches!(p, StepParameter::ThrownPlayerCoordinate(None))),
                "ThrownPlayerCoordinate(None) missing when has_ball={has_ball}"
            );
        }
    }
}
