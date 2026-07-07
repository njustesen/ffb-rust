use ffb_model::model::game::Game;
use crate::step::framework::{DeferredCommand, DeferredCommandId, StepParameter};

/// Publishes UseAlternateLabel and clears ThrownPlayerCoordinate for Animal Savagery control flow.
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.command.AnimalSavageryControlCommand`.
pub struct AnimalSavageryControlCommand;

impl DeferredCommand for AnimalSavageryControlCommand {
    fn id(&self) -> DeferredCommandId { DeferredCommandId::AnimalSavageryControl }

    fn execute(&self, _game: &mut Game) -> Vec<StepParameter> {
        vec![
            StepParameter::UseAlternateLabel(true),
            StepParameter::ThrownPlayerCoordinate(None),
        ]
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
    fn publishes_use_alternate_label_and_thrown_player_coord() {
        let mut game = make_game();
        let cmd = AnimalSavageryControlCommand;
        let params = cmd.execute(&mut game);
        assert!(params.iter().any(|p| matches!(p, StepParameter::UseAlternateLabel(true))));
        assert!(params.iter().any(|p| matches!(p, StepParameter::ThrownPlayerCoordinate(None))));
    }

    #[test]
    fn publishes_exactly_two_params() {
        let mut game = make_game();
        let params = AnimalSavageryControlCommand.execute(&mut game);
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn id_returns_correct_variant() {
        assert_eq!(AnimalSavageryControlCommand.id(), DeferredCommandId::AnimalSavageryControl);
    }

    #[test]
    fn thrown_player_coordinate_is_none() {
        let mut game = make_game();
        let params = AnimalSavageryControlCommand.execute(&mut game);
        let coord = params.iter().find(|p| matches!(p, StepParameter::ThrownPlayerCoordinate(_)));
        assert!(matches!(coord, Some(StepParameter::ThrownPlayerCoordinate(None))));
    }
    #[test]
    fn is_zero_sized_unit_struct() {
        assert_eq!(std::mem::size_of::<AnimalSavageryControlCommand>(), 0);
    }
}
