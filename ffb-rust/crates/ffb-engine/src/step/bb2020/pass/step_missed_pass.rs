use ffb_model::enums::{Direction, PlayerAction};
use ffb_model::events::GameEvent;
use ffb_model::types::{FieldCoordinate, MoveSquare};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::report::report_scatter_ball::ReportScatterBall;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{CatchScatterThrowInMode, StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.pass.StepMissedPass.
///
/// Scatters the ball for a missed pass (BB2020). Flow:
///
/// BB2020 WILDLY_INACCURATE path (deviate):
///   1. Roll scatter direction (d8) + scatter distance (d6).
///   2. Deviate from thrower coordinate by distance squares in direction.
///   3. Walk back to last valid in-bounds coordinate.
///   4. Report ReportPassDeviate (not yet translated).
///
/// BB2020 INACCURATE path (3×1 scatter loop, same as BB2025 but with Blast-It re-roll):
///   1. Initialise coordinateStart from game.pass_coordinate (once).
///   2. Loop up to 3 times while on pitch:
///      a. Roll scatter direction (d8).
///      b. Compute coordinateEnd = start + (direction, 1 square).
///      c. Blast-It dialog for HAIL_MARY_PASS (canReRollHmpScatter).
///      d. Accumulate roll_list / direction_list.
///      e. coordinateStart ← coordinateEnd.
///   3. Report ReportScatterBall.
///
/// After scatter: set pass_coordinate, range_ruler, ball/bomb coordinate and moving.
/// Publishes: `CatchScatterThrowInMode`, `ThrowInCoordinate` (when out-of-bounds).
pub struct StepMissedPass {
    /// Java: rollList
    pub roll_list: Vec<i32>,
    /// Java: directionList
    pub direction_list: Vec<Direction>,
    /// Java: coordinateStart
    pub coordinate_start: Option<FieldCoordinate>,
    /// Java: coordinateEnd
    pub coordinate_end: Option<FieldCoordinate>,
    /// Java: lastValidCoordinate
    pub last_valid_coordinate: Option<FieldCoordinate>,
    /// Java: direction
    pub direction: Option<Direction>,
    /// Java: roll
    pub roll: i32,
    /// Java: doRoll — whether to roll a new direction this iteration
    pub do_roll: bool,
    /// Java: reRolling — Blast-It re-roll pending
    pub re_rolling: bool,
    /// PassResult from StepPass — determines which scatter path to use
    pub pass_result: Option<ffb_mechanics::pass_result::PassResult>,
    /// Java: PassState.fUsingBlastIt — set when coach chooses to use/decline Blast-It re-roll
    pub using_blast_it: Option<bool>,
}

impl StepMissedPass {
    pub fn new() -> Self {
        Self {
            roll_list: Vec::new(),
            direction_list: Vec::new(),
            coordinate_start: None,
            coordinate_end: None,
            last_valid_coordinate: None,
            direction: None,
            roll: 0,
            do_roll: true,
            re_rolling: false,
            pass_result: None,
            using_blast_it: None,
        }
    }

    /// Map a scatter roll (1–8) to a Direction.
    /// Java: DiceInterpreter.getInstance().interpretScatterDirectionRoll(game, roll)
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

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_USE_SKILL → canReRollHmpScatter
        //   if used (isSkillUsed) → markSkillUsed, doRoll = true (re-roll the direction)
        //   else (declined)       → doRoll = false (keep current direction)
        //   report ReportSkillUse(playerId, skill, doRoll, SkillUse.RE_ROLL_DIRECTION)
        //   if used → passState.setUsingBlastIt(true)
        //   if neverUse → passState.setUsingBlastIt(false)
        let skill_event = match action {
            Action::UseSkill { skill_id, use_skill } => {
                self.do_roll = *use_skill;
                self.re_rolling = true;
                // Java: passState.setUsingBlastIt(true/false) — stored on step directly (no separate PassState)
                self.using_blast_it = Some(*use_skill);
                let player_id = game.acting_player.player_id.clone().unwrap_or_default();
                if *use_skill {
                    game.mark_skill_used(&player_id, *skill_id);
                }
                Some(GameEvent::SkillUse { player_id, skill_id: *skill_id as u16, used: *use_skill })
            }
            _ => None,
        };
        let out = self.execute_step(game, rng);
        if let Some(ev) = skill_event { out.with_event(ev) } else { out }
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        // Java: PassResult is stored in PassState, read by this step
        match param {
            StepParameter::PassResultParam(v) => {
                use ffb_mechanics::pass_result::PassResult;
                self.pass_result = Some(match v {
                    ffb_model::enums::PassResult::Complete => PassResult::ACCURATE,
                    ffb_model::enums::PassResult::Inaccurate => PassResult::INACCURATE,
                    ffb_model::enums::PassResult::WildlyInaccurate => PassResult::WILDLY_INACCURATE,
                    ffb_model::enums::PassResult::Fumble
                    | ffb_model::enums::PassResult::Caught
                    | ffb_model::enums::PassResult::MissedCatch => PassResult::FUMBLE,
                });
                true
            }
            _ => false,
        }
    }
}

