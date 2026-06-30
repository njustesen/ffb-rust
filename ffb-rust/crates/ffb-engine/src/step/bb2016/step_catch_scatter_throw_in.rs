/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.StepCatchScatterThrowIn.
///
/// BB2016 has the same ball-handling logic as BB2025 minus:
///   - ThreeSquareScatter (deflected-pass 3-scatter) — not in BB2016
///   - CatchPunt mode — not in BB2016
///   - FailedDeflectionConversion — not in BB2016
///
/// Re-exports BB2025 implementation unchanged: the BB2016 Java class is identical
/// in structure (same fields, same switch cases minus BB2025 additions).
///
/// All game-logic paths map 1:1 to BB2025 (BB2016 Java StepCatchScatterThrowIn
/// is a @RulesCollection(BB2016) copy with identical body — confirmed by reading source).
pub use crate::step::bb2025::shared::step_catch_scatter_throw_in::StepCatchScatterThrowIn;

#[cfg(test)]
mod tests {
    use crate::step::bb2025::shared::step_catch_scatter_throw_in::StepCatchScatterThrowIn;
    use crate::step::framework::{test_team, CatchScatterThrowInMode, Step, StepAction, StepParameter};
    use ffb_model::enums::Rules;
    use ffb_model::model::game::Game;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2016)
    }

    #[test]
    fn no_mode_returns_next() {
        let mut step = StepCatchScatterThrowIn::new();
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn catch_scatter_throw_in_mode_parameter_accepted() {
        let mut step = StepCatchScatterThrowIn::new();
        let accepted = step.set_parameter(
            &StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall)
        );
        assert!(accepted);
        assert_eq!(step.catch_scatter_throw_in_mode, Some(CatchScatterThrowInMode::ScatterBall));
    }

    #[test]
    fn throw_in_coordinate_parameter_accepted() {
        let mut step = StepCatchScatterThrowIn::new();
        let coord = FieldCoordinate::new(3, 3);
        let accepted = step.set_parameter(&StepParameter::ThrowInCoordinate(coord));
        assert!(accepted);
        assert_eq!(step.throw_in_coordinate, Some(coord));
    }

    #[test]
    fn scatter_ball_no_ball_in_play_returns_next() {
        let mut step = StepCatchScatterThrowIn::new();
        let mut game = make_game();
        game.field_model.ball_in_play = false;
        step.catch_scatter_throw_in_mode = Some(CatchScatterThrowInMode::ScatterBall);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(step.catch_scatter_throw_in_mode.is_none());
    }

    #[test]
    fn catcher_id_parameter_accepted() {
        let mut step = StepCatchScatterThrowIn::new();
        let accepted = step.set_parameter(&StepParameter::CatcherId(Some("p1".to_string())));
        assert!(accepted);
        assert_eq!(step.catcher_id, Some("p1".to_string()));
    }
}
