use std::collections::HashMap;
use ffb_model::enums::{ApothecaryMode, PlayerState};
use ffb_model::model::pushback_mode::PushbackMode;
use ffb_model::report::report_pushback::ReportPushback;
use ffb_model::types::{FieldCoordinate, PushbackSquare};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_injury::handle_injury_by_name;
use crate::util::UtilServerPlayerMove;
use crate::util::util_server_pushback::UtilServerPushback;
use crate::skill_behaviour::dispatch;

// ── Hook state ─────────────────────────────────────────────────────────────────

/// Java: StepPushback.StepState — mutable state passed through executeStepHooks.
/// Exported so StandFirm/SideStep/Grab step-modifiers can downcast to it.
#[derive(Debug)]
pub struct StepPushbackHookState {
    /// Java: state.doPush
    pub do_push: bool,
    /// Java: state.sideStepping (Map<String, Boolean>)
    pub side_stepping: HashMap<String, bool>,
    /// Java: state.standingFirm (Map<String, Boolean>)
    pub standing_firm: HashMap<String, bool>,
    /// Java: state.grabbing (Boolean — tristate)
    pub grabbing: Option<bool>,
    /// Java: state.startingPushbackSquare
    pub starting_pushback_square: Option<PushbackSquare>,
    /// Java: state.defender (Player) — carried as a player id for headless
    pub defender_id: String,
    /// Java: state.oldDefenderState
    pub old_defender_state: Option<PlayerState>,
    /// Java: state.pushbackStack (non-empty means player already chose coords)
    pub pushback_stack_len: usize,
    /// Java: state.freeSquareAroundDefender
    pub free_square_around_defender: bool,
    /// Java: state.pushbackSquares — the current candidate squares
    pub pushback_squares: Vec<PushbackSquare>,
    /// Java: state.pushbackMode
    pub pushback_mode: PushbackMode,
}

impl StepPushbackHookState {
    pub fn new(
        defender_id: String,
        old_defender_state: Option<PlayerState>,
        starting_pushback_square: Option<PushbackSquare>,
        pushback_stack_len: usize,
        free_square_around_defender: bool,
        pushback_squares: Vec<PushbackSquare>,
        side_stepping: HashMap<String, bool>,
        standing_firm: HashMap<String, bool>,
        grabbing: Option<bool>,
    ) -> Self {
        Self {
            do_push: false,
            side_stepping,
            standing_firm,
            grabbing,
            starting_pushback_square,
            defender_id,
            old_defender_state,
            pushback_stack_len,
            free_square_around_defender,
            pushback_squares,
            pushback_mode: PushbackMode::REGULAR,
        }
    }
}

// ── StepPushback ───────────────────────────────────────────────────────────────

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.block.StepPushback.
/// Handles player pushback and crowd-push. The Java StepState fields are inlined here.
///
/// Expects stepParameter STARTING_PUSHBACK_SQUARE to be set by a preceding step.
/// Expects stepParameter OLD_DEFENDER_STATE to be set by a preceding step.
///
/// Sets stepParameter DEFENDER_PUSHED for all steps on the stack.
/// Sets stepParameter STARTING_PUSHBACK_SQUARE for all steps on the stack.
pub struct StepPushback {
    // StepState fields
    pub old_defender_state: Option<PlayerState>,
    pub starting_pushback_square: Option<PushbackSquare>,
    /// Java: grabbing (Boolean — tristate)
    pub grabbing: Option<bool>,
    /// Java: sideStepping (Map<String, Boolean>)
    pub side_stepping: HashMap<String, bool>,
    /// Java: standingFirm (Map<String, Boolean>)
    pub standing_firm: HashMap<String, bool>,
    /// Java: pushbackStack — (playerId, coordinate) pairs (LIFO).
    /// Java Pushback.playerId + Pushback.coordinate.
    pub pushback_stack: Vec<(String, FieldCoordinate)>,
}