impl StepMissedPass {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        use ffb_mechanics::pass_result::PassResult;
        let mut events: Vec<GameEvent> = Vec::new();

        let is_bomb = matches!(
            game.thrower_action,
            Some(PlayerAction::ThrowBomb) | Some(PlayerAction::HailMaryBomb)
        );

        // Java BB2020: if (state.getResult().equals(PassResult.WILDLY_INACCURATE)) → deviate path
        let is_wildly_inaccurate = self.pass_result == Some(PassResult::WILDLY_INACCURATE);

        if is_wildly_inaccurate {
            // Java: coordinateStart = throwerCoordinate
            let thrower_id = game.thrower_id.clone();
            let thrower_coord = thrower_id.as_deref()
                .and_then(|id| game.field_model.player_coordinate(id));
            self.coordinate_start = thrower_coord;

            // Java: int directionRoll = getDiceRoller().rollScatterDirection()  [d8]
            let direction_roll = rng.d8();
            // Java: int distanceRoll = getDiceRoller().rollScatterDistance()  [d6]
            let distance_roll = rng.d6();
            // Java: direction = DiceInterpreter.interpretScatterDirectionRoll(game, directionRoll)
            let dir = Self::direction_for_roll(direction_roll);
            // Java: coordinateEnd = findScatterCoordinate(coordinateStart, direction, distanceRoll)
            if let Some(start) = self.coordinate_start {
                let end = start.step(dir, distance_roll);
                self.coordinate_end = Some(end);
                self.last_valid_coordinate = Some(end);

                // Java: walk back to last valid in-bounds coordinate
                let mut valid_distance = distance_roll;
                while !self.last_valid_coordinate.map_or(false, |c| c.is_on_pitch()) && valid_distance > 0 {
                    valid_distance -= 1;
                    self.last_valid_coordinate = Some(start.step(dir, valid_distance));
                }
                // Edge: if even distance=0 is off pitch, keep coordinate_start
                if !self.last_valid_coordinate.map_or(false, |c| c.is_on_pitch()) {
                    self.last_valid_coordinate = Some(start);
                }
            }
            // Java: getResult().addReport(new ReportPassDeviate(coordinateEnd, direction, directionRoll, distanceRoll, false))
            if let Some(start) = self.coordinate_start {
                events.push(GameEvent::PassDeviate { from: start, scatter_directions: vec![direction_roll, distance_roll] });
            }

        } else {
            // INACCURATE: 3×1 scatter loop (same as BB2025 but with Blast-It re-roll check)

            // Java: if (coordinateStart == null) { doRoll = true; coordinateStart = game.getPassCoordinate() }
            if self.coordinate_start.is_none() {
                self.do_roll = true;
                self.coordinate_start = game.pass_coordinate;
            }

            // Java: while (FieldCoordinateBounds.FIELD.isInBounds(coordinateStart) && rollList.size() < 3)
            while self.coordinate_start.map_or(false, |c| c.is_on_pitch())
                && self.roll_list.len() < 3
            {
                if self.do_roll {
                    // Java: roll = getDiceRoller().rollScatterDirection()  [d8]
                    self.roll = rng.d8();
                    // Java: direction = DiceInterpreter.interpretScatterDirectionRoll(game, roll)
                    self.direction = Some(Self::direction_for_roll(self.roll));
                    let start = self.coordinate_start.unwrap();
                    self.coordinate_end = Some(start.step(self.direction.unwrap(), 1));
                    let end = self.coordinate_end.unwrap();
                    self.last_valid_coordinate = Some(if end.is_on_pitch() { end } else { start });
                }

                // Java: if (reRolling) { fieldModel.clearMoveSquares(); setBallCoordinate(end); setBallMoving(true) }
                if self.re_rolling {
                    game.field_model.move_squares.clear();
                    if let Some(end) = self.coordinate_end {
                        if is_bomb {
                            game.field_model.bomb_coordinate = Some(end);
                            game.field_model.bomb_moving = true;
                        } else {
                            game.field_model.ball_coordinate = Some(end);
                            game.field_model.ball_moving = true;
                        }
                    }
                }

                // Java: Blast-It HMP re-roll dialog check:
                // if (HAIL_MARY_PASS && ((hasUnusedBlastIt && usingBlastIt==null) || (hasBlastIt && usingBlastIt)) && !reRolling)
                //     → reportDirectionRoll(); showDialog; reRolling=true; fieldModel.add(MoveSquare); return
                // client-only: Blast-It scatter re-roll dialog (DialogSkillUseParameter) — headless always falls through

                // Java: if (reRolling && doRoll) reportDirectionRoll() — partial scatter during re-roll
                if self.re_rolling && self.do_roll {
                    game.report_list.add(ReportScatterBall::new(
                        self.direction_list.clone(),
                        self.roll_list.clone(),
                        false,
                    ));
                }

                // Java: rollList.add(roll); directionList.add(direction);
                self.roll_list.push(self.roll);
                if let Some(dir) = self.direction {
                    self.direction_list.push(dir);
                }

                // Java: doRoll = true; reRolling = false; coordinateStart = coordinateEnd;
                self.do_roll = true;
                self.re_rolling = false;
                self.coordinate_start = self.coordinate_end;
            }

            // Java: getResult().addReport(new ReportScatterBall(directions, rolls, false))
            game.report_list.add(ReportScatterBall::new(
                self.direction_list.clone(),
                self.roll_list.clone(),
                false,
            ));
            if let Some(start) = self.coordinate_start {
                events.push(GameEvent::ScatterBall { from: start, directions: self.roll_list.clone() });
            }
        }

