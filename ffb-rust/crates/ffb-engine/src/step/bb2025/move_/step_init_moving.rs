use ffb_model::enums::{PlayerAction, SkillId};
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::report::mixed::report_fumblerooskie::ReportFumblerooskie;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepAction, StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.move.StepInitMoving.
///
/// Initialises a move action: decodes the move command, publishes
/// COORDINATE_FROM / COORDINATE_TO / MOVE_STACK, dispatches block/foul/pass/gaze
/// sub-sequences via GOTO_LABEL_ON_END + DISPATCH_PLAYER_ACTION.
///
/// Init params: GOTO_LABEL_ON_END (mandatory), GAZE_VICTIM_ID (optional),
///              MOVE_STACK (optional), BALL_AND_CHAIN_RE_ROLL_SETTING (optional).
///
/// Command dispatch (Move/Block/Foul/Pass/HandOff/ThrowTeamMate/KickTeamMate/Gaze/EndTurn) ported.
/// CLIENT_USE_FUMBLEROOSKIE / CLIENT_USE_SKILL (canAddBlockDie) not yet ported.
/// no-op: UtilServerPlayerMove.isValidMove path validation not ported; agent-submitted paths are trusted.
/// setDodging/setGoingForIt, setTurnStarted, concessionPossible, per-action TurnData flags are wired.
pub struct StepInitMoving {
    /// Java: fGotoLabelOnEnd
    pub goto_label_on_end: String,
    /// Java: fMoveStack
    pub move_stack: Vec<FieldCoordinate>,
    /// Java: fGazeVictimId
    pub gaze_victim_id: Option<String>,
    /// Java: ballAndChainRrSetting
    pub ball_and_chain_rr_setting: Option<String>,
    /// Java: fEndTurn
    pub end_turn: bool,
    /// Java: fEndPlayerAction
    pub end_player_action: bool,
}

impl StepInitMoving {
    pub fn new(goto_label_on_end: String) -> Self {
        Self {
            goto_label_on_end,
            move_stack: Vec::new(),
            gaze_victim_id: None,
            ball_and_chain_rr_setting: None,
            end_turn: false,
            end_player_action: false,
        }
    }
}

impl Step for StepInitMoving {
    fn id(&self) -> StepId { StepId::InitMoving }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_action = game.acting_player.player_action;
        let has_blocked = game.acting_player.has_blocked;
        let has_fouled = game.acting_player.has_fouled;