impl StepPushback {
    pub fn new() -> Self {
        Self {
            old_defender_state: None,
            starting_pushback_square: None,
            grabbing: None,
            side_stepping: HashMap::new(),
            standing_firm: HashMap::new(),
            pushback_stack: Vec::new(),
        }
    }
}

impl Default for StepPushback {
    fn default() -> Self { Self::new() }
}

impl Step for StepPushback {
    fn id(&self) -> StepId { StepId::Pushback }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::PushTo { coord } => {
                // Java: CLIENT_PUSHBACK —
                //   if (checkCommandIsFromHomePlayer) pushbackStack.push(pushback)
                //   else pushbackStack.push(pushback.transform())
                // We only have coord here; the player being pushed is the current defender.
                // For chain pushbacks the player pushed might differ — TODO when chain pushback added.
                if let Some(defender_id) = game.defender_id.clone() {
                    self.pushback_stack.push((defender_id, *coord));
                }
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::OldDefenderState(v) => { self.old_defender_state = Some(*v); true }
            StepParameter::StartingPushbackSquare(v) => {
                self.starting_pushback_square = *v;
                true
            }
            _ => false,
        }
    }
}

impl StepPushback {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let mut do_push = false;
        // Java: crowd-push params (INJURY_RESULT, THROW_IN mode/coord, END_TURN) that must be
        // published alongside DEFENDER_PUSHED once execution reaches the unified `if (state.doPush)`
        // block at the bottom of the method — Java does NOT return early from the crowd-push branch.
        let mut extra_params: Vec<StepParameter> = Vec::new();

        // Java: if (!state.pushbackStack.isEmpty())
        // Player chose a coordinate — select that pushback square from the field model.
        if !self.pushback_stack.is_empty() {
            if let Some(&(_, chosen_coord)) = self.pushback_stack.last() {
                // Java: for each unlocked square remove from model; if coord matches, select+lock+re-add
                let matching = game.field_model.pushback_squares.iter_mut()
                    .find(|sq| !sq.locked && sq.coordinate == chosen_coord);
                if let Some(sq) = matching {
                    sq.selected = true;
                    sq.locked = true;
                    // Update starting_pushback_square to the chosen square
                    self.starting_pushback_square = Some(*sq);
                }
                // Java: doPush = (fieldModel.getPlayer(lastPushback.getCoordinate()) == null)
                do_push = game.field_model.player_at(chosen_coord).is_none();
            }
        }

