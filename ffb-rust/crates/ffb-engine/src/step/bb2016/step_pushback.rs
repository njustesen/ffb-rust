/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.StepPushback.
///
/// BB2016 pushback is identical in structure to BB2025 pushback.
/// The Java class differs only by @RulesCollection annotation.
/// Re-exports the BB2025 implementation directly.
///
/// Expects: STARTING_PUSHBACK_SQUARE, OLD_DEFENDER_STATE.
/// Sets: CATCH_SCATTER_THROW_IN_MODE, DEFENDER_PUSHED, FOLLOWUP_CHOICE,
///       STARTING_PUSHBACK_SQUARE, INJURY_RESULT.
pub use crate::step::bb2025::block::step_pushback::StepPushback;

#[cfg(test)]
mod tests {
    use crate::step::bb2025::block::step_pushback::StepPushback;
    use crate::step::framework::{test_team, Step, StepAction, StepParameter};
    use ffb_model::enums::{Rules, PS_STANDING, PlayerState};
    use ffb_model::model::game::Game;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2016)
    }

    #[test]
    fn no_starting_square_stays_cont() {
        let mut step = StepPushback::new();
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn old_defender_state_parameter_accepted() {
        let mut step = StepPushback::new();
        let state = PlayerState::new(PS_STANDING);
        let accepted = step.set_parameter(&StepParameter::OldDefenderState(state));
        assert!(accepted);
        assert!(step.old_defender_state.is_some());
    }

    #[test]
    fn starting_pushback_square_parameter_accepted() {
        use ffb_model::enums::Direction;
        use ffb_model::types::PushbackSquare;
        let mut step = StepPushback::new();
        let coord = FieldCoordinate::new(7, 5);
        let sq = PushbackSquare::new(coord, Direction::North, true);
        let accepted = step.set_parameter(&StepParameter::StartingPushbackSquare(Some(sq)));
        assert!(accepted);
        assert!(step.starting_pushback_square.is_some());
        assert_eq!(step.starting_pushback_square.unwrap().coordinate, coord);
    }

    #[test]
    fn push_to_command_on_empty_square_publishes_defender_pushed() {
        let mut step = StepPushback::new();
        let coord = FieldCoordinate::new(5, 5);
        let mut game = make_game();
        game.defender_id = Some("p1".into());
        let out = step.handle_command(
            &crate::action::Action::PushTo { coord },
            &mut game,
            &mut GameRng::new(0),
        );
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::DefenderPushed(true))));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepPushback::new();
        let accepted = step.set_parameter(&StepParameter::EndTurn(true));
        assert!(!accepted);
    }
}