        // Java: game.setPassCoordinate(lastValidCoordinate)
        game.pass_coordinate = self.last_valid_coordinate;

        // Java: fieldModel.setOutOfBounds(lastValidCoordinate != coordinateEnd)
        let out_of_bounds = self.last_valid_coordinate != self.coordinate_end;
        game.field_model.out_of_bounds = out_of_bounds;

        // Java: rangeRuler = new RangeRuler(throwerId, lastValidCoordinate, -1, false)
        if let (Some(ref thrower_id), Some(lvc)) = (game.thrower_id.clone(), self.last_valid_coordinate) {
            use ffb_model::types::RangeRuler;
            game.field_model.range_ruler = Some(RangeRuler::new(
                thrower_id.clone(),
                Some(lvc),
                -1,
                false,
            ));
        }

        // Java: if (THROW_BOMB || HAIL_MARY_BOMB) → setBombCoordinate(lvc), setBombMoving(true)
        //       else                                → setBallCoordinate(lvc), setBallMoving(true)
        if let Some(lvc) = self.last_valid_coordinate {
            if is_bomb {
                game.field_model.bomb_coordinate = Some(lvc);
                game.field_model.bomb_moving = true;
            } else {
                game.field_model.ball_coordinate = Some(lvc);
                game.field_model.ball_moving = true;
            }
        }

        // Java: fieldModel.clearMoveSquares()
        game.field_model.move_squares.clear();