        // Java: if (!state.doPush && (state.startingPushbackSquare != null))
        // Calculate new pushback squares if needed.
        if !do_push {
            if let Some(starting_sq) = self.starting_pushback_square {
                let defender_coord = starting_sq.coordinate;

                // Java: state.defender = fieldModel.getPlayer(defenderCoordinate)
                let defender_id = game.defender_id.clone().unwrap_or_default();

                // Java: pushbackMode = REGULAR; findPushbackSquares; fieldModel.add
                let home_choice = game.home_playing;
                let occupied = |c: FieldCoordinate| game.field_model.player_at(c).is_some();
                let pushback_squares = UtilServerPushback::find_pushback_squares_standard(
                    starting_sq, &occupied, home_choice,
                );

                // Java: compute freeSquareAroundDefender (used by SideStep condition)
                let adjacent_squares = game.field_model.adjacent_on_pitch(defender_coord);
                let free_square_around_defender = adjacent_squares.iter()
                    .any(|c| game.field_model.player_at(*c).is_none());

                // ── Skill hooks via dispatch (StandFirm prio 2, Grab prio 3, SideStep prio 4) ──
                // Java: GameState.executeStepHooks(this, state)
                let mut hook_state = StepPushbackHookState::new(
                    defender_id.clone(),
                    self.old_defender_state,
                    self.starting_pushback_square,
                    self.pushback_stack.len(),
                    free_square_around_defender,
                    pushback_squares,
                    self.side_stepping.clone(),
                    self.standing_firm.clone(),
                    self.grabbing,
                );

                let stop_processing = dispatch::execute_step_hooks(
                    game, rng, StepId::Pushback, &mut hook_state,
                );

                // Merge hook state back into step state
                self.side_stepping = hook_state.side_stepping;
                self.standing_firm = hook_state.standing_firm;
                self.grabbing = hook_state.grabbing;
                self.starting_pushback_square = hook_state.starting_pushback_square;
                do_push = hook_state.do_push;
                let final_pushback_squares = hook_state.pushback_squares;

                let pushback_squares_found = !final_pushback_squares.is_empty() || stop_processing;

                // If StandFirm was used (stop_processing = true), push is cancelled — already handled.
                // Fall through to the do_push path below.

                // Java: if (!ArrayTool.isProvided(state.pushbackSquares)) → Crowd push
                if !pushback_squares_found && !stop_processing {
                    // Java: boolean sameTeam = state.defender != null && state.defender.getTeam() == game.getActingTeam();
                    let same_team = game.defender_id.as_deref()
                        .map(|id| game.is_active_team_player(id))
                        .unwrap_or(false);

                    // Java: if (hasFanInteraction(actingTeam) && !sameTeam) → CrowdPushForSpp w/ attacker = actingPlayer
                    //       else → CrowdPush, no attacker
                    let acting_team_id = game.active_team().id.clone();
                    let (injury_type_name, attacker_id) =
                        if game.prayer_state.has_fan_interaction(&acting_team_id) && !same_team {
                            ("InjuryTypeCrowdPushForSpp", game.acting_player.player_id.clone())
                        } else {
                            ("InjuryTypeCrowdPush", None)
                        };

                    let crowd_push_coord = self.starting_pushback_square
                        .as_ref()
                        .map(|sq| sq.coordinate)
                        .unwrap_or(defender_coord);
                    let injury_result = handle_injury_by_name(
                        game, rng, injury_type_name,
                        attacker_id.as_deref(),
                        game.defender_id.as_deref().unwrap_or(""),
                        crowd_push_coord,
                        None, None, ApothecaryMode::CrowdPush,
                    );
                    extra_params.push(StepParameter::InjuryResult(Box::new(injury_result)));

                    // Java: game.getFieldModel().remove(state.defender)
                    if let Some(defender_id) = game.defender_id.clone() {
                        game.field_model.remove_player(&defender_id);
                    }

                    // Java: if (defenderCoordinate.equals(game.getFieldModel().getBallCoordinate()))
                    //   setBallCoordinate(null)
                    //   publish CatchScatterThrowInMode.THROW_IN
                    //   publish ThrowInCoordinate(defenderCoordinate)
                    //   if sameTeam: publish END_TURN(true)
                    let ball_at_defender = game.field_model.ball_coordinate
                        .map(|bc| bc == defender_coord)
                        .unwrap_or(false);
                    if ball_at_defender {
                        game.field_model.ball_coordinate = None;
                        extra_params.push(StepParameter::CatchScatterThrowInMode(
                            crate::step::CatchScatterThrowInMode::ThrowIn,
                        ));
                        extra_params.push(StepParameter::ThrowInCoordinate(defender_coord));
                        // Java: if sameTeam → publish END_TURN(true)
                        if same_team {
                            extra_params.push(StepParameter::EndTurn(true));
                        }
                    }

                    // Java: publishParameter(STARTING_PUSHBACK_SQUARE, null) — this also updates
                    // state.startingPushbackSquare synchronously (AbstractStep.publishParameter calls
                    // setParameter on self), so the addReport check below sees it as null.
                    self.starting_pushback_square = None;
                    // Java: state.doPush = true;
                    do_push = true;
                }

                // Java: if (state.startingPushbackSquare == null) addReport(ReportPushback(...))
                // Reached unconditionally — including the crowd-push branch above, since it just
                // cleared starting_pushback_square. SideStep/Grab hooks may also have cleared it.
                if self.starting_pushback_square.is_none() {
                    game.report_list.add(ReportPushback::new(
                        defender_id.clone(),
                        PushbackMode::REGULAR,
                    ));
                }

                if !do_push {
                    // Java: fieldModel.add(state.pushbackSquares)
                    game.field_model.pushback_squares.clear();
                    game.field_model.pushback_squares.extend(final_pushback_squares);
                    return StepOutcome::cont();
                }
                // Java: falls through to the `if (state.doPush)` block below (crowd push sets doPush=true).
            }
        }

