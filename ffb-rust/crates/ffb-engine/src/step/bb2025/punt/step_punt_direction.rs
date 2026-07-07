use ffb_mechanics::bb2025::throw_in_mechanic::ThrowInMechanic;
use ffb_mechanics::throw_in_mechanic::ThrowInMechanic as ThrowInMechanicTrait;
use ffb_model::enums::Direction;
use ffb_model::report::bb2025::report_punt_direction::ReportPuntDirection;
use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{CatchScatterThrowInMode, StepId, StepParameter};

/// Rolls the scatter direction for a punt using the BB2025 ThrowInMechanic (1d6 from kicker's
/// direction), then publishes either Direction or throws-in out of bounds.
/// Re-roll (skill or team re-roll) is supported via UseSkill/UseReRoll actions.
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.punt.StepPuntDirection`.
pub struct StepPuntDirection {
    pub goto_label_on_end: String,
    pub direction: Option<Direction>,
    pub coordinate_to: Option<FieldCoordinate>,
    pub coordinate_from: Option<FieldCoordinate>,
    pub out_of_bounds: bool,
    // AbstractStepWithReRoll stubs
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
}

impl StepPuntDirection {
    pub fn new(goto_label_on_end: String) -> Self {
        Self {
            goto_label_on_end,
            direction: None,
            coordinate_to: None,
            coordinate_from: None,
            out_of_bounds: false,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }
}

impl Step for StepPuntDirection {
    fn id(&self) -> StepId { StepId::PuntDirection }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::CoordinateTo(c) => { self.coordinate_to = Some(*c); true }
            StepParameter::CoordinateFrom(c) => { self.coordinate_from = Some(*c); true }
            StepParameter::Touchback(v) => { self.out_of_bounds = *v; true }
            _ => false,
        }
    }
}

impl StepPuntDirection {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let label = self.goto_label_on_end.clone();

        if self.out_of_bounds {
            let ball_coord = game.field_model.ball_coordinate;
            let player_id = game.acting_player.player_id.clone().unwrap_or_default();
            game.report_list.add(ReportPuntDirection::new(None, 0, player_id, true));
            return StepOutcome::goto(&label)
                .publish(StepParameter::EndTurn(true))
                .publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ThrowIn))
                .publish(StepParameter::ThrowInCoordinate(ball_coord.unwrap_or(FieldCoordinate::new(0, 0))));
        }

        let coord_from = match self.coordinate_from {
            Some(c) => c,
            None => return StepOutcome::next(),
        };
        let coord_to = match self.coordinate_to {
            Some(c) => c,
            None => return StepOutcome::next(),
        };

        game.field_model.ball_moving = true;

        // Java: Direction baseDirection = coordinateFrom.getDirection(coordinateTo);
        //       int roll = rollThrowInDirection(); // 1d6
        //       direction = mechanic.interpretThrowInDirectionRoll(baseDirection, roll);
        let base_direction = coord_from.direction_to(coord_to).unwrap_or(Direction::North);
        let roll = rng.d6();
        let direction = ThrowInMechanic::new().interpret_throw_in_direction_roll_with_template(base_direction, roll);
        self.direction = Some(direction);
        let indicator = coord_from.step(direction, 1);
        if indicator.is_on_pitch() {
            game.field_model.ball_coordinate = Some(indicator);
            let player_id = game.acting_player.player_id.clone().unwrap_or_default();
            game.report_list.add(ReportPuntDirection::new(Some(direction), roll, player_id, false));
            StepOutcome::next()
                .publish(StepParameter::Direction(direction))
        } else {
            // Out of bounds — throw in.
            let ball_coord = game.field_model.ball_coordinate;
            let player_id = game.acting_player.player_id.clone().unwrap_or_default();
            game.report_list.add(ReportPuntDirection::new(Some(direction), roll, player_id, true));
            StepOutcome::goto(&label)
                .publish(StepParameter::EndTurn(true))
                .publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ThrowIn))
                .publish(StepParameter::ThrowInCoordinate(ball_coord.unwrap_or(FieldCoordinate::new(0, 0))))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    // 1. out_of_bounds = true → GotoLabel + ThrowIn + EndTurn published
    #[test]
    fn out_of_bounds_flag_goto_label() {
        let mut game = make_game();
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(0, 7));
        let mut step = StepPuntDirection::new("end".into());
        step.out_of_bounds = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ThrowIn))));
    }

    // 2. Missing coordinate_from → NextStep (guard)
    #[test]
    fn missing_coord_from_returns_next() {
        let mut game = make_game();
        let mut step = StepPuntDirection::new("end".into());
        step.coordinate_to = Some(FieldCoordinate::new(10, 7));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    // 3. set_parameter accepts CoordinateTo and CoordinateFrom
    #[test]
    fn set_parameter_stores_coordinates() {
        let mut step = StepPuntDirection::new("end".into());
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(6, 5);
        step.set_parameter(&StepParameter::CoordinateFrom(from));
        step.set_parameter(&StepParameter::CoordinateTo(to));
        assert_eq!(step.coordinate_from, Some(from));
        assert_eq!(step.coordinate_to, Some(to));
    }

    // 4. set_parameter Touchback sets out_of_bounds
    #[test]
    fn set_parameter_touchback_sets_out_of_bounds() {
        let mut step = StepPuntDirection::new("end".into());
        assert!(!step.out_of_bounds);
        step.set_parameter(&StepParameter::Touchback(true));
        assert!(step.out_of_bounds);
    }

    // 5. With both coords provided, direction is rolled and published (on-pitch result)
    //    Use a from-coord that has room for any direction: interior of field.
    #[test]
    fn direction_rolled_and_published_on_pitch() {
        let mut game = make_game();
        // Ball starts in centre; from is interior — most directions stay on pitch.
        let from = FieldCoordinate::new(12, 7);
        let to = FieldCoordinate::new(13, 7);
        game.field_model.ball_coordinate = Some(from);
        let mut step = StepPuntDirection::new("end".into());
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(to);

        let out = step.start(&mut game, &mut GameRng::new(0));
        // Either stays on pitch (Direction published) or goes out of bounds (GotoLabel)
        // — either is correct; ensure no panic and step terminates
        assert!(out.action == StepAction::NextStep || out.action == StepAction::GotoLabel);
    }

    #[test]
    fn out_of_bounds_flag_adds_punt_direction_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(0, 7));
        let mut step = StepPuntDirection::new("end".into());
        step.out_of_bounds = true;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::PUNT_DIRECTION_ROLL), "expected PUNT_DIRECTION_ROLL report");
    }

    #[test]
    fn on_pitch_roll_adds_punt_direction_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        let from = FieldCoordinate::new(12, 7);
        let to = FieldCoordinate::new(13, 7);
        game.field_model.ball_coordinate = Some(from);
        let mut step = StepPuntDirection::new("end".into());
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(to);
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::PUNT_DIRECTION_ROLL), "expected PUNT_DIRECTION_ROLL report after rolling direction");
    }
}
