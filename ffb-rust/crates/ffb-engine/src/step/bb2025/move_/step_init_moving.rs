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
        // A failed Blood Lust roll owes ONE move-stack discard, and it is owed against the
        // activation's FIRST command — the one Java's client sends at `StepInitSelecting` phase 2,
        // BEFORE its `StepBloodLust` runs (see `ActingPlayer::blood_lust_discards_move_stack`).
        // Whatever that first command is, it settles the debt: Java's
        // `publishParameter(new StepParameter(MOVE_STACK, null))` throws away a move stack if one
        // is there and is a no-op otherwise. A blitzer whose first command is a BLOCK therefore
        // keeps its post-block movement (bb2025 seed 17 i=3: Rust discarded that walk and ended the
        // activation while Java walked it).
        let blood_lust_discards_stack =
            std::mem::take(&mut game.acting_player.blood_lust_discards_move_stack);

        match action {
            // Java: CLIENT_MOVE / CLIENT_BLITZ_MOVE — agent provides the path to move through
            // UtilServerPlayerMove.isValidMove + fetchMoveStack not ported; trust agent path
            Action::Move { path } if !path.is_empty() => {
                // The deferred blood-lust discard. Java's
                // `BloodLustBehaviour.handleExecuteStepHook` FAILURE branch is
                //     step.publishParameter(new StepParameter(StepParameterKey.MOVE_STACK, null));
                //     getResult().setNextAction(StepAction.GOTO_LABEL, state.goToLabelOnFailure);
                // and the stack it discards is the one the client delivered at `StepInitSelecting`
                // phase 2. Rust asks for that path HERE instead, one step after the roll, so the
                // discard lands on this first delivery: the vampire takes no square, the sequence
                // jumps to END_MOVING, and `StepEndMoving` pushes a fresh Move sequence whose
                // empty-stack prompt is Java's second look — where the agent's plan is already
                // spent and `MoveReplay` answers END_PLAYER_ACTION (or, for a PICKUP plan,
                // re-plans and walks, which is what bb2020 seed 8 does).
                if blood_lust_discards_stack {
                    let label = self.goto_label_on_end.clone();
                    return StepOutcome::goto(&label)
                        .publish(StepParameter::MoveStack(Vec::new()));
                }
                // A PINNED (rooted / chomped) player cannot take the first step of a MOVE.
                //
                // Java routes the FIRST move command of a `*_MOVE` activation through
                // `StepInitSelecting`, whose CLIENT_MOVE arm sets
                // `fDispatchPlayerAction = PlayerAction.MOVE` — it does NOT re-use the declared
                // HAND_OVER_MOVE / PASS_MOVE / THROW_TEAM_MATE_MOVE. `StepEndSelecting`
                // (bb2025/shared, `dispatchPlayerAction`) then runs
                // `case MOVE: if (playerState.isPinned()) { endGenerator.pushSequence(endParams); break; }`
                // BEFORE the fall-through that pushes the Move sequence. So a rooted Treeman that
                // declares a give and is told to walk simply ENDS ITS ACTIVATION: it never moves,
                // `StepInitMoving.executeStep` never runs and therefore never sets
                // `turnData.setHandOverUsed(true)`, and because `actingPlayer.hasActed()` stays
                // false `UtilActingPlayer.changeActingPlayer` puts the player back to STANDING with
                // its ACTIVE bit intact.
                //
                // Rust pushes the Move sequence at declaration time and raises the move prompt from
                // here, so this arm is where that first command lands; `!has_moved` is what makes it
                // the command Java hands to StepInitSelecting rather than to StepInitMoving (which
                // has no pinned guard). Without it the rooted Treeman consumed the team hand-over,
                // lost its ACTIVE bit and went on to fire the give terminal — halfling bb2020 seed 4
                // idx 19, where Java's whole activation is a no-op (`post_hash == pre_hash`).
                if !game.acting_player.has_moved
                    && game.acting_player.player_id.as_deref()
                        .and_then(|id| game.field_model.player_state(id))
                        .map(|ps| ps.is_pinned())
                        .unwrap_or(false)
                {
                    self.end_player_action = true;
                    return self.execute_step(game, rng);
                }
                // Java (all three `StepInitMoving`s, CLIENT_MOVE / CLIENT_BLITZ_MOVE):
                //   publishParameter(new StepParameter(MOVE_START, fetchFromSquare(moveCommand, ...)));
                // Rust published MOVE_START only from `StepInitSelecting`, and the heuristic's
                // move answers land HERE, not there - so `StepJump` and `StepGoForIt` both read a
                // MISSING move start. That made `JumpContext.from` collapse onto the LANDING
                // square, so the jump's tackle-zone modifier was counted at the destination alone
                // and a leap out of two tackle zones rolled its bare agility (slann bb2020 seed 17:
                // Rust target 4, Java 5). Unreachable before the agent could declare a jump -
                // `StepGoForIt`'s only other reader gates on `jumping` too.
                let move_start = game.acting_player.player_id.clone()
                    .and_then(|pid| game.field_model.player_coordinate(&pid));
                if self.move_stack.is_empty() {
                    self.move_stack = path.clone();
                }
                let out = self.execute_step(game, rng);
                return match move_start {
                    Some(c) => out.publish(StepParameter::MoveStart(c)),
                    None => out,
                };
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

            // Java: CLIENT_ACTING_PLAYER with a playerId and `isJumping() == true`, re-sent while
            // the player is already acting. `UtilServerSteps.changePlayerAction` forwards it to
            // `UtilActingPlayer.changeActingPlayer`, which (same player, so `changed == false`)
            // only runs `actingPlayer.setJumping(jumping)`, and then refreshes the move squares
            // through `updateMoveSquares(gameState, isJumping())` — the distance-2 jump squares.
            // The PlayerAction is re-asserted unchanged, exactly as the client re-sends the one
            // already declared; sending MOVE here would downgrade a BLITZ_MOVE.
            Action::DeclareJump => {
                if let Some(pid) = game.acting_player.player_id.clone() {
                    if let Some(pa) = game.acting_player.player_action {
                        crate::step::util_server_steps::change_player_action(game, &pid, pa, true);
                    }
                }
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
        // Java's `StepInitMoving.executeStep` has NO blood-lust branch at all: with an empty
        // MOVE_STACK it sets no next action, so the step PARKS and the client is asked
        // (`ParityRunner case INIT_MOVING` -> `sendMoveAction` -> `MoveReplay`). The discard a
        // failed roll owes is applied to the first delivered path in `handle_command`, which is
        // where Rust receives what Java received at `StepInitSelecting` phase 2.
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

    /// A game with one on-pitch home player, rooted, activated with the given action.
    fn rooted_mover(action: PlayerAction, has_moved: bool) -> (Game, String) {
        use ffb_model::enums::{PlayerState, PS_MOVING};
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        let pid = "home_01".to_string();
        game.team_home.players.push(ffb_model::model::player::Player {
            id: pid.clone(), name: pid.clone(), nr: 1, position_id: "pos".into(),
            movement: 2, strength: 6, agility: 5, passing: 11, armour: 11,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(&pid, FieldCoordinate::new(12, 7));
        game.field_model.set_player_state(&pid,
            PlayerState::new(PS_MOVING).change_active(true).change_rooted(true));
        game.home_playing = true;
        game.acting_player.player_id = Some(pid.clone());
        game.acting_player.player_action = Some(action);
        game.acting_player.has_moved = has_moved;
        (game, pid)
    }

    /// Java `bb2025/shared/StepEndSelecting.dispatchPlayerAction`:
    /// ```java
    /// case MOVE:
    ///   if (game.getFieldModel().getPlayerState(actingPlayer.getPlayer()).isPinned()) {
    ///     endGenerator.pushSequence(endParams);
    ///     break;
    ///   }
    ///   // fall through … HAND_OVER_MOVE … → moveGenerator.pushSequence(...)
    /// ```
    /// The FIRST move command of a `*_MOVE` activation is dispatched as `PlayerAction.MOVE` by
    /// `StepInitSelecting`'s CLIENT_MOVE arm, so a ROOTED player told to walk ends its activation:
    /// no square, no `setHandOverUsed(true)` (that lives in `StepInitMoving.executeStep`, which
    /// never runs), and `hasActed()` stays false so `changeActingPlayer` keeps the ACTIVE bit.
    #[test]
    fn a_pinned_players_first_move_ends_the_activation() {
        let (mut game, _pid) = rooted_mover(PlayerAction::HandOverMove, false);
        let mut step = StepInitMoving::new("end".into());
        let out = step.handle_command(
            &Action::Move { path: vec![ffb_model::types::FieldCoordinate::new(11, 8)] },
            &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))),
            "Java ends the player action instead of pushing the Move sequence");
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::CoordinateTo(_))),
            "a rooted player never takes the square");
        assert!(!game.turn_data_home.hand_over_used,
            "the declared team hand-over is NOT consumed — StepInitMoving.executeStep never runs");
        assert!(!game.acting_player.has_moved,
            "hasActed() must stay false so the player keeps its ACTIVE bit");
    }

    /// The guard is only for the command Java routes through `StepInitSelecting`. A move command
    /// that arrives mid-activation lands in `StepInitMoving`, whose CLIENT_MOVE arm has NO pinned
    /// check — `StepMove` is what then skips the actual step.
    #[test]
    fn a_pinned_player_that_has_already_moved_is_not_guarded() {
        let (mut game, _pid) = rooted_mover(PlayerAction::HandOverMove, true);
        let mut step = StepInitMoving::new("end".into());
        let out = step.handle_command(
            &Action::Move { path: vec![ffb_model::types::FieldCoordinate::new(11, 8)] },
            &mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CoordinateTo(_))),
            "the continuation move is processed exactly as before");
    }

    /// Java's `bb2025/move/StepInitMoving` has NO blood-lust branch: with an empty MOVE_STACK
    /// `executeStep` sets no next action, so the step PARKS and the client is asked
    /// (`ParityRunner case INIT_MOVING` -> `sendMoveAction` -> `MoveReplay`). A blood-lust blitzer
    /// must therefore be OFFERED its move, not auto-dispatched into a block by the engine: the
    /// agent's `MoveReplay` is what decides between the run-up and `FIRE_TERMINAL`, and Java's
    /// engine only dispatches BLITZ from the CLIENT_BLOCK arm of `handleCommand`.
    ///
    /// This test replaces one that asserted the auto-dispatch, i.e. that encoded the invented
    /// early-out this step used to carry (vampire ITER3).
    #[test]
    fn blood_lust_blitzer_is_offered_its_move_not_auto_dispatched() {
        let mut game = make_game();
        game.acting_player.player_id = Some("h1".into());
        game.acting_player.player_action = Some(PlayerAction::BlitzMove);
        game.acting_player.suffering_blood_lust = true;
        game.defender_id = Some("a1".into());
        let mut step = StepInitMoving::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.prompt, Some(ffb_model::prompts::AgentPrompt::Move { .. })),
            "Java parks and asks; the engine must not decide for the agent: {:?}", out.prompt);
        assert!(!out.published.iter().any(|p|
            matches!(p, StepParameter::DispatchPlayerAction(Some(PlayerAction::Blitz)))),
            "the BLITZ dispatch belongs to the CLIENT_BLOCK arm, not to executeStep");
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))));
    }

    /// The POST-BLOCK leg of the same blitz. Java has no early-out in `StepInitMoving`: an empty
    /// MOVE_STACK leaves `executeStep` with no next action, so the step parks and the client is
    /// asked — ParityRunner answers `case INIT_MOVING` with `sendMoveAction`. Measured on bb2025
    /// vampire seed 3 step 54: Java carried the blood-lust blitzer four squares past its block
    /// (JSTATE INIT_MOVING cm=1 -> cm=5); ending the player action here diverged the post-hash on
    /// an otherwise identical declaration.
    #[test]
    fn blood_lust_blitzer_is_offered_its_move_after_the_block() {
        let mut game = make_game();
        game.acting_player.player_id = Some("h1".into());
        game.acting_player.player_action = Some(PlayerAction::BlitzMove);
        game.acting_player.suffering_blood_lust = true;
        game.acting_player.has_blocked = true;
        game.defender_id = Some("a1".into());
        let mut step = StepInitMoving::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.prompt, Some(ffb_model::prompts::AgentPrompt::Move { .. })),
            "the post-block leg must park and ask, like Java: {:?}", out.prompt);
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))),
            "it must not end the activation before the move is offered");
    }

    /// A plain MOVE whose blood lust failed AFTER the first square was already popped. Java at
    /// bb2020 vampire seed 7 i=30: `JSTATE ... step=BLOOD_LUST ... moved=true cm=0` then two
    /// INIT_MOVING looks carrying the thrower to cm=4. `hasMoved` is what separates it from i=29,
    /// where the same failure with `moved=false` ended the activation with no move at all.
    #[test]
    fn blood_lust_move_already_under_way_is_offered_the_rest_of_its_move() {
        let mut game = make_game();
        game.acting_player.player_id = Some("h1".into());
        game.acting_player.player_action = Some(PlayerAction::Move);
        game.acting_player.suffering_blood_lust = true;
        game.acting_player.has_moved = true;
        let mut step = StepInitMoving::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.prompt, Some(ffb_model::prompts::AgentPrompt::Move { .. })),
            "an activation already under way must park and ask: {:?}", out.prompt);
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))));
    }

    /// The plain-MOVE blood-lust case, ported from where Java actually decides it.
    ///
    /// `BloodLustBehaviour.handleExecuteStepHook` (bb2025), FAILURE branch:
    /// ```java
    /// step.publishParameter(new StepParameter(StepParameterKey.MOVE_STACK, null));
    /// getResult().setNextAction(StepAction.GOTO_LABEL, state.goToLabelOnFailure);   // END_MOVING
    /// ```
    /// The stack it throws away is the path the client delivered at `StepInitSelecting` phase 2,
    /// one step BEFORE the roll. Rust asks for that path here instead, so the discard is deferred
    /// onto the first command of the activation (`blood_lust_discards_move_stack`): the vampire
    /// takes NO square, the sequence goes to END_MOVING, and `StepEndMoving` pushes a fresh Move
    /// sequence whose prompt is Java's second look.
    #[test]
    fn blood_lusts_first_delivered_path_is_discarded_and_takes_no_square() {
        let mut game = make_game();
        game.acting_player.player_id = Some("h1".into());
        game.acting_player.player_action = Some(PlayerAction::Move);
        game.acting_player.suffering_blood_lust = true;
        game.acting_player.blood_lust_discards_move_stack = true;
        let mut step = StepInitMoving::new("end".into());
        let out = step.handle_command(
            &Action::Move { path: vec![ffb_model::types::FieldCoordinate::new(11, 8)] },
            &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::MoveStack(v) if v.is_empty())),
            "Java publishes MOVE_STACK = null");
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::CoordinateTo(_))),
            "the discarded path must not move the vampire");
        assert!(!game.acting_player.has_moved);
        assert!(!game.acting_player.blood_lust_discards_move_stack,
            "the discard is owed exactly once, like Java's single BLOOD_LUST failure");
    }

    /// The SECOND look (Java's `case INIT_MOVING`) is an ordinary move prompt again: once the
    /// discard is spent, a path the agent re-plans is walked. bb2020 seed 8 i=26 is exactly this —
    /// a PICKUP plan re-plans at the second look and the blood-lusting vampire walks three squares.
    #[test]
    fn blood_lusts_second_delivered_path_is_walked() {
        let mut game = make_game();
        game.acting_player.player_id = Some("h1".into());
        game.acting_player.player_action = Some(PlayerAction::Move);
        game.acting_player.suffering_blood_lust = true;
        game.acting_player.blood_lust_discards_move_stack = false;
        let mut step = StepInitMoving::new("end".into());
        let out = step.handle_command(
            &Action::Move { path: vec![ffb_model::types::FieldCoordinate::new(11, 8)] },
            &mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CoordinateTo(_))),
            "with the debt settled the move is processed exactly as any other");
    }

    /// The debt is owed against the activation's FIRST command, whatever it is: Java's
    /// `MOVE_STACK, null` discards a stack if one is there and is a no-op otherwise. A blitzer
    /// whose first command is a BLOCK keeps its post-block movement (bb2025 seed 17 i=3, where
    /// discarding that walk ended the activation while Java walked it).
    #[test]
    fn a_first_command_that_is_not_a_move_still_settles_the_discard() {
        let mut game = make_game();
        game.acting_player.player_id = Some("h1".into());
        game.acting_player.player_action = Some(PlayerAction::BlitzMove);
        game.acting_player.suffering_blood_lust = true;
        game.acting_player.blood_lust_discards_move_stack = true;
        let mut step = StepInitMoving::new("end".into());
        let _ = step.handle_command(
            &Action::Block { defender_id: "a1".into() }, &mut game, &mut GameRng::new(0));
        assert!(!game.acting_player.blood_lust_discards_move_stack,
            "the BLOCK settles the debt; Java had no stack to throw away either");
        let out = step.handle_command(
            &Action::Move { path: vec![ffb_model::types::FieldCoordinate::new(11, 8)] },
            &mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CoordinateTo(_))),
            "the post-block movement must NOT be discarded");
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
    fn declare_jump_sets_jumping_and_keeps_the_declared_action() {
        // BACKLOG E12, written FROM Java's `StepInitSelecting`/`StepInitMoving`
        // CLIENT_ACTING_PLAYER arm: a command naming the player who is ALREADY acting reaches
        // `UtilServerSteps.changePlayerAction(this, id, playerAction, isJumping())`, which sets
        // `ActingPlayer.jumping` and re-runs `updateMoveSquares(gameState, true)`. It must NOT
        // change the declared action — the client re-sends the one already in force, so a
        // BLITZ_MOVE stays a BLITZ_MOVE.
        use ffb_model::enums::{PlayerState, PS_MOVING};
        let mut game = make_game();
        let pid = "home_01".to_string();
        game.team_home.players.push(ffb_model::model::player::Player {
            id: pid.clone(), name: pid.clone(), nr: 1, position_id: "pos".into(),
            movement: 7, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(&pid, FieldCoordinate::new(12, 7));
        game.field_model.set_player_state(&pid, PlayerState::new(PS_MOVING).change_active(true));
        game.home_playing = true;
        game.acting_player.player_id = Some(pid.clone());
        game.acting_player.player_action = Some(PlayerAction::BlitzMove);

        let mut step = StepInitMoving::new("end".into());
        let out = step.handle_command(&Action::DeclareJump, &mut game, &mut GameRng::new(0));
        assert!(game.acting_player.jumping, "the declaration sets the jumping flag");
        assert_eq!(game.acting_player.player_action, Some(PlayerAction::BlitzMove),
            "the declared action is re-asserted, not downgraded to MOVE");
        assert!(matches!(out.prompt, Some(ffb_model::prompts::AgentPrompt::Move { .. })),
            "the step waits again with a fresh move prompt, now over the jump squares");
    }

    #[test]
    fn a_move_publishes_the_square_it_started_from() {
        // The other half of the same Java line, in `StepInitMoving`'s CLIENT_MOVE arm:
        // `publishParameter(new StepParameter(MOVE_START, fetchFromSquare(moveCommand, ...)))`.
        // `StepJump` builds its `JumpContext` from that square, so without it the jump's
        // tackle-zone modifier is counted at the LANDING square alone.
        use ffb_model::enums::{PlayerState, PS_MOVING};
        let mut game = make_game();
        let pid = "home_01".to_string();
        game.team_home.players.push(ffb_model::model::player::Player {
            id: pid.clone(), name: pid.clone(), nr: 1, position_id: "pos".into(),
            movement: 7, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        });
        let from = FieldCoordinate::new(12, 7);
        game.field_model.set_player_coordinate(&pid, from);
        game.field_model.set_player_state(&pid, PlayerState::new(PS_MOVING).change_active(true));
        game.home_playing = true;
        game.acting_player.player_id = Some(pid.clone());
        game.acting_player.player_action = Some(PlayerAction::Move);

        let mut step = StepInitMoving::new("end".into());
        let out = step.handle_command(
            &Action::Move { path: vec![FieldCoordinate::new(13, 7)] },
            &mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::MoveStart(c) if *c == from)),
            "MOVE_START must carry the pre-move square, got {:?}", out.published);
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