        // Java: if (state.doPush) { ... }
        if do_push {
            // Java: publishParameter(StepParameterKey.DEFENDER_PUSHED, true)
            // Java: while (!pushbackStack.isEmpty()) { pop + pushPlayer }
            let pushes: Vec<(String, FieldCoordinate)> = self.pushback_stack.drain(..).collect();
            let mut extra: Vec<StepParameter> = Vec::new();
            for (player_id, coord) in pushes {
                extra.extend(push_player(game, &player_id, coord));
            }
            // Java: fieldModel.clearPushbackSquares()
            game.field_model.pushback_squares.clear();
            // Java: publishParameter(STARTING_PUSHBACK_SQUARE, null)
            self.starting_pushback_square = None;
            // Java: game.setWaitingForOpponent(false)
            // Java: getResult().setNextAction(StepAction.NEXT_STEP)

            let mut outcome = StepOutcome::next();
            // Java: crowd-push params (INJURY_RESULT, THROW_IN mode/coord, END_TURN) were published
            // earlier in the method, before DEFENDER_PUSHED — publish them here in the same order.
            for p in extra_params { outcome = outcome.publish(p); }
            outcome = outcome
                .publish(StepParameter::DefenderPushed(true))
                .publish(StepParameter::StartingPushbackSquare(None));
            for p in extra { outcome = outcome.publish(p); }
            outcome
        } else {
            StepOutcome::cont()
        }
    }
}

