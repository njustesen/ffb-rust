use ffb_model::enums::ApothecaryMode;
use ffb_model::model::game::Game;
use crate::step::framework::{DeferredCommand, DeferredCommandId, StepParameter};
use crate::step::util_server_injury::drop_player;

/// Drops a player hit by a bomb: runs injury, optionally suppresses the turnover, and preserves
/// the active flag for non-bombardiers. Mirrors Java
/// `com.fumbbl.ffb.server.step.bb2025.command.DropPlayerFromBombCommand`.
pub struct DropPlayerFromBombCommand {
    pub player_id: String,
    pub apothecary_mode: ApothecaryMode,
    pub eligible_for_safe_pair_of_hands: bool,
    pub was_active: bool,
    pub suppress_end_turn: bool,
}

impl DropPlayerFromBombCommand {
    pub fn new(
        player_id: String,
        apothecary_mode: ApothecaryMode,
        eligible_for_safe_pair_of_hands: bool,
        was_active: bool,
        suppress_end_turn: bool,
    ) -> Self {
        Self { player_id, apothecary_mode, eligible_for_safe_pair_of_hands, was_active, suppress_end_turn }
    }
}

impl DeferredCommand for DropPlayerFromBombCommand {
    fn id(&self) -> DeferredCommandId { DeferredCommandId::DropPlayerFromBomb }

    fn execute(&self, game: &mut Game) -> Vec<StepParameter> {
        // Java: UtilServerInjury.dropPlayer(step, player, apothecaryMode, eligibleForSafePairOfHands)
        // apothecary_mode and was_active (original_bombardier comparison) deferred — dialog infra needed.
        let mut params = drop_player(game, &self.player_id, self.eligible_for_safe_pair_of_hands);
        // Java: if (suppressEndTurn) → remove END_TURN from published params
        if self.suppress_end_turn {
            params.retain(|p| !matches!(p, StepParameter::EndTurn(_)));
        }
        params
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
        let cmd = DropPlayerFromBombCommand::new("p1".into(), ApothecaryMode::Defender, true, false, false);
        let params = cmd.execute(&mut game);
        assert!(params.is_empty());
    }
}
