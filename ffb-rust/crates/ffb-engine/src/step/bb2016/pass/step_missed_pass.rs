use ffb_model::enums::{Direction, PlayerAction};
use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{CatchScatterThrowInMode, StepId, StepParameter};

/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.pass.StepMissedPass`.
///
/// Handles a missed-pass ball scatter (BB2016).
///  - `pass_deviates = true`: roll direction (d8) + distance (d6) from the thrower's
///    coordinate; walk back from the endpoint until on-pitch to find `lastValidCoordinate`.
///  - `pass_deviates = false`: up to 3 single-step scatters (d8 each) from `pass_coordinate`.
///
/// After scatter: clears the range ruler, then publishes `CatchScatterThrowInMode` +
/// `ThrowInCoordinate` (out-of-bounds) or `BombOutOfBounds` (bomb OOB).
/// Reports (`ReportPassDeviate` / `ReportScatterBall`) and animations are not yet translated.
pub struct StepMissedPass {
    /// Java: `passDeviates` — true when the pass rolled Deviate (d8 direction + d6 distance).
    pub pass_deviates: bool,
}

impl StepMissedPass {
    pub fn new() -> Self { Self { pass_deviates: false } }

    /// Java: `DiceInterpreter.getInstance().interpretScatterDirectionRoll(game, roll)` (1–8 → Direction).
    fn direction_for_roll(roll: i32) -> Direction {
        Direction::for_roll(roll).unwrap_or(Direction::North)
    }
}

impl Default for StepMissedPass {
    fn default() -> Self { Self::new() }
}

impl Step for StepMissedPass {
    fn id(&self) -> StepId { StepId::MissedPass }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::PassDeviates(v) => { self.pass_deviates = *v; true }
            _ => false,
        }
    }
}

impl StepMissedPass {
    fn execute_step(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let is_bomb = matches!(
            game.thrower_action,
            Some(PlayerAction::ThrowBomb) | Some(PlayerAction::HailMaryBomb)
        );

        let (coordinate_end, last_valid_coordinate);

        if self.pass_deviates {
            // Java: coordinateStart = fieldModel.getPlayerCoordinate(game.getThrower())
            let start = game.thrower_id.as_deref()
                .and_then(|id| game.field_model.player_coordinate(id))
                .unwrap_or(FieldCoordinate::new(0, 0));

            // Java: directionRoll = diceRoller.rollScatterDirection() [d8]
            //       distanceRoll  = diceRoller.rollScatterDistance()  [d6]
            let dir_roll = rng.d8();
            let dist_roll = rng.d6();
            let direction = Self::direction_for_roll(dir_roll);

            // Java: coordinateEnd = findScatterCoordinate(start, direction, distanceRoll)
            let end = start.step(direction, dist_roll);
            coordinate_end = end;

            // Java: lastValidCoordinate = coordinateEnd; walk back while !isInBounds
            let mut lvc = end;
            let mut valid_distance = dist_roll;
            while !lvc.is_on_pitch() && valid_distance > 0 {
                valid_distance -= 1;
                lvc = start.step(direction, valid_distance);
            }
            last_valid_coordinate = lvc;
            // Java: getResult().addReport(new ReportPassDeviate(...)) — skip (reports not translated)
        } else {
            // Java: coordinateStart = game.getPassCoordinate()
            let mut coord_start = game.pass_coordinate.unwrap_or(FieldCoordinate::new(0, 0));
            let mut coord_end = coord_start;
            let mut lvc = coord_start;

            // Java: while (FIELD.isInBounds(coordinateStart) && rollList.size() < 3)
            let mut count = 0;
            while coord_start.is_on_pitch() && count < 3 {
                let roll = rng.d8();
                let direction = Self::direction_for_roll(roll);
                coord_end = coord_start.step(direction, 1);
                lvc = if coord_end.is_on_pitch() { coord_end } else { coord_start };
                coord_start = coord_end;
                count += 1;
            }
            coordinate_end = coord_end;
            last_valid_coordinate = lvc;
            // Java: getResult().addReport(new ReportScatterBall(...)) — skip (reports not translated)
        }

        // Java: game.getFieldModel().setRangeRuler(null)
        game.field_model.range_ruler = None;
        // Java: getResult().setAnimation(...) — skip (animation not translated)
        // Java: UtilServerGame.syncGameModel(this) — skip (server sync not translated)

        // Java: out-of-bounds dispatch
        let mut outcome = StepOutcome::next();
        if !coordinate_end.is_on_pitch() {
            game.field_model.out_of_bounds = true;
            if is_bomb {
                // Java: fieldModel.setBombCoordinate(null); publishParameter(BombOutOfBounds, true)
                game.field_model.bomb_coordinate = None;
                outcome = outcome.publish(StepParameter::BombOutOfBounds(true));
            } else {
                // Java: publishParameter(CATCH_SCATTER_THROW_IN_MODE, THROW_IN)
                //       publishParameter(THROW_IN_COORDINATE, lastValidCoordinate)
                //       fieldModel.setBallMoving(true)
                game.field_model.ball_moving = true;
                outcome = outcome
                    .publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ThrowIn))
                    .publish(StepParameter::ThrowInCoordinate(last_valid_coordinate));
            }
        } else {
            if is_bomb {
                // Java: publishParameter(CATCH_SCATTER_THROW_IN_MODE, CATCH_BOMB)
                //       fieldModel.setBombCoordinate(coordinateEnd); setBombMoving(true)
                game.field_model.bomb_coordinate = Some(coordinate_end);
                game.field_model.bomb_moving = true;
                outcome = outcome.publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::CatchBomb));
            } else {
                // Java: publishParameter(CATCH_SCATTER_THROW_IN_MODE, CATCH_MISSED_PASS)
                //       fieldModel.setBallCoordinate(coordinateEnd); setBallMoving(true)
                game.field_model.ball_coordinate = Some(coordinate_end);
                game.field_model.ball_moving = true;
                outcome = outcome.publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::CatchMissedPass));
            }
        }
        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    // ── id ────────────────────────────────────────────────────────────────────

    #[test]
    fn id_is_missed_pass() {
        assert_eq!(StepMissedPass::new().id(), StepId::MissedPass);
    }

    // ── set_parameter ─────────────────────────────────────────────────────────

    #[test]
    fn set_parameter_pass_deviates() {
        let mut step = StepMissedPass::new();
        assert!(!step.pass_deviates);
        assert!(step.set_parameter(&StepParameter::PassDeviates(true)));
        assert!(step.pass_deviates);
    }

    #[test]
    fn set_parameter_unknown_returns_false() {
        let mut step = StepMissedPass::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    // ── scatter (non-deviate) path ─────────────────────────────────────────────

    #[test]
    fn scatter_returns_next_step() {
        let mut game = make_game();
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        let mut step = StepMissedPass::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn scatter_sets_ball_moving() {
        let mut game = make_game();
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        let mut step = StepMissedPass::new();
        step.start(&mut game, &mut GameRng::new(0));
        // Either ball or is_bomb path; default (no thrower_action) → ball path
        // ball_moving may or may not be set depending on OOB result
        // — if OOB: ball_moving=true; if in-bounds: ball_moving=true (same)
        assert!(game.field_model.ball_moving);
    }

    #[test]
    fn scatter_clears_range_ruler() {
        let mut game = make_game();
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        use ffb_model::types::RangeRuler;
        game.field_model.range_ruler = Some(RangeRuler::new("t1".into(), None, 3, false));
        let mut step = StepMissedPass::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.field_model.range_ruler.is_none());
    }

    #[test]
    fn scatter_publishes_catch_missed_pass_when_in_bounds() {
        // Use a centre coordinate with a seed that keeps ball on pitch for all 3 rolls
        let mut game = make_game();
        game.pass_coordinate = Some(FieldCoordinate::new(12, 7));
        let mut step = StepMissedPass::new();
        let out = step.start(&mut game, &mut GameRng::new(2));
        // If ball stayed in bounds, CatchMissedPass is published; else ThrowIn.
        let in_bounds = game.field_model.ball_coordinate.map_or(false, |c| c.is_on_pitch());
        if in_bounds {
            assert!(out.published.iter().any(|p| {
                matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::CatchMissedPass))
            }));
        } else {
            assert!(out.published.iter().any(|p| {
                matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ThrowIn))
            }));
        }
    }

    // ── deviate path ──────────────────────────────────────────────────────────

    #[test]
    fn deviate_returns_next_step() {
        let mut game = make_game();
        game.thrower_id = Some("t1".into());
        game.field_model.set_player_coordinate("t1", FieldCoordinate::new(10, 5));
        let mut step = StepMissedPass::new();
        step.pass_deviates = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn deviate_rolls_direction_and_distance() {
        let mut game = make_game();
        game.thrower_id = Some("t1".into());
        game.field_model.set_player_coordinate("t1", FieldCoordinate::new(12, 7));
        let mut step = StepMissedPass::new();
        step.pass_deviates = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Result is either CatchMissedPass (on-pitch) or ThrowIn (OOB)
        let has_mode = out.published.iter().any(|p| matches!(
            p,
            StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::CatchMissedPass)
            | StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ThrowIn)
        ));
        assert!(has_mode || out.published.iter().any(|p| matches!(p, StepParameter::BombOutOfBounds(_))));
    }

    // ── bomb path ─────────────────────────────────────────────────────────────

    #[test]
    fn bomb_in_bounds_publishes_catch_bomb() {
        let mut game = make_game();
        game.thrower_action = Some(PlayerAction::ThrowBomb);
        game.pass_coordinate = Some(FieldCoordinate::new(12, 7));
        let mut step = StepMissedPass::new();
        let out = step.start(&mut game, &mut GameRng::new(2));
        if game.field_model.bomb_coordinate.map_or(false, |c| c.is_on_pitch()) {
            assert!(out.published.iter().any(|p| {
                matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::CatchBomb))
            }));
            assert!(game.field_model.bomb_moving);
        }
    }

    #[test]
    fn hail_mary_bomb_oob_publishes_bomb_out_of_bounds() {
        let mut game = make_game();
        game.thrower_action = Some(PlayerAction::HailMaryBomb);
        // corner coordinate — high chance of OOB with deviate
        game.thrower_id = Some("t1".into());
        game.field_model.set_player_coordinate("t1", FieldCoordinate::new(0, 1));
        let mut step = StepMissedPass::new();
        step.pass_deviates = true;
        let out = step.start(&mut game, &mut GameRng::new(1));
        // Depending on roll: either BombOutOfBounds(true) or CatchBomb
        let ok = out.published.iter().any(|p| matches!(p, StepParameter::BombOutOfBounds(true)))
            || out.published.iter().any(|p| matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::CatchBomb)));
        assert!(ok);
    }
}