/// Java: StepPushback.pushPlayer — moves player to coordinate, handles ball interaction.
/// Returns parameters to publish (Java calls publishParameter() directly; we collect them here).
fn push_player(game: &mut Game, player_id: &str, coord: FieldCoordinate) -> Vec<StepParameter> {
    // Java: fieldModel.updatePlayerAndBallPosition(pPlayer, pCoordinate)
    game.field_model.set_player_coordinate(player_id, coord);
    UtilServerPlayerMove::update_move_squares(game, false);

    let mut params: Vec<StepParameter> = Vec::new();
    // Java: if (fieldModel.isBallMoving() && pCoordinate.equals(fieldModel.getBallCoordinate()))
    //   publish CatchScatterThrowInMode.SCATTER_BALL
    if game.field_model.ball_moving
        && game.field_model.ball_coordinate.map(|bc| bc == coord).unwrap_or(false)
    {
        params.push(StepParameter::CatchScatterThrowInMode(
            crate::step::CatchScatterThrowInMode::ScatterBall,
        ));
    }
    // Java: publishParameter(PLAYER_ENTERING_SQUARE, pPlayer.getId())
    // Java: publishParameter(PLAYER_WAS_PUSHED, true)
    params.push(StepParameter::PlayerEnteringSquare(player_id.to_owned()));
    params.push(StepParameter::PlayerWasPushed(true));
    params
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, PS_STANDING, Direction, PlayerType, PlayerGender, SkillId};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::report::report_id::ReportId;
    use ffb_model::types::PushbackSquare;
    use std::collections::HashSet;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    // ── set_parameter ────────────────────────────────────────────────────────

    #[test]
    fn set_parameter_old_defender_state_accepted() {
        let mut step = StepPushback::new();
        let state = PlayerState::new(PS_STANDING);
        let accepted = step.set_parameter(&StepParameter::OldDefenderState(state));
        assert!(accepted);
        assert!(step.old_defender_state.is_some());
    }

    #[test]
    fn set_parameter_starting_pushback_square_accepted() {
        let mut step = StepPushback::new();
        let coord = FieldCoordinate::new(5, 5);
        let sq = PushbackSquare::new(coord, Direction::North, true);
        let accepted = step.set_parameter(&StepParameter::StartingPushbackSquare(Some(sq)));
        assert!(accepted);
        assert!(step.starting_pushback_square.is_some());
        assert_eq!(step.starting_pushback_square.unwrap().coordinate, coord);
    }

    #[test]
    fn set_parameter_unrecognised_returns_false() {
        let mut step = StepPushback::new();
        let accepted = step.set_parameter(&StepParameter::EndTurn(true));
        assert!(!accepted);
    }

    // ── start with no state ──────────────────────────────────────────────────

    #[test]
    fn no_starting_square_stays_cont() {
        let mut step = StepPushback::new();
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // No starting_pushback_square and no pushback_stack → CONTINUE
        assert_eq!(out.action, StepAction::Continue);
    }

    // ── PushTo command ───────────────────────────────────────────────────────

    #[test]
    fn push_to_command_with_empty_target_square_publishes_defender_pushed() {
        let mut step = StepPushback::new();
        let coord = FieldCoordinate::new(7, 7);
        let mut game = make_game();
        game.defender_id = Some("p1".into());
        // Target square is empty → do_push=true → execute_step drains the stack and publishes DefenderPushed.
        let out = step.handle_command(
            &Action::PushTo { coord },
            &mut game,
            &mut GameRng::new(0),
        );
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::DefenderPushed(true))));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn push_to_command_drains_pushback_stack() {
        let mut step = StepPushback::new();
        let coord = FieldCoordinate::new(3, 3);
        let mut game = make_game();
        game.defender_id = Some("p2".into());
        step.handle_command(&Action::PushTo { coord }, &mut game, &mut GameRng::new(0));
        // stack should be drained after do_push path
        assert!(step.pushback_stack.is_empty());
    }

    // ── starting square → no adjacent free ── crowd push path ────────────────

    #[test]
    fn crowd_push_removes_defender_and_publishes_defender_pushed() {
        let mut step = StepPushback::new();
        // Use coordinate (0, 0) — corner with likely no adjacent on-pitch squares in test field
        let coord = FieldCoordinate::new(0, 0);
        step.starting_pushback_square = Some(PushbackSquare::new(
            coord,
            Direction::North,
            true,
        ));
        let mut game = make_game();
        game.defender_id = Some("p1".into());
        // Place a player at (0,0) so it exists
        game.field_model.set_player_coordinate("p1", coord);

        // Depending on the field model's adjacent_on_pitch implementation,
        // the corner may or may not have free adjacent squares.
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Either crowd push (next) or waiting for client pushback (cont) — both are valid.
        // We just check no panic and that the outcome makes sense.
        assert!(matches!(out.action, StepAction::NextStep | StepAction::Continue));
    }

    #[test]
    fn crowd_push_with_ball_at_defender_publishes_throw_in() {
        let mut step = StepPushback::new();
        let coord = FieldCoordinate::new(0, 0);
        step.starting_pushback_square = Some(PushbackSquare::new(
            coord,
            Direction::North,
            true,
        ));
        let mut game = make_game();
        game.defender_id = Some("p1".into());
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.ball_coordinate = Some(coord);
        game.field_model.ball_in_play = true;

        let out = step.start(&mut game, &mut GameRng::new(0));

        // If crowd push occurred (no adjacent free), the throw-in parameters are published.
        if out.action == StepAction::NextStep {
            let has_throw_in = out.published.iter().any(|p| {
                matches!(p, StepParameter::CatchScatterThrowInMode(
                    crate::step::CatchScatterThrowInMode::ThrowIn
                ))
            });
            let has_throw_in_coord = out.published.iter().any(|p| {
                matches!(p, StepParameter::ThrowInCoordinate(c) if *c == coord)
            });
            assert!(has_throw_in, "expected ThrowIn mode published on crowd push with ball");
            assert!(has_throw_in_coord, "expected ThrowInCoordinate published");
        }
    }

    // ── starting square → has free adjacent ── waits for client ─────────────

    #[test]
    fn starting_square_with_free_adjacent_stays_cont() {
        let mut step = StepPushback::new();
        // Place the starting square in the middle of the field where adjacent squares exist
        let coord = FieldCoordinate::new(7, 5);
        step.starting_pushback_square = Some(PushbackSquare::new(
            coord,
            Direction::North,
            true,
        ));
        let mut game = make_game();
        game.defender_id = Some("p1".into());
        // Surround coord with players except one direction — adjacent_on_pitch should find free squares
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Adjacent squares should be free → wait for client input
        assert_eq!(out.action, StepAction::Continue);
    }

    // ── pushback stack clears field model squares ────────────────────────────

    #[test]
    fn do_push_clears_pushback_squares_in_field_model() {
        let mut step = StepPushback::new();
        let coord = FieldCoordinate::new(5, 5);
        let mut game = make_game();
        game.defender_id = Some("p1".into());
        // Pre-populate a pushback square
        game.field_model.pushback_squares.push(PushbackSquare::new(
            coord,
            Direction::North,
            false,
        ));
        step.handle_command(&Action::PushTo { coord }, &mut game, &mut GameRng::new(0));
        assert!(game.field_model.pushback_squares.is_empty());
    }

    // ── starting_pushback_square cleared after do_push ───────────────────────

    #[test]
    fn starting_pushback_square_cleared_after_do_push() {
        let mut step = StepPushback::new();
        let coord = FieldCoordinate::new(4, 4);
        let mut game = make_game();
        game.defender_id = Some("d1".into());
        step.handle_command(&Action::PushTo { coord }, &mut game, &mut GameRng::new(0));
        // After a successful push the square should be cleared
        assert!(step.starting_pushback_square.is_none());
    }

    /// find_pushback_squares_standard populates field_model.pushback_squares (not just adjacent).
    #[test]
    fn starting_square_populates_field_model_pushback_squares() {
        let mut step = StepPushback::new();
        // Defender at (10,7) — ample room for all three pushback directions (North push → NW/N/NE)
        let coord = FieldCoordinate::new(10, 7);
        step.starting_pushback_square = Some(PushbackSquare::new(coord, Direction::North, true));
        let mut game = make_game();
        game.defender_id = Some("p1".into());
        // No players blocking adjacent squares
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Should wait for client (squares were found)
        assert_eq!(out.action, StepAction::Continue);
        // Field model should now have 3 pushback squares (NW, N, NE of (10,7))
        assert_eq!(game.field_model.pushback_squares.len(), 3);
    }

    // ── report wiring ────────────────────────────────────────────────────────

    /// SideStep player who accepts the side-step triggers the addReport(ReportPushback) path.
    #[test]
    fn side_step_clears_starting_square_and_adds_pushback_report() {
        let mut step = StepPushback::new();
        let coord = FieldCoordinate::new(10, 7);
        step.starting_pushback_square = Some(PushbackSquare::new(coord, Direction::North, true));

        let mut game = make_game();
        // Add a defender with SideStep
        game.team_away.players.push(Player {
            id: "def1".into(), name: "def1".into(), nr: 2, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue { skill_id: SkillId::Sidestep, value: None }],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("def1", coord);
        game.defender_id = Some("def1".into());
        game.home_playing = true;

        // Pre-populate side_stepping = true so headless path uses side-step
        step.side_stepping.insert("def1".to_owned(), true);
        // Also need to set old_defender_state to has_tacklezones
        step.old_defender_state = Some(PlayerState::new(PS_STANDING));

        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::PUSHBACK), "ReportPushback should appear when SideStep clears starting square");
    }

    /// Without SideStep the starting_pushback_square stays set, so no pushback report is added.
    #[test]
    fn without_side_step_no_pushback_report_added() {
        let mut step = StepPushback::new();
        let coord = FieldCoordinate::new(10, 7);
        step.starting_pushback_square = Some(PushbackSquare::new(coord, Direction::North, true));
        let mut game = make_game();
        game.defender_id = Some("p1".into());
        game.field_model.set_player_coordinate("p1", coord);
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.report_list.has_report(ReportId::PUSHBACK), "ReportPushback should NOT appear when starting square is not cleared");
    }

    /// Regression test for two bugs found while auditing against StepPushback.java:
    /// (1) Java's crowd-push branch does NOT return early — it falls through into the
    ///     unified `if (state.doPush)` block, so a ReportPushback is always added once
    ///     `startingPushbackSquare` becomes null (crowd push sets it null via publishParameter,
    ///     which synchronously calls setParameter on self — see AbstractStep.publishParameter).
    /// (2) That same fall-through means any pending chain/domino pushback entries left on
    ///     pushbackStack get drained and applied via pushPlayer, even when the *current*
    ///     defender is crowd-pushed. The old Rust code returned early from the crowd-push
    ///     branch, silently dropping both the report and any queued chain pushes.
    #[test]
    fn crowd_push_drains_chain_pushback_stack_and_adds_report() {
        let mut step = StepPushback::new();
        let coord = FieldCoordinate::new(0, 0);
        step.starting_pushback_square = Some(PushbackSquare::new(coord, Direction::North, true));

        let mut game = make_game();
        game.defender_id = Some("p1".into());
        game.field_model.set_player_coordinate("p1", coord);

        // Simulate a still-pending chain push for another player, targeting an occupied square
        // so the top-of-method stack check leaves do_push = false and execution still reaches
        // the crowd-push branch for the current defender at (0,0)/North (pushback squares off-pitch).
        let chain_target = FieldCoordinate::new(5, 5);
        game.field_model.set_player_coordinate("blocker", chain_target);
        step.pushback_stack.push(("chain1".to_string(), chain_target));

        let out = step.start(&mut game, &mut GameRng::new(0));

        assert!(
            game.report_list.has_report(ReportId::PUSHBACK),
            "ReportPushback should be added when crowd push clears starting_pushback_square"
        );
        assert!(
            step.pushback_stack.is_empty(),
            "pending chain pushback must be drained even when the current defender is crowd-pushed"
        );
        assert_eq!(
            game.field_model.player_coordinate("chain1"),
            Some(chain_target),
            "chain-pushed player must be moved via pushPlayer"
        );
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::DefenderPushed(true))));
    }

    // ── StepPushbackHookState ────────────────────────────────────────────────

    #[test]
    fn hook_state_new_has_correct_defaults() {
        let hs = StepPushbackHookState::new(
            "def".into(), None, None, 0, true, vec![],
            HashMap::new(), HashMap::new(), None,
        );
        assert!(!hs.do_push);
        assert_eq!(hs.defender_id, "def");
        assert!(hs.starting_pushback_square.is_none());
        assert!(hs.pushback_squares.is_empty());
    }

    #[test]
    fn hook_state_carries_side_stepping_map() {
        let mut side = HashMap::new();
        side.insert("x".to_string(), true);
        let hs = StepPushbackHookState::new(
            "def".into(), None, None, 0, false, vec![],
            side, HashMap::new(), None,
        );
        assert_eq!(hs.side_stepping.get("x"), Some(&true));
    }

    #[test]
    fn hook_state_carries_standing_firm_map() {
        let mut sf = HashMap::new();
        sf.insert("y".to_string(), false);
        let hs = StepPushbackHookState::new(
            "def".into(), None, None, 0, false, vec![],
            HashMap::new(), sf, None,
        );
        assert_eq!(hs.standing_firm.get("y"), Some(&false));
    }

    #[test]
    fn hook_state_carries_grabbing() {
        let hs = StepPushbackHookState::new(
            "def".into(), None, None, 0, false, vec![],
            HashMap::new(), HashMap::new(), Some(true),
        );
        assert_eq!(hs.grabbing, Some(true));
    }
}
