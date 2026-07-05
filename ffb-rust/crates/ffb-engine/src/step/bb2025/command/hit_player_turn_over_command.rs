use ffb_model::model::game::Game;
use crate::step::framework::{DeferredCommand, DeferredCommandId, StepParameter};

/// Publishes EndTurn(true) — triggers a turnover when a player is hit.
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.command.HitPlayerTurnOverCommand`.
pub struct HitPlayerTurnOverCommand;

impl DeferredCommand for HitPlayerTurnOverCommand {
    fn id(&self) -> DeferredCommandId { DeferredCommandId::HitPlayer }

    fn execute(&self, _game: &mut Game) -> Vec<StepParameter> {
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
        let params = cmd.execute(&mut game);
        assert!(params.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn publishes_exactly_one_param() {
        let mut game = make_game();
        let params = HitPlayerTurnOverCommand.execute(&mut game);
        assert_eq!(params.len(), 1);
    }
}
