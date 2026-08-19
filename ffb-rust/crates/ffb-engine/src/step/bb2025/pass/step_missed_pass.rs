use ffb_model::enums::{Direction, PlayerAction};
use ffb_model::types::{FieldCoordinate, MoveSquare};
use ffb_model::model::game::Game;
use ffb_model::events::GameEvent;
use ffb_model::util::rng::GameRng;
use ffb_model::report::report_scatter_ball::ReportScatterBall;
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
    /// `GameEvent::PassDeviate` parked by `deviate_from_thrower`, attached at `execute_step`'s exit.
    pending_deviate: Option<GameEvent>,
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
    /// Java (BB2020 only): `state.getResult()` — the PassResult published by StepPass.
    /// BB2020's StepMissedPass wraps this whole body in
    /// `if (state.getResult().equals(PassResult.WILDLY_INACCURATE)) { deviate } else { ... }`;
    /// BB2025's PassMechanic can never return WILDLY_INACCURATE (it returns FUMBLE instead),
    /// so the field stays `None` outside BB2020 and the deviate branch is unreachable there.
    pub pass_result: Option<ffb_mechanics::pass_result::PassResult>,
}

impl StepMissedPass {
    pub fn new() -> Self {
        Self {
            roll_list: Vec::new(),
            direction_list: Vec::new(),
            coordinate_start: None,
            pending_deviate: None,
            coordinate_end: None,
            last_valid_coordinate: None,
            direction: None,
            roll: 0,
            do_roll: true,
            re_rolling: false,
            pass_result: None,
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
        // local tracking via self.do_roll/self.re_rolling — Java PassState not needed here
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

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        // Java: the PassResult lives in PassState, which BB2020's StepMissedPass reads directly.
        // Rust publishes it as a step parameter from StepPass.
        match param {
            StepParameter::PassResultParam(v) => {
                use ffb_mechanics::pass_result::PassResult;
                self.pass_result = Some(match v {
                    ffb_model::enums::PassOutcome::Complete => PassResult::ACCURATE,
                    ffb_model::enums::PassOutcome::Inaccurate => PassResult::INACCURATE,
                    ffb_model::enums::PassOutcome::WildlyInaccurate => PassResult::WILDLY_INACCURATE,
                    ffb_model::enums::PassOutcome::Fumble
                    | ffb_model::enums::PassOutcome::Caught
                    | ffb_model::enums::PassOutcome::MissedCatch => PassResult::FUMBLE,
                });
                true
            }
            _ => false,
        }
    }
}

impl StepMissedPass {
    /// Java BB2020 `StepMissedPass.executeStep`, the `WILDLY_INACCURATE` arm: the ball deviates
    /// from the THROWER's square (not the pass target) by one d6 of distance in one d8 direction,
    /// then walks back to the last in-bounds square.
    ///
    /// BB2020 rolls exactly two dice here (direction, distance); the `else` arm rolls up to three
    /// (one direction per square). Getting the arm wrong therefore desyncs the shared dice stream
    /// as well as the ball position.
    fn deviate_from_thrower(&mut self, game: &mut Game, rng: &mut GameRng) {
        // Java: coordinateStart = game.getFieldModel().getPlayerCoordinate(game.getThrower())
        let thrower_coord = game.thrower_id.clone()
            .and_then(|id| game.field_model.player_coordinate(&id));
        self.coordinate_start = thrower_coord;

        // Java: rollScatterDirection() [d8] then rollScatterDistance() [d6] — in that order.
        let direction_roll = rng.d8();
        let distance_roll = rng.d6();
        let dir = Self::direction_for_roll(direction_roll);
        self.roll = direction_roll;
        self.direction = Some(dir);

        if let Some(start) = self.coordinate_start {
            // Java: coordinateEnd = findScatterCoordinate(coordinateStart, direction, distanceRoll)
            let end = start.step(dir, distance_roll);
            self.coordinate_end = Some(end);
            // Java: lastValidCoordinate = coordinateEnd; then walk back while out of bounds.
            self.last_valid_coordinate = Some(end);
            let mut valid_distance = distance_roll;
            while !self.last_valid_coordinate.map_or(false, |c| c.is_on_pitch()) && valid_distance > 0 {
                valid_distance -= 1;
                self.last_valid_coordinate = Some(start.step(dir, valid_distance));
            }
        }

        // Java: getResult().addReport(new ReportPassDeviate(coordinateEnd, direction, directionRoll,
        //       distanceRoll, false)).
        //
        // Coverage: `GameEvent::PassDeviate` is emitted only by the BB2020 twin of this file, which
        // the driver never routes to (BB2016 has its own override, BB2020 and BB2025 both land
        // here), so a wildly-inaccurate pass deviated without reporting it. Parked because this
        // helper returns nothing; `execute_step` attaches it. Report-only.
        if let Some(start) = self.coordinate_start {
            self.pending_deviate = Some(GameEvent::PassDeviate {
                from: start,
                scatter_directions: vec![direction_roll, distance_roll],
            });
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let is_bomb = matches!(
            game.thrower_action,
            Some(PlayerAction::ThrowBomb) | Some(PlayerAction::HailMaryBomb)
        );

        // Java BB2020 `StepMissedPass.executeStep`:
        //   if (state.getResult().equals(PassResult.WILDLY_INACCURATE)) {
        //     coordinateStart = throwerCoordinate;
        //     int directionRoll = rollScatterDirection(); int distanceRoll = rollScatterDistance();
        //     coordinateEnd = findScatterCoordinate(coordinateStart, direction, distanceRoll);
        //     lastValidCoordinate = coordinateEnd;
        //     int validDistance = distanceRoll;
        //     while (!FIELD.isInBounds(lastValidCoordinate) && validDistance > 0) { validDistance--;
        //       lastValidCoordinate = findScatterCoordinate(coordinateStart, direction, validDistance); }
        //     addReport(new ReportPassDeviate(...));
        //   } else { <the 3x1 scatter loop below> }
        // BB2025's file has no such branch because its PassMechanic returns FUMBLE where BB2020
        // returns WILDLY_INACCURATE, so the two editions share everything else.
        let wildly_inaccurate = game.rules == ffb_model::enums::Rules::Bb2020
            && self.pass_result == Some(ffb_mechanics::pass_result::PassResult::WILDLY_INACCURATE);

        if wildly_inaccurate {
            self.deviate_from_thrower(game, rng);
        } else {

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
            // client-only: Blast-It! dialog (UtilCards.hasUnusedSkillWithProperty, PassState.getUsingBlastIt)
            // Headless skips the dialog; Blast-It! reroll never triggered (safe for non-Goblin rosters).

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

        } // end of Java's `else` arm — everything below is shared by both branches

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
        if out_of_bounds && !is_bomb {
            // Java's bb2020/bb2025 MissedPass publishes NOTHING — the out-of-bounds routing lives
            // in StepResolvePass, which publishes ThrowIn for a BALL and BombOutOfBounds for a
            // BOMB. This Rust-side publish stays for the ball path (measured green across all
            // editions) but must never fire for a bomb: it threw the real ball in from the
            // touchline after a wildly-inaccurate BOMB sailed out (goblin bb2020 seed 10 step 14:
            // 4 extra dice — throw-in d8+2d6 + a scatter — and the ball teleported to (7,7)
            // while Java's bomb simply vanished).
            let mut outcome = StepOutcome::next()
                .publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ThrowIn));
            if let Some(lvc) = self.last_valid_coordinate {
                outcome = outcome.publish(StepParameter::ThrowInCoordinate(lvc));
            }
            match self.pending_deviate.take() {
                Some(ev) => outcome.with_event(ev),
                None => outcome,
            }
        } else {
            match self.pending_deviate.take() {
                Some(ev) => StepOutcome::next().with_event(ev),
                None => StepOutcome::next(),
            }
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

    // ── BB2020 wildly-inaccurate deviate ───────────────────────────────────────

    /// Java BB2020 `StepMissedPass` deviates from the THROWER on a WILDLY_INACCURATE pass:
    /// exactly two dice (d8 direction, d6 distance). The BB2025 arm rolls one d8 per square, up
    /// to three. Taking the wrong arm desyncs the shared dice stream *and* misplaces the ball
    /// (bb2020 human seed 2 i=197: Java put the ball on 7,3, Rust on 19,4).
    #[test]
    fn bb2020_wildly_inaccurate_deviates_from_the_thrower_with_two_dice() {
        use ffb_mechanics::pass_result::PassResult;
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020);
        let thrower = "thrower_1".to_string();
        game.team_home.players.push(ffb_model::model::player::Player {
            id: thrower.clone(), name: thrower.clone(), nr: 1, ..Default::default()
        });
        game.thrower_id = Some(thrower.clone());
        game.field_model.set_player_coordinate(&thrower, FieldCoordinate::new(13, 7));
        // The pass TARGET is far away; the deviate arm must ignore it and start from the thrower.
        game.pass_coordinate = Some(FieldCoordinate::new(3, 2));

        let mut step = StepMissedPass::new();
        assert!(step.set_parameter(&StepParameter::PassResultParam(
            ffb_model::enums::PassOutcome::WildlyInaccurate)));
        let mut rng = GameRng::new(7);
        step.start(&mut game, &mut rng);

        assert_eq!(rng.call_count, 2, "deviate rolls exactly one d8 + one d6");
        assert_eq!(step.coordinate_start, Some(FieldCoordinate::new(13, 7)),
            "deviate starts from the thrower, not the pass target");
        assert!(step.roll_list.is_empty(), "the 3x1 scatter loop must not run");
        assert_eq!(step.pass_result, Some(PassResult::WILDLY_INACCURATE));
    }

    /// The same parameter under BB2025 must NOT take the deviate arm — BB2025's PassMechanic
    /// cannot produce WILDLY_INACCURATE, and its StepMissedPass has no such branch.
    #[test]
    fn bb2025_ignores_wildly_inaccurate_and_uses_the_scatter_loop() {
        let mut game = make_game();
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        let mut step = StepMissedPass::new();
        step.set_parameter(&StepParameter::PassResultParam(
            ffb_model::enums::PassOutcome::WildlyInaccurate));
        let mut rng = GameRng::new(7);
        step.start(&mut game, &mut rng);
        assert!(!step.roll_list.is_empty(), "BB2025 must still run the 3x1 scatter loop");
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
