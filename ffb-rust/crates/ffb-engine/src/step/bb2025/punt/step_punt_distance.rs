use ffb_model::enums::{Direction, ReRollSource};
use ffb_model::report::bb2025::report_punt_distance::ReportPuntDistance;
use ffb_model::report::report_id::ReportId;
use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{CatchScatterThrowInMode, StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

/// Rolls d6 distance for a punt and advances the ball that many squares.
/// Publishes CatchScatterThrowInMode::CatchPunt or throws into crowd with ThrowIn.
/// Re-roll (skill/team) is supported via UseSkill/UseReRoll actions.
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.punt.StepPuntDistance`.
pub struct StepPuntDistance {
    pub direction: Option<Direction>,
    pub coordinate_from: Option<FieldCoordinate>,
    pub distance: i32,
    // AbstractStepWithReRoll stubs
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
}

impl StepPuntDistance {
    pub fn new() -> Self {
        Self { direction: None, coordinate_from: None, distance: 0, re_rolled_action: None, re_roll_source: None }
    }
}

impl Default for StepPuntDistance {
    fn default() -> Self { Self::new() }
}

impl Step for StepPuntDistance {
    fn id(&self) -> StepId { StepId::PuntDistance }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_USE_SKILL — if (command.isSkillUsed()) setReRollSource(...); else leave unset (declined).
        if let Action::UseReRoll { use_reroll: false } = action {
            self.re_roll_source = None;
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::Direction(d) => { self.direction = Some(*d); true }
            StepParameter::CoordinateFrom(c) => { self.coordinate_from = Some(*c); true }
            _ => false,
        }
    }
}

impl StepPuntDistance {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: fieldModel.setBallMoving(true);
        game.field_model.ball_moving = true;
        let player_id = game.acting_player.player_id.clone().unwrap_or_default();

        // Java: if (ReRolledActions.PUNT_DISTANCE == getReRolledAction()) {
        //           if (getReRollSource() == null || !UtilServerReRoll.useReRoll(...)) { leave(); return; }
        //       }
        if self.re_rolled_action.as_deref() == Some("PUNT_DISTANCE") {
            match self.re_roll_source.clone() {
                Some(ref source_name) => {
                    let source = ReRollSource::new(source_name.as_str());
                    if !use_reroll(game, &source, &player_id) {
                        return self.leave(game);
                    }
                    // Re-roll consumed — fall through to roll again.
                }
                None => return self.leave(game),
            }
        }

        let direction = match self.direction {
            Some(d) => d,
            None => return StepOutcome::next(),
        };
        let coord_from = match self.coordinate_from {
            Some(c) => c,
            None => return StepOutcome::next(),
        };

        self.distance = rng.d6() as i32;
        let landing = coord_from.step(direction, self.distance);

        if landing.is_on_pitch() {
            game.field_model.out_of_bounds = false;
            game.field_model.ball_coordinate = Some(landing);
        } else {
            // Find last valid square on path.
            let last = find_last_on_pitch(coord_from, direction, self.distance - 1);
            game.field_model.out_of_bounds = true;
            game.field_model.ball_coordinate = last;
        }

        // Java: getResult().addReport(new ReportPuntDistance(distance, fieldModel.isOutOfBounds()))
        game.report_list.add(ReportPuntDistance::new(self.distance, game.field_model.out_of_bounds));

        // Java: if (getReRolledAction() == null) { setReRolledAction(PUNT_DISTANCE); ... offer re-roll ... }
        if self.re_rolled_action.is_none() {
            self.re_rolled_action = Some("PUNT_DISTANCE".into());
            if let Some(prompt) = ask_for_reroll_if_available(game, "PUNT_DISTANCE", 0, false) {
                self.re_roll_source = Some("TRR".into());
                return StepOutcome::cont().with_prompt(prompt);
            }
        }
        self.leave(game)
    }

    /// Java: `StepPuntDistance.leave()` (Animation set-up is client-visual only, not ported).
    fn leave(&mut self, game: &mut Game) -> StepOutcome {
        if game.field_model.out_of_bounds {
            let ball_coord = game.field_model.ball_coordinate;
            StepOutcome::next()
                .publish(StepParameter::EndTurn(true))
                .publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ThrowIn))
                .publish(StepParameter::ThrowInCoordinate(ball_coord.unwrap_or(FieldCoordinate::new(0, 0))))
        } else {
            StepOutcome::next()
                .publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::CatchPunt))
        }
    }
}

fn find_last_on_pitch(from: FieldCoordinate, dir: Direction, dist: i32) -> Option<FieldCoordinate> {
    if dist <= 0 { return None; }
    let c = from.step(dir, dist);
    if c.is_on_pitch() { Some(c) } else { find_last_on_pitch(from, dir, dist - 1) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, Direction};

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    // 1. Missing direction → NextStep (guard)
    #[test]
    fn missing_direction_returns_next() {
        let mut game = make_game();
        let mut step = StepPuntDistance::new();
        step.coordinate_from = Some(FieldCoordinate::new(12, 7));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    // 2. Missing coordinate_from → NextStep (guard)
    #[test]
    fn missing_coord_from_returns_next() {
        let mut game = make_game();
        let mut step = StepPuntDistance::new();
        step.direction = Some(Direction::East);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    // 3. set_parameter stores direction and coordinate_from
    #[test]
    fn set_parameter_stores_fields() {
        let mut step = StepPuntDistance::new();
        let coord = FieldCoordinate::new(5, 7);
        step.set_parameter(&StepParameter::Direction(Direction::North));
        step.set_parameter(&StepParameter::CoordinateFrom(coord));
        assert_eq!(step.direction, Some(Direction::North));
        assert_eq!(step.coordinate_from, Some(coord));
    }

    // 4. On-pitch landing → NextStep + CatchPunt mode
    //    Centre field + East direction: d6 ∈ [1,6], all land on pitch from x=12.
    #[test]
    fn on_pitch_publishes_catch_punt() {
        let mut game = make_game();
        let from = FieldCoordinate::new(5, 7); // East: x+d ≤ 11 still on pitch (25 wide)
        let mut step = StepPuntDistance::new();
        step.direction = Some(Direction::East);
        step.coordinate_from = Some(from);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::CatchPunt))));
    }

    // 5. find_last_on_pitch helper: boundary from coord going West off pitch
    #[test]
    fn find_last_on_pitch_stops_at_boundary() {
        // Start at x=2 going West (dx=-1); dist=3 would reach x=-1 (out of bounds)
        let from = FieldCoordinate::new(2, 7);
        let result = find_last_on_pitch(from, Direction::West, 3);
        // last valid square going West from x=2 is x=0 (dist=2) since x=-1 is off pitch
        assert!(result.is_some());
        let c = result.unwrap();
        assert!(c.is_on_pitch());
    }

    #[test]
    fn report_punt_distance_added_on_valid_step() {
        let mut game = make_game();
        let from = FieldCoordinate::new(5, 7);
        let mut step = StepPuntDistance::new();
        step.direction = Some(Direction::East);
        step.coordinate_from = Some(from);
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::PUNT_DISTANCE_ROLL));
    }

    #[test]
    fn no_punt_distance_report_without_direction() {
        let mut game = make_game();
        let mut step = StepPuntDistance::new();
        step.coordinate_from = Some(FieldCoordinate::new(5, 7));
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.report_list.has_report(ReportId::PUNT_DISTANCE_ROLL));
    }

    // Java: `fieldModel.setOutOfBounds(false)` runs unconditionally on the in-bounds path
    // (isInBounds(ballIndicatorCoordinate)). If a prior step (e.g. StepPuntDirection) had
    // left `out_of_bounds = true`, an in-bounds distance roll must clear it — otherwise
    // downstream CatchScatterThrowIn logic (which reads this flag) would wrongly treat an
    // on-pitch landing as a throw-in.
    #[test]
    fn on_pitch_landing_clears_stale_out_of_bounds_flag() {
        let mut game = make_game();
        game.field_model.out_of_bounds = true; // stale from a prior step
        let from = FieldCoordinate::new(5, 7); // East: any d6 distance stays on pitch
        let mut step = StepPuntDistance::new();
        step.direction = Some(Direction::East);
        step.coordinate_from = Some(from);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.field_model.out_of_bounds, "expected out_of_bounds to be cleared on in-bounds landing");
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::CatchPunt))));
    }
}