        match action {
            // Java: CLIENT_MOVE / CLIENT_BLITZ_MOVE — agent provides the path to move through
            // UtilServerPlayerMove.isValidMove + fetchMoveStack not ported; trust agent path
            Action::Move { path } if !path.is_empty() => {
                if self.move_stack.is_empty() {
                    self.move_stack = path.clone();
                }
                return self.execute_step(game, rng);
            }

            // Java: CLIENT_BLOCK → dispatchPlayerAction(BLITZ/KICK_EM_BLITZ)
            // Guard: (BLITZ_MOVE || KICK_EM_BLITZ) && !hasBlocked || PUTRID_REGURGITATION_BLITZ
            Action::Block { .. } => {
                let is_blitz_dispatch = matches!(player_action,
                    Some(PlayerAction::BlitzMove) | Some(PlayerAction::KickEmBlitz)
                    | Some(PlayerAction::PutridRegurgitationBlitz))
                    && (!has_blocked || matches!(player_action, Some(PlayerAction::PutridRegurgitationBlitz)));
                if is_blitz_dispatch {
                    let dispatch = if player_action == Some(PlayerAction::KickEmBlitz) {
                        PlayerAction::KickEmBlitz
                    } else {
                        PlayerAction::Blitz
                    };
                    return self.dispatch_player_action(dispatch)
                        .publish(StepParameter::UsingChainsaw(false));
                }
            }

            // Java: CLIENT_FOUL → dispatchPlayerAction(FOUL)
            // Guard: FOUL_MOVE && !hasFouled
            Action::Foul { .. } => {
                if player_action == Some(PlayerAction::FoulMove) && !has_fouled {
                    return self.dispatch_player_action(PlayerAction::Foul);
                }
            }

            // Java: CLIENT_HAND_OVER → dispatchPlayerAction(HAND_OVER)
            // Guard: HAND_OVER_MOVE || HAND_OVER
            Action::HandOff { receiver_id } => {
                if matches!(player_action, Some(PlayerAction::HandOverMove) | Some(PlayerAction::HandOver)) {
                    // Java records this from CLIENT_HAND_OVER inside StepInitPassing; Rust sees the
                    // command here instead, and without it InitPassing has no thrower and parks with
                    // no prompt (StepOutcome::cont()), stopping the game outright.
                    if let Some(c) = game.field_model.player_coordinate(receiver_id) {
                        game.pass_coordinate = Some(c);
                    }
                    game.thrower_id = game.acting_player.player_id.clone();
                    game.thrower_action = Some(PlayerAction::HandOver);
                    return self
                        .dispatch_player_action(PlayerAction::HandOver)
                        .publish(StepParameter::CatcherId(Some(receiver_id.clone())));
                }
            }

            // Java: CLIENT_PASS → dispatchPlayerAction(PASS or HAIL_MARY_PASS)
            // Guard: PASS_MOVE || PASS → PASS; HAIL_MARY_PASS → HAIL_MARY_PASS
            Action::Pass { coord } => {
                // Same as the hand-over: Java's CLIENT_PASS sets the pass coordinate, derives the
                // catcher from that square and takes the thrower from the acting player.
                let set_thrower = |game: &mut Game| {
                    game.pass_coordinate = Some(*coord);
                    game.thrower_id = game.acting_player.player_id.clone();
                    game.thrower_action = game.acting_player.player_action;
                };
                let catcher = game.field_model.player_at(*coord).cloned();
                match player_action {
                    Some(PlayerAction::PassMove) | Some(PlayerAction::Pass) => {
                        set_thrower(game);
                        game.thrower_action = Some(PlayerAction::Pass);
                        return self
                            .dispatch_player_action(PlayerAction::Pass)
                            .publish(StepParameter::CatcherId(catcher));
                    }
                    Some(PlayerAction::HailMaryPass) => {
                        return self.dispatch_player_action(PlayerAction::HailMaryPass);
                    }
                    _ => {}
                }
            }

            // Java: CLIENT_THROW_TEAM_MATE → dispatchPlayerAction(THROW_TEAM_MATE or KICK_TEAM_MATE)
            // Guard: THROW_TEAM_MATE_MOVE || KICK_TEAM_MATE_MOVE
            Action::ThrowTeamMate { player_id, .. } => {
                if matches!(player_action, Some(PlayerAction::ThrowTeamMateMove)) {
                    let pid = player_id.clone();
                    return self.dispatch_player_action(PlayerAction::ThrowTeamMate)
                        .publish(StepParameter::ThrownPlayerId(Some(pid)));
                }
            }
            Action::KickTeamMate { player_id, .. } => {
                if matches!(player_action, Some(PlayerAction::KickTeamMateMove)) {
                    let pid = player_id.clone();
                    return self.dispatch_player_action(PlayerAction::KickTeamMate)
                        .publish(StepParameter::ThrownPlayerId(Some(pid)));
                }
            }

            // Java: CLIENT_GAZE → fGazeVictimId = victimId, EXECUTE_STEP
            Action::HypnoticGaze { target_id } => {
                self.gaze_victim_id = Some(target_id.clone());
                return self.execute_step(game, rng);
            }

            // Java: CLIENT_ACTING_PLAYER with no playerId (deselect) → fEndPlayerAction = true, EXECUTE_STEP
            Action::EndPlayerAction => {
                // Java: the ParityRunner deselects an empty-plan MOVE at PHASE 2, while
                // INIT_SELECTING is still waiting for the concrete command — StepStandUp has NOT
                // run, hasActed() is false, and changeActingPlayer reverts the charged stand-up
                // to PRONE (still active). Rust's folded flow has already run StandUp
                // (has_moved = true) by the time this prompt exists, so undo its mark when no
                // square was ever taken: the deselect must land in to_none's standing_up→PRONE
                // branch, not the acted()→STANDING+inactive one (chaos bb2025 seed 46 @0 i=75:
                // Java's A9 ends PRONE and still active; Rust ended it Standing).
                // "No square was ever taken" must be an ACTIVATION-level fact (an instance flag
                // misfired: every move round pushes a FRESH InitMoving), and it must NOT be
                // inferred from current_move (a Jump Up stand costs 0, so `current_move <= 3`
                // wrongly unmarked a jumped-up player who then moved 1-3 squares — amazon bb2020
                // 100→59). `acting_player.took_square` is set at the square pop and cleared on
                // player change; the prone gate is Java's own `standingUp || wasProne`.
                let was_prone = game.acting_player.old_player_state
                    .map(|st| st.base() == ffb_model::enums::PS_PRONE)
                    .unwrap_or(false);
                if !game.acting_player.took_square
                    && (game.acting_player.standing_up || was_prone)
                {
                    game.acting_player.has_moved = false;
                }
                self.end_player_action = true;
                return self.execute_step(game, rng);
            }

            // Java: CLIENT_END_TURN → fEndTurn = true, EXECUTE_STEP
            Action::EndTurn => {
                self.end_turn = true;
                return self.execute_step(game, rng);
            }

            // Java: CLIENT_USE_FUMBLEROOSKIE — if (playerAction != null && playerAction.allowsFumblerooskie()
            //       && UtilPlayer.hasBall(game, player)) { setBallMoving(true); addReport(...); setFumblerooskiePending(true); }
            Action::UseSkill { skill_id: SkillId::Fumblerooskie, use_skill: true } => {
                let player_id = game.acting_player.player_id.clone();
                let allows = game.acting_player.player_action
                    .map(|a| a.allows_fumblerooskie())
                    .unwrap_or(false);
                let has_ball = player_id.as_deref()
                    .map(|id| UtilPlayer::has_ball(game, id))
                    .unwrap_or(false);
                if allows && has_ball {
                    game.field_model.ball_moving = true;
                    // client-only: getResult().setSound(SoundId.BOUNCE)
                    game.report_list.add(ReportFumblerooskie::new(player_id, true));
                    game.acting_player.fumblerooskie_pending = true;
                }
                // Java: commandStatus stays UNHANDLED_COMMAND (no explicit assignment in this
                // case), so executeStep() is NOT re-invoked. Mirrored as cont() (no re-run).
                return StepOutcome::cont();
            }

            // Java: CLIENT_USE_SKILL → canAddBlockDie → ReportSkillUse(skill, true, ADD_BLOCK_DIE)
            Action::UseSkill { skill_id, use_skill: true } => {
                if skill_id.properties().contains(&NamedProperties::CAN_ADD_BLOCK_DIE) {
                    let player_id = game.acting_player.player_id.clone();
                    game.report_list.add(ReportSkillUse::new(
                        player_id,
                        *skill_id,
                        true,
                        SkillUse::ADD_BLOCK_DIE,
                    ));
                    // Java: also dispatches to blitz if BlitzMove && !hasBlocked — deferred
                }
            }

            // Java: CLIENT_ACTING_PLAYER with no player_id → fEndPlayerAction = true, EXECUTE_STEP
            // In Rust this could arrive as some kind of "end action" signal — leave as fallthrough
            _ => {}
        }

        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            StepParameter::MoveStack(v) => { self.move_stack = v.clone(); true }
            StepParameter::GazeVictimId(v) => { self.gaze_victim_id = v.clone(); true }
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            StepParameter::BallAndChainRrSetting(v) => { self.ball_and_chain_rr_setting = v.clone(); true }
            _ => false,
        }
    }
}