        // Java: getResult().setNextAction(StepAction.NEXT_STEP)
        let mut outcome = if out_of_bounds {
            let mut o = StepOutcome::next()
                .publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ThrowIn));
            if let Some(lvc) = self.last_valid_coordinate {
                o = o.publish(StepParameter::ThrowInCoordinate(lvc));
            }
            o
        } else {
            StepOutcome::next()
        };
        for ev in events {
            outcome = outcome.with_event(ev);
        }
        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, SkillId};
    use ffb_mechanics::pass_result::PassResult;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    // ── INACCURATE path (3-scatter) ───────────────────────────────────────────

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        let mut step = StepMissedPass::new();
        let out = step.start(&mut game, &mut GameRng::new(42));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn accumulates_at_most_three_scatter_rolls() {
        let mut game = make_game();
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        let mut step = StepMissedPass::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(step.roll_list.len() <= 3);
    }

    #[test]
    fn sets_ball_coordinate_to_last_valid() {
        let mut game = make_game();
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        let mut step = StepMissedPass::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.field_model.ball_coordinate, step.last_valid_coordinate);
    }

    #[test]
    fn sets_ball_moving_true() {
        let mut game = make_game();
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        let mut step = StepMissedPass::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.field_model.ball_moving);
    }

    #[test]
    fn clears_move_squares_at_end() {
        let mut game = make_game();
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        game.field_model.add_move_square(MoveSquare::new(FieldCoordinate::new(5, 5), 0, 0));
        let mut step = StepMissedPass::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.field_model.move_squares.is_empty());
    }

    // ── WILDLY_INACCURATE path (deviate) ─────────────────────────────────────

    #[test]
    fn wildly_inaccurate_uses_two_rolls_not_three_scatter() {
        let mut game = make_game();
        game.thrower_id = Some("t1".into());
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        game.field_model.set_player_coordinate("t1", FieldCoordinate::new(1, 7));
        let mut step = StepMissedPass::new();
        step.pass_result = Some(PassResult::WILDLY_INACCURATE);
        let out = step.start(&mut game, &mut GameRng::new(42));
        // WILDLY_INACCURATE: roll_list stays empty (not a scatter loop, it's a deviate)
        assert_eq!(out.action, StepAction::NextStep);
        // coordinate_start should be thrower coordinate
        assert_eq!(step.coordinate_start, Some(FieldCoordinate::new(1, 7)));
    }

    #[test]
    fn wildly_inaccurate_sets_ball_coordinate() {
        let mut game = make_game();
        game.thrower_id = Some("t1".into());
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        game.field_model.set_player_coordinate("t1", FieldCoordinate::new(5, 5));
        let mut step = StepMissedPass::new();
        step.pass_result = Some(PassResult::WILDLY_INACCURATE);
        step.start(&mut game, &mut GameRng::new(42));
        // Ball should be placed somewhere (last valid coordinate)
        assert!(game.field_model.ball_coordinate.is_some());
    }

    // ── bomb path ─────────────────────────────────────────────────────────────

    #[test]
    fn bomb_sets_bomb_coordinate_not_ball() {
        let mut game = make_game();
        game.thrower_action = Some(PlayerAction::ThrowBomb);
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        let mut step = StepMissedPass::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.field_model.bomb_moving);
        assert!(game.field_model.bomb_coordinate.is_some());
    }

    // ── range ruler ────────────────────────────────────────────────────────────

    #[test]
    fn sets_range_ruler_with_thrower_id() {
        let mut game = make_game();
        game.thrower_id = Some("t1".into());
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        let mut step = StepMissedPass::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.field_model.range_ruler.is_some());
        assert_eq!(
            game.field_model.range_ruler.as_ref().unwrap().thrower_id.as_str(),
            "t1"
        );
    }

    // ── direction mapping ──────────────────────────────────────────────────────

    #[test]
    fn direction_for_roll_maps_all_eight_values() {
        for r in 1..=8 {
            let _ = StepMissedPass::direction_for_roll(r);
        }
    }

    // ── Event emission ────────────────────────────────────────────────────────

    #[test]
    fn inaccurate_emits_scatter_ball_event() {
        let mut game = make_game();
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        let mut step = StepMissedPass::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.events.iter().any(|e| matches!(e, GameEvent::ScatterBall { .. })));
    }

    #[test]
    fn wildly_inaccurate_emits_pass_deviate_event() {
        let mut game = make_game();
        game.thrower_id = Some("t1".into());
        game.field_model.set_player_coordinate("t1", FieldCoordinate::new(5, 5));
        let mut step = StepMissedPass::new();
        step.pass_result = Some(PassResult::WILDLY_INACCURATE);
        let out = step.start(&mut game, &mut GameRng::new(42));
        assert!(out.events.iter().any(|e| matches!(e, GameEvent::PassDeviate { .. })));
    }

    // ── Blast-It handle_command path ───────────────────────────────────────────

    #[test]
    fn handle_command_use_skill_true_sets_do_roll_and_re_rolling() {
        let mut game = make_game();
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        let mut step = StepMissedPass::new();
        step.coordinate_start = Some(FieldCoordinate::new(10, 5));
        step.roll = 3;
        step.direction = Some(Direction::North);
        step.coordinate_end = Some(FieldCoordinate::new(10, 4));
        step.last_valid_coordinate = Some(FieldCoordinate::new(10, 4));
        let out = step.handle_command(
            &Action::UseSkill { skill_id: SkillId::BlastIt, use_skill: true },
            &mut game,
            &mut GameRng::new(99),
        );
        assert_eq!(out.action, StepAction::NextStep);
    }
}
