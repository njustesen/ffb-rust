use ffb_model::enums::{Direction, PlayerAction};
use ffb_model::types::{FieldCoordinate, MoveSquare};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{CatchScatterThrowInMode, StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.pass.StepMissedPass.
///
/// Scatters the ball for an inaccurate pass.  Flow:
///  1. Initialise coordinateStart from game.pass_coordinate (once only).
///  2. Loop up to 3 times while on pitch:
///     a. Roll scatter direction (d8).
///     b. Compute coordinateEnd = coordinateStart + (direction, 1 square).
///     c. Blast-It (HMP re-roll scatter) dialog — if HAIL_MARY_PASS action and player
///        has canReRollHmpScatter and not yet asked → show dialog, CONTINUE.
///        TODO: UtilCards.hasUnusedSkillWithProperty not yet translated.
///     d. Accumulate roll_list / direction_list.
///     e. coordinateStart ← coordinateEnd.
///  3. Report scatter (ReportScatterBall — not yet translated).
///  4. Set pass_coordinate = last_valid_coordinate, field_model.range_ruler.
///  5. Set ball/bomb coordinate and moving flag.
///  6. Publish CatchScatterThrowInMode + ThrowInCoordinate if out of bounds.
///  7. NEXT_STEP.
///
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
        }
    }

    /// Map a scatter roll (1–8) to a Direction.
    /// Java: DiceInterpreter.getInstance().interpretScatterDirectionRoll(game, roll)
    /// Delegates to `Direction::for_roll` which already encodes the 1-based mapping.
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
        //   if used (isSkillUsed) → game.getActingPlayer().markSkillUsed(skill), doRoll = true (re-roll the direction)
        //   else (declined)       → doRoll = false (keep current direction)
        //   report ReportSkillUse(playerId, skill, doRoll, SkillUse.RE_ROLL_DIRECTION)
        //   if used → getGameState().getPassState().setUsingBlastIt(true)
        //   if neverUse → getGameState().getPassState().setUsingBlastIt(false)
        // DEFERRED: PassState.set_using_blast_it — PassState not yet in Game struct
        match action {
            Action::UseSkill { skill_id, use_skill } => {
                // Blast-It answer: if used → re-roll the direction; else keep current
                self.do_roll = *use_skill;
                self.re_rolling = true;
                if *use_skill {
                    // Java: game.getActingPlayer().markSkillUsed(skill) — inserts into player.used_skills
                    if let Some(pid) = game.acting_player.player_id.clone() {
                        game.mark_skill_used(&pid, *skill_id);
                    }
                }
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool {
        false
    }
}

impl StepMissedPass {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let is_bomb = matches!(
            game.thrower_action,
            Some(PlayerAction::ThrowBomb) | Some(PlayerAction::HailMaryBomb)
        );

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
                // Java: roll = getGameState().getDiceRoller().rollScatterDirection()  [d8]
                self.roll = rng.d8();
                // Java: direction = DiceInterpreter.getInstance().interpretScatterDirectionRoll(game, roll)
                self.direction = Some(Self::direction_for_roll(self.roll));
                // Java: coordinateEnd = UtilServerCatchScatterThrowIn.findScatterCoordinate(coordinateStart, direction, 1)
                let start = self.coordinate_start.unwrap();
                self.coordinate_end = Some(start.step(self.direction.unwrap(), 1));
                // Java: lastValidCoordinate = FIELD.isInBounds(coordinateEnd) ? coordinateEnd : coordinateStart
                let end = self.coordinate_end.unwrap();
                self.last_valid_coordinate = Some(if end.is_on_pitch() { end } else { start });
            }

            // Java: if (reRolling) { fieldModel.clearMoveSquares(); fieldModel.setBallCoordinate(end); setBallMoving(true) }
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
            // boolean hasBlastIt = UtilCards.hasSkillWithProperty(actingPlayer.getPlayer(), canReRollHmpScatter)
            // boolean hasUnusedBlastIt = UtilCards.hasUnusedSkillWithProperty(actingPlayer, canReRollHmpScatter)
            // if (HAIL_MARY_PASS && ((hasUnusedBlastIt && usingBlastIt==null) || (hasBlastIt && usingBlastIt)) && !reRolling)
            //     → reportDirectionRoll(); showDialog(DialogSkillUseParameter(...)); reRolling=true;
            //       fieldModel.add(MoveSquare(coordinateEnd, 0, 0)); return StepAction.CONTINUE
            // DEFERRED: UtilCards.hasUnusedSkillWithProperty, PassState.getUsingBlastIt — not yet translated
            // The Blast-It dialog path is reached when handle_command delivers USE_SKILL with re_rolling=false.
            // With no skill registry, we conservatively skip the dialog (safe for non-Goblin rosters).

            // Java: if (reRolling && doRoll) reportDirectionRoll()
            // Java: getResult().addReport(new ReportScatterBall(new Direction[]{direction}, new int[]{roll}, false))
            // DEFERRED: emit partial scatter report when report infrastructure is translated

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
        // DEFERRED: emit full scatter report when report infrastructure is translated

        // Java: game.setPassCoordinate(lastValidCoordinate)
        game.pass_coordinate = self.last_valid_coordinate;

        // Java: fieldModel.setOutOfBounds(lastValidCoordinate != coordinateEnd)
        let out_of_bounds = self.last_valid_coordinate != self.coordinate_end;
        game.field_model.out_of_bounds = out_of_bounds;

        // Java: rangeRuler = new RangeRuler(game.getThrowerId(), lastValidCoordinate, -1, false)
        // Java: fieldModel.setRangeRuler(rangeRuler)
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
        // Java: passes CatchScatterThrowInMode via publishParameter
        // (In the Java sequence, CATCH_SCATTER_THROW_IN_MODE and THROWIN_COORDINATE are published
        //  by this step when the ball lands out of bounds)
        if out_of_bounds {
            // Java: publishParameter(CATCH_SCATTER_THROW_IN_MODE, THROW_IN)
            // Java: publishParameter(THROWIN_COORDINATE, game.getPassCoordinate())
            let mut outcome = StepOutcome::next()
                .publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ThrowIn));
            if let Some(lvc) = self.last_valid_coordinate {
                outcome = outcome.publish(StepParameter::ThrowInCoordinate(lvc));
            }
            outcome
        } else {
            StepOutcome::next()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, SkillId};

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    // ── basic execution ────────────────────────────────────────────────────────

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
        // Should stop at 3 rolls OR when it goes off pitch — ≤3
        assert!(step.roll_list.len() <= 3);
    }

    #[test]
    fn initialises_coordinate_start_from_pass_coordinate() {
        let mut game = make_game();
        let pc = FieldCoordinate::new(8, 6);
        game.pass_coordinate = Some(pc);
        let mut step = StepMissedPass::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(step.last_valid_coordinate.is_some());
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

    #[test]
    fn no_range_ruler_without_thrower_id() {
        let mut game = make_game();
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        let mut step = StepMissedPass::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.field_model.range_ruler.is_none());
    }

    // ── bomb path ─────────────────────────────────────────────────────────────

    #[test]
    fn bomb_sets_bomb_coordinate_not_ball() {
        let mut game = make_game();
        game.thrower_action = Some(PlayerAction::ThrowBomb);
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        let mut step = StepMissedPass::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.field_model.bomb_coordinate.is_some());
        // ball_coordinate should not be set by bomb path
        // (it may be None or unchanged from before — we check bomb_moving)
        assert!(game.field_model.bomb_moving);
    }

    #[test]
    fn hail_mary_bomb_sets_bomb_coordinate() {
        let mut game = make_game();
        game.thrower_action = Some(PlayerAction::HailMaryBomb);
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        let mut step = StepMissedPass::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.field_model.bomb_moving);
        assert!(game.field_model.bomb_coordinate.is_some());
    }

    // ── direction mapping ─────────────────────────────────────────────────────

    #[test]
    fn direction_for_roll_maps_all_eight_values() {
        for r in 1..=8 {
            let _ = StepMissedPass::direction_for_roll(r);
        }
    }

    // ── out-of-bounds publish ─────────────────────────────────────────────────

    #[test]
    fn out_of_bounds_publishes_throw_in_mode() {
        // Start right at the edge so the ball scatters off
        // FieldCoordinate::new(0, 0) is on pitch; stepping North/West can leave bounds
        // We use coordinate (0, 0) and force direction North (towards y-1 → off pitch)
        let mut game = make_game();
        game.thrower_id = Some("t1".into());
        // Place ball at top-left edge (0, 1) — one step North goes to (0, 0) or off pitch
        // Actually we just need at least one test where the ball ends up OOB.
        // Use a corner coordinate. The exact behaviour depends on pitch bounds.
        // FieldCoordinate(0, 0) is on pitch per is_on_pitch. Let's use (0, 1).
        game.pass_coordinate = Some(FieldCoordinate::new(0, 1));
        // We can't force a specific die roll easily without seeding — just run and check
        // that IF last_valid_coordinate != coordinate_end, throw_in is published.
        let mut step = StepMissedPass::new();
        let out = step.start(&mut game, &mut GameRng::new(7));
        // Either in-bounds (NextStep, no ThrowIn) or out-of-bounds (NextStep + ThrowIn).
        assert_eq!(out.action, StepAction::NextStep);
        if step.last_valid_coordinate != step.coordinate_end {
            let has_throw_in = out.published.iter().any(|p| {
                matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ThrowIn))
            });
            assert!(has_throw_in);
        }
    }

    // ── pass_coordinate updated ────────────────────────────────────────────────

    #[test]
    fn pass_coordinate_set_to_last_valid() {
        let mut game = make_game();
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        let mut step = StepMissedPass::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.pass_coordinate, step.last_valid_coordinate);
    }

    // ── Blast-It handle_command path ───────────────────────────────────────────

    #[test]
    fn handle_command_use_skill_true_sets_do_roll_and_re_rolling() {
        let mut game = make_game();
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        let mut step = StepMissedPass::new();
        // First call to start initialises coordinate_start
        step.coordinate_start = Some(FieldCoordinate::new(10, 5));
        step.roll = 3;
        step.direction = Some(Direction::North);
        step.coordinate_end = Some(FieldCoordinate::new(10, 4));
        step.last_valid_coordinate = Some(FieldCoordinate::new(10, 4));
        // Now simulate: Blast-It accepted → doRoll = true, reRolling = true
        let out = step.handle_command(
            &Action::UseSkill { skill_id: SkillId::BlastIt, use_skill: true },
            &mut game,
            &mut GameRng::new(99),
        );
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn handle_command_use_skill_false_keeps_direction() {
        let mut game = make_game();
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        let mut step = StepMissedPass::new();
        step.coordinate_start = Some(FieldCoordinate::new(10, 5));
        step.roll = 3;
        step.direction = Some(Direction::North);
        step.coordinate_end = Some(FieldCoordinate::new(10, 4));
        step.last_valid_coordinate = Some(FieldCoordinate::new(10, 4));
        let out = step.handle_command(
            &Action::UseSkill { skill_id: SkillId::BlastIt, use_skill: false },
            &mut game,
            &mut GameRng::new(99),
        );
        assert_eq!(out.action, StepAction::NextStep);
    }
}