impl StepInitMoving {
    /// Java: dispatchPlayerAction(pPlayerAction) — publish DISPATCH_PLAYER_ACTION + GOTO_LABEL_ON_END.
    fn dispatch_player_action(&self, action: PlayerAction) -> StepOutcome {
        StepOutcome::goto(&self.goto_label_on_end)
            .publish(StepParameter::DispatchPlayerAction(Some(action)))
    }

    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: if (fEndTurn) → publish END_TURN + CHECK_FORGO, GOTO fGotoLabelOnEnd
        if self.end_turn {
            let label = self.goto_label_on_end.clone();
            return StepOutcome::goto(&label)
                .publish(StepParameter::EndTurn(true))
                .publish(StepParameter::CheckForgo(true));
        }
        // Java: else if (fEndPlayerAction) → publish END_PLAYER_ACTION, GOTO fGotoLabelOnEnd
        if self.end_player_action {
            let label = self.goto_label_on_end.clone();
            return StepOutcome::goto(&label)
                .publish(StepParameter::EndPlayerAction(true));
        }
        // Java: else if (StringTool.isProvided(fGazeVictimId)) → setDefenderId, setPlayerAction(GAZE), NEXT_STEP
        if self.gaze_victim_id.is_some() {
            game.acting_player.player_action = Some(ffb_model::enums::PlayerAction::Gaze);
            game.defender_id = self.gaze_victim_id.clone();
            return StepOutcome::next();
        }
        // Java: publish BALL_AND_CHAIN_RE_ROLL_SETTING
        // Java: if (ArrayTool.isProvided(fMoveStack)) → pop first coord, publish COORDINATE_FROM/TO/MOVE_STACK
        if !self.move_stack.is_empty() {
            let coordinate_to = self.move_stack[0];
            let new_stack: Vec<FieldCoordinate> = self.move_stack[1..].to_vec();
            self.move_stack = new_stack.clone();

            let coordinate_from = game.acting_player.player_id.as_deref()
                .and_then(|id| game.field_model.player_coordinate(id))
                .unwrap_or(FieldCoordinate::new(0, 0));

            if !FieldCoordinateBounds::FIELD.is_in_bounds(coordinate_to) {
                return StepOutcome::cont();
            }
            // Java: MoveSquare moveSquare = game.getFieldModel().getMoveSquare(coordinateTo);
            // Java: actingPlayer.setDodging((moveSquare != null) && moveSquare.isDodging() && !actingPlayer.isJumping());
            // Java: actingPlayer.setGoingForIt((moveSquare != null) && moveSquare.isGoingForIt());
            let move_square = game.field_model.get_move_square(coordinate_to);
            game.acting_player.dodging = move_square
                .map(|ms| ms.is_dodging() && !game.acting_player.jumping)
                .unwrap_or(false);
            game.acting_player.goes_for_it = move_square
                .map(|ms| ms.is_going_for_it())
                .unwrap_or(false);
            game.field_model.target_selection_state.as_mut().map(|t| t.commit());
            game.acting_player.has_moved = true;
            game.acting_player.took_square = true;
            game.turn_data_mut().turn_started = true;
            // Java: per-PlayerAction TurnData flags
            let player_action = game.acting_player.player_action;
            use ffb_model::enums::PlayerAction;
            use ffb_model::model::property::named_properties::NamedProperties;
            match player_action {
                Some(PlayerAction::BlitzMove) | Some(PlayerAction::KickEmBlitz) => {
                    game.turn_data_mut().blitz_used = true;
                }
                Some(PlayerAction::FoulMove) => {
                    let allows_extra = game.acting_player.player_id.as_deref()
                        .and_then(|id| game.player(id))
                        .map(|p| p.has_skill_property(NamedProperties::ALLOWS_ADDITIONAL_FOUL))
                        .unwrap_or(false);
                    if !allows_extra {
                        game.turn_data_mut().foul_used = true;
                    }
                }
                Some(PlayerAction::HandOverMove) => {
                    game.turn_data_mut().hand_over_used = true;
                }
                Some(PlayerAction::PassMove) => {
                    game.turn_data_mut().pass_used = true;
                }
                Some(PlayerAction::ThrowTeamMateMove) => {
                    game.turn_data_mut().ttm_used = true;
                }
                Some(PlayerAction::KickTeamMateMove) => {
                    game.turn_data_mut().ktm_used = true;
                }
                Some(PlayerAction::SecureTheBall) => {
                    game.turn_data_mut().secure_the_ball_used = true;
                }
                Some(PlayerAction::PuntMove) | Some(PlayerAction::Punt) => {
                    game.turn_data_mut().punt_used = true;
                }
                _ => {}
            }
            game.concession_possible = false;

            return StepOutcome {
                action: StepAction::NextStep,
                goto_label: None,
                events: Vec::new(),
                pushes: Vec::new(),
                prompt: None,
                published: vec![
                    StepParameter::BallAndChainRrSetting(self.ball_and_chain_rr_setting.clone()),
                    StepParameter::MoveStack(new_stack),
                    StepParameter::CoordinateFrom(coordinate_from),
                    StepParameter::CoordinateTo(coordinate_to),
                ],
                clear_stack: false, push_self: false
            };
        }
        // A Vampire overcome by Blood Lust (failed the roll this activation) takes NO move: Java's
        // server reaches StepInitMoving with an EMPTY move stack (the client is never asked for a move)
        // and just proceeds — the vampire stays put, rolling only the Bloodlust die, then feeds
        // (turnover if no adjacent thrall). Confirmed via gated Java instrumentation (JMOVE_EXEC
        // suffering=true fMoveStackProvided=false). Rust's harness instead PROMPTS the agent on an
        // empty stack (below), which supplies a path and moves the vampire (seed 1 i=43: prone home_01
        // moved to (11,8)+dodge vs Java staying at (12,7)). Skip straight to END_MOVING so the move
        // steps (Move/GoForIt/MoveDodge) are bypassed and the sequence proceeds to feeding.
        if game.acting_player.suffering_blood_lust {
            // §12: this early-out was written for a plain MOVE, where "no move, then feed" IS the
            // whole activation. On the chain path a BLITZ reaches this step too - its second pass
            // dispatches BLITZ_MOVE - and a blitzer still owes its BLOCK. Ending the player action
            // here killed it: vampire seed 1 in BOTH editions has main reaching StepInitBlocking
            // with the blood-lust flag set and blocking anyway (7 InitBlocking executions to the
            // branch's 6), while the branch never got there.
            //
            // Simply dropping the EndPlayerAction is NOT the fix - the activation then has no
            // terminator at all and the driver loops forever (measured: the engine hangs). The
            // blitzer instead takes the same route the agent's Block answer would have taken,
            // dispatching BLITZ so the block sequence runs and ends the activation itself. The
            // vampire still takes no MOVE, which is the behaviour this guard exists to preserve.
            if matches!(game.acting_player.player_action,
                        Some(PlayerAction::BlitzMove) | Some(PlayerAction::KickEmBlitz))
                && !game.acting_player.has_blocked
                && game.defender_id.is_some()
            {
                let dispatch = if game.acting_player.player_action == Some(PlayerAction::KickEmBlitz) {
                    PlayerAction::KickEmBlitz
                } else {
                    PlayerAction::Blitz
                };
                return self.dispatch_player_action(dispatch);
            }
            let label = self.goto_label_on_end.clone();
            return StepOutcome::goto(&label)
                .publish(StepParameter::EndPlayerAction(true));
        }
        // Empty move stack — compute legal move targets and prompt the agent for a destination.
        // The live driver.rs/step architecture never carried this over from the pre-driver.rs
        // engine.rs (`Step::InitMoving` there did exactly this via the same `legal_move_targets`/
        // `legal_blitz_move_targets` helpers) — without it, `AgentPrompt::Move` was never emitted
        // at all and the driver hung waiting for a client command nothing ever asked for.
        let Some(player_id) = game.acting_player.player_id.clone() else {
            return StepOutcome::next();
        };
        let squares = match game.acting_player.player_action {
            Some(PlayerAction::Blitz) => match game.defender_id.clone() {
                Some(def_id) => crate::legal_actions::legal_blitz_move_targets(game, &player_id, &def_id),
                None => crate::legal_actions::legal_move_targets(game, &player_id),
            },
            _ => crate::legal_actions::legal_move_targets(game, &player_id),
        };
        StepOutcome::cont().with_prompt(ffb_model::prompts::AgentPrompt::Move { player_id, squares })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepParameter;
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    /// §12: a blitzer suffering Blood Lust must still throw its BLOCK. The early-out here was
    /// written for a plain MOVE ("no move, then feed"), but the blitz chain routes a BLITZ through
    /// this step too, and publishing END_PLAYER_ACTION killed the block - vampire was 57/100
    /// (bb2025) and 53/100 (bb2020) until this was fixed.
    ///
    /// Simply dropping the terminator is NOT the fix and must not be re-introduced: with no
    /// END_PLAYER_ACTION the activation never ends and the driver loops forever (measured - the
    /// engine hangs). The blitzer dispatches BLITZ instead, so the block sequence runs and ends
    /// the activation itself.
    #[test]
    fn blood_lust_blitzer_dispatches_block_instead_of_ending_the_action() {
        let mut game = make_game();
        game.acting_player.player_id = Some("h1".into());
        game.acting_player.player_action = Some(PlayerAction::BlitzMove);
        game.acting_player.suffering_blood_lust = true;
        game.defender_id = Some("a1".into());
        let mut step = StepInitMoving::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(out.published.iter().any(|p|
            matches!(p, StepParameter::DispatchPlayerAction(Some(PlayerAction::Blitz)))),
            "a blood-lust blitzer must dispatch BLITZ so the block still happens");
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))),
            "it must NOT end the player action - the block sequence ends the activation");
    }

    /// The plain-MOVE case this guard exists for is unchanged: no dispatch, and the activation
    /// ends here. Without this the fix above could silently widen to every blood-lust move.
    #[test]
    fn blood_lust_plain_move_still_ends_the_player_action() {
        let mut game = make_game();
        game.acting_player.player_id = Some("h1".into());
        game.acting_player.player_action = Some(PlayerAction::Move);
        game.acting_player.suffering_blood_lust = true;
        let mut step = StepInitMoving::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))));
        assert!(!out.published.iter().any(|p|
            matches!(p, StepParameter::DispatchPlayerAction(_))));
    }

    #[test]
    fn end_turn_goes_to_label_with_end_turn_and_check_forgo() {
        let mut game = make_game();
        let mut step = StepInitMoving::new("end".into());
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CheckForgo(true))));
    }

    #[test]
    /// Java parity (chaos bb2025 seed 46 @0 i=75): deselecting a prone mover that stood up but
    /// never took a square must NOT count the stand-up as having acted — the harness deselects
    /// at phase 2 in Java, before StepStandUp runs, so hasActed() is false and the player
    /// reverts to PRONE at changeActingPlayer(null).
    #[test]
    fn deselect_before_any_square_unmarks_the_standup_move() {
        let mut game = ffb_model::model::game::Game::new(
            crate::step::framework::test_team("home", 0),
            crate::step::framework::test_team("away", 0),
            ffb_model::enums::Rules::Bb2025,
        );
        game.acting_player.player_id = Some("home_01".into());
        game.acting_player.standing_up = true;
        game.acting_player.has_moved = true; // StandUp's mark
        game.acting_player.took_square = false; // no square ever popped this activation
        let mut step = StepInitMoving::new(String::new());
        let _ = step.handle_command(&Action::EndPlayerAction, &mut game, &mut GameRng::new(0));
        assert!(!game.acting_player.has_moved,
            "the never-moved stand-up must be unmarked so the deselect lands in the PRONE branch");

        // A player who stood AND moved keeps the mark (he has acted) — a Jump Up stand costs 0,
        // so this must key on took_square, never on current_move (amazon bb2020 100→59).
        game.acting_player.standing_up = true;
        game.acting_player.has_moved = true;
        game.acting_player.took_square = true;
        let mut step2 = StepInitMoving::new(String::new());
        let _ = step2.handle_command(&Action::EndPlayerAction, &mut game, &mut GameRng::new(0));
        assert!(game.acting_player.has_moved, "a stood-up player who took squares HAS acted");
    }

    fn end_player_action_goes_to_label_with_end_player_action() {
        let mut game = make_game();
        let mut step = StepInitMoving::new("end".into());
        step.end_player_action = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))));
    }

    #[test]
    fn empty_move_stack_returns_continue() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        let mut step = StepInitMoving::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
        assert!(matches!(out.prompt, Some(ffb_model::prompts::AgentPrompt::Move { .. })));
    }

    #[test]
    fn move_stack_pops_first_coord_and_publishes_coordinate_to() {
        let mut game = make_game();
        let mut step = StepInitMoving::new("end".into());
        let sq1 = FieldCoordinate::new(5, 3);
        let sq2 = FieldCoordinate::new(6, 3);
        step.move_stack = vec![sq1, sq2];
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        let has_to = out.published.iter().any(|p| matches!(p, StepParameter::CoordinateTo(c) if *c == sq1));
        assert!(has_to, "CoordinateTo(sq1) must be published");
        let remaining = out.published.iter().find_map(|p| {
            if let StepParameter::MoveStack(v) = p { Some(v.clone()) } else { None }
        }).unwrap();
        assert_eq!(remaining, vec![sq2]);
    }

    #[test]
    fn set_parameter_end_turn_accepted() {
        let mut step = StepInitMoving::new("end".into());
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn gaze_victim_causes_next_step_and_sets_gaze_action() {
        let mut game = make_game();
        let mut step = StepInitMoving::new("end".into());
        step.gaze_victim_id = Some("p1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.acting_player.player_action, Some(ffb_model::enums::PlayerAction::Gaze));
    }

    // ── handle_command dispatch tests ────────────────────────────────────────

    #[test]
    fn handle_command_move_sets_move_stack_and_executes() {
        let mut game = make_game();
        let mut step = StepInitMoving::new("end".into());
        let path = vec![FieldCoordinate::new(5, 5), FieldCoordinate::new(6, 5)];
        let action = crate::action::Action::Move { path };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        // execute_step processes move_stack → NextStep
        assert_eq!(out.action, StepAction::NextStep);
        let has_coord_to = out.published.iter().any(|p| matches!(p, StepParameter::CoordinateTo(_)));
        assert!(has_coord_to, "CoordinateTo should be published after Move command");
    }

    #[test]
    fn handle_command_end_turn_goes_to_label() {
        let mut game = make_game();
        let mut step = StepInitMoving::new("end".into());
        let out = step.handle_command(&crate::action::Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn handle_command_block_on_blitz_move_dispatches_blitz() {
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::BlitzMove);
        game.acting_player.has_blocked = false;
        let mut step = StepInitMoving::new("end".into());
        let action = crate::action::Action::Block { defender_id: "def1".into() };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end"));
        let has_dispatch = out.published.iter().any(|p| matches!(p, StepParameter::DispatchPlayerAction(Some(PlayerAction::Blitz))));
        assert!(has_dispatch, "should publish DispatchPlayerAction(Blitz)");
    }

    #[test]
    fn handle_command_block_not_dispatched_when_already_blocked() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::BlitzMove);
        game.acting_player.has_blocked = true;
        let mut step = StepInitMoving::new("end".into());
        let action = crate::action::Action::Block { defender_id: "def1".into() };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        // Falls through to execute_step → cont (empty move_stack)
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn handle_command_foul_on_foul_move_dispatches_foul() {
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::FoulMove);
        game.acting_player.has_fouled = false;
        let mut step = StepInitMoving::new("end".into());
        let action = crate::action::Action::Foul { target_id: "def1".into() };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        let has_dispatch = out.published.iter().any(|p| matches!(p, StepParameter::DispatchPlayerAction(Some(PlayerAction::Foul))));
        assert!(has_dispatch, "should publish DispatchPlayerAction(Foul)");
    }

    #[test]
    fn handle_command_pass_on_pass_move_dispatches_pass() {
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::PassMove);
        let mut step = StepInitMoving::new("end".into());
        let action = crate::action::Action::Pass { coord: FieldCoordinate::new(10, 10) };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        let has_dispatch = out.published.iter().any(|p| matches!(p, StepParameter::DispatchPlayerAction(Some(PlayerAction::Pass))));
        assert!(has_dispatch, "should publish DispatchPlayerAction(Pass)");
    }

    #[test]
    fn handle_command_gaze_sets_victim_and_executes() {
        let mut game = make_game();
        let mut step = StepInitMoving::new("end".into());
        let action = crate::action::Action::HypnoticGaze { target_id: "victim1".into() };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        // execute_step with gaze_victim_id set → NextStep
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(step.gaze_victim_id.as_deref(), Some("victim1"));
    }

    #[test]
    fn move_to_dodge_square_sets_dodging_flag() {
        use ffb_model::types::MoveSquare;
        let mut game = make_game();
        let dodge_coord = FieldCoordinate::new(5, 5);
        // Register as a dodging move square (minimum_roll_dodge > 0)
        game.field_model.add_move_square(MoveSquare::new(dodge_coord, 3, 0));
        let mut step = StepInitMoving::new("end".into());
        step.move_stack = vec![dodge_coord];
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.acting_player.dodging, "actingPlayer.setDodging should be true");
        assert!(!game.acting_player.goes_for_it, "setGoingForIt should be false");
    }

    #[test]
    fn move_to_gfi_square_sets_goes_for_it_flag() {
        use ffb_model::types::MoveSquare;
        let mut game = make_game();
        let gfi_coord = FieldCoordinate::new(6, 5);
        game.field_model.add_move_square(MoveSquare::new(gfi_coord, 0, 2));
        let mut step = StepInitMoving::new("end".into());
        step.move_stack = vec![gfi_coord];
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.acting_player.dodging, "setDodging should be false for GFI square");
        assert!(game.acting_player.goes_for_it, "setGoingForIt should be true");
    }

    #[test]
    fn move_to_unknown_square_clears_dodging_and_goes_for_it() {
        let mut game = make_game();
        // No move square registered for this coord
        let coord = FieldCoordinate::new(7, 5);
        game.acting_player.dodging = true;
        game.acting_player.goes_for_it = true;
        let mut step = StepInitMoving::new("end".into());
        step.move_stack = vec![coord];
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.acting_player.dodging, "unknown square clears dodging");
        assert!(!game.acting_player.goes_for_it, "unknown square clears goes_for_it");
    }

    #[test]
    fn dodge_square_not_set_when_jumping() {
        use ffb_model::types::MoveSquare;
        let mut game = make_game();
        let dodge_coord = FieldCoordinate::new(5, 5);
        game.field_model.add_move_square(MoveSquare::new(dodge_coord, 3, 0));
        game.acting_player.jumping = true;
        let mut step = StepInitMoving::new("end".into());
        step.move_stack = vec![dodge_coord];
        step.start(&mut game, &mut GameRng::new(0));
        // Java: setDodging(moveSquare.isDodging() && !actingPlayer.isJumping()) → false when jumping
        assert!(!game.acting_player.dodging, "dodging suppressed while jumping");
    }

    #[test]
    fn use_fumblerooskie_adds_report_and_sets_pending_when_moving_and_carrying_ball() {
        // Java: playerAction.allowsFumblerooskie() (isMoving()) && UtilPlayer.hasBall(game, player)
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Move);
        let coord = FieldCoordinate::new(5, 5);
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.ball_coordinate = Some(coord);
        game.field_model.ball_in_play = true;
        game.field_model.ball_moving = false;
        let mut step = StepInitMoving::new("end".into());
        let action = crate::action::Action::UseSkill { skill_id: SkillId::Fumblerooskie, use_skill: true };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(
            game.report_list.has_report(ReportId::FUMBLEROOSKIE),
            "expected FUMBLEROOSKIE report when moving and carrying the ball"
        );
        assert!(game.field_model.ball_moving, "ball should be set moving");
        assert!(game.acting_player.is_fumblerooskie_pending(), "fumblerooskie_pending should be set");
    }

    #[test]
    fn use_fumblerooskie_is_noop_without_ball() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Move);
        let mut step = StepInitMoving::new("end".into());
        let action = crate::action::Action::UseSkill { skill_id: SkillId::Fumblerooskie, use_skill: true };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(!game.report_list.has_report(ReportId::FUMBLEROOSKIE));
        assert!(!game.acting_player.is_fumblerooskie_pending());
    }

    #[test]
    fn use_skill_non_block_die_does_not_add_report() {
        // Dodge does NOT have CAN_ADD_BLOCK_DIE -- no report should be added
        use ffb_model::enums::SkillId;
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        let mut step = StepInitMoving::new("end".into());
        let action = crate::action::Action::UseSkill { skill_id: SkillId::Dodge, use_skill: true };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(!game.report_list.has_report(ReportId::SKILL_USE), "non-block-die UseSkill should not add report");
    }

    #[test]
    fn use_skill_false_does_not_add_report() {
        // use_skill: false -- no report should be added regardless of skill
        use ffb_model::enums::SkillId;
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        let mut step = StepInitMoving::new("end".into());
        let action = crate::action::Action::UseSkill { skill_id: SkillId::Block, use_skill: false };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(!game.report_list.has_report(ReportId::SKILL_USE), "use_skill: false should not add report");
    }
}
