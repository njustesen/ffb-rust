use ffb_model::enums::ApothecaryMode;
use ffb_model::model::game::Game;
use crate::step::framework::{DeferredCommand, DeferredCommandId, StepParameter};
use crate::step::util_server_injury::drop_player;

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

    fn execute(&self, game: &mut Game) -> Vec<StepParameter> {
        // Java: UtilServerInjury.dropPlayer(step, player, apothecaryMode, eligibleForSafePairOfHands)
        // apothecary_mode controls which team's apo gets offered — dialog infra needed for full port.
        drop_player(game, &self.player_id, self.eligible_for_safe_pair_of_hands)
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
        let params = cmd.execute(&mut game);
        assert!(params.is_empty());
    }

    #[test]
    fn id_is_drop_player() {
        use crate::step::framework::DeferredCommandId;
        let cmd = DropPlayerCommand::new("p1".into(), ApothecaryMode::Defender, false);
        assert_eq!(cmd.id(), DeferredCommandId::DropPlayer);
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
        let params = cmd.execute(&mut game);
        assert!(params.is_empty());
    }
}
