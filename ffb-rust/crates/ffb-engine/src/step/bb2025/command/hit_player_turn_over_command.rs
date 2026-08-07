use ffb_model::model::game::Game;
use crate::step::framework::{DeferredCommand, DeferredCommandId, StepParameter};

/// Publishes EndTurn(true) — triggers a turnover when a player is hit.
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.command.HitPlayerTurnOverCommand`.
pub struct HitPlayerTurnOverCommand;

impl DeferredCommand for HitPlayerTurnOverCommand {
    fn id(&self) -> DeferredCommandId { DeferredCommandId::HitPlayer }

    fn execute(&self, _game: &mut Game, _rng: &mut ffb_model::util::rng::GameRng) -> Vec<StepParameter> {
        vec![StepParameter::EndTurn(true)]
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
    fn publishes_end_turn_true() {
        let mut game = make_game();
        let cmd = HitPlayerTurnOverCommand;
        let params = cmd.execute(&mut game, &mut ffb_model::util::rng::GameRng::new(0));
        assert!(params.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn publishes_exactly_one_param() {
        let mut game = make_game();
        let params = HitPlayerTurnOverCommand.execute(&mut game, &mut ffb_model::util::rng::GameRng::new(0));
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn id_returns_hit_player_variant() {
        assert_eq!(HitPlayerTurnOverCommand.id(), DeferredCommandId::HitPlayer);
    }

    #[test]
    fn end_turn_value_is_true() {
        let mut game = make_game();
        let params = HitPlayerTurnOverCommand.execute(&mut game, &mut ffb_model::util::rng::GameRng::new(0));
        let end_turn = params.iter().find(|p| matches!(p, StepParameter::EndTurn(_)));
        assert!(matches!(end_turn, Some(StepParameter::EndTurn(true))));
    }
}
