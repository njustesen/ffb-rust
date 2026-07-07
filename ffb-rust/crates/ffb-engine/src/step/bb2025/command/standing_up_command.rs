use ffb_model::model::game::Game;
use crate::step::framework::{DeferredCommand, DeferredCommandId, StepParameter};

/// Clears the acting player's standing_up flag.
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.command.StandingUpCommand`.
pub struct StandingUpCommand;

impl DeferredCommand for StandingUpCommand {
    fn id(&self) -> DeferredCommandId { DeferredCommandId::StandUp }

    fn execute(&self, game: &mut Game) -> Vec<StepParameter> {
        game.acting_player.standing_up = false;
        vec![]
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
    fn clears_standing_up_flag() {
        let mut game = make_game();
        game.acting_player.standing_up = true;
        let cmd = StandingUpCommand;
        let params = cmd.execute(&mut game);
        assert!(!game.acting_player.standing_up);
        assert!(params.is_empty());
    }

    #[test]
    fn noop_when_already_not_standing_up() {
        let mut game = make_game();
        game.acting_player.standing_up = false;
        StandingUpCommand.execute(&mut game);
        assert!(!game.acting_player.standing_up);
    }

    #[test]
    fn id_returns_stand_up_variant() {
        assert_eq!(StandingUpCommand.id(), DeferredCommandId::StandUp);
    }

    #[test]
    fn execute_returns_empty_params() {
        let mut game = make_game();
        game.acting_player.standing_up = true;
        let params = StandingUpCommand.execute(&mut game);
        assert!(params.is_empty());
    }
    #[test]
    fn is_zero_sized_unit_struct() {
        assert_eq!(std::mem::size_of::<StandingUpCommand>(), 0);
    }
}
