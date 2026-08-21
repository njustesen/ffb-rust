use ffb_model::enums::PlayerAction;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::{Action, PlayerActionChoice};
use ffb_model::events::GameEvent;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_steps::change_player_action;
use crate::util::ServerUtilBlock;
use crate::util::UtilServerPlayerMove;

const MINIMUM_MOVE_TO_STAND_UP: i32 = 3;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.move.StepInitSelecting.
///
/// Initialises the select sequence. Waits for a client command; on receipt,
/// publishes the relevant parameters and GOTOs the end label so EndSelecting
/// can route to the correct action sequence.
///
/// Init params: GOTO_LABEL_ON_END (mandatory), UPDATE_PERSISTENCE (mandatory).
///
/// On start: if fUpdatePersistence → clear flag (queueDbUpdate TODO).
///
/// On executeStep:
/// - timeout || fEndTurn → publish END_TURN + GOTO_LABEL
/// - fEndPlayerAction → publish END_PLAYER_ACTION + GOTO_LABEL
/// - fDispatchPlayerAction set → publish DISPATCH_PLAYER_ACTION + GOTO_LABEL
///   (unless standingUp → NEXT_STEP)
/// - else → prepareStandingUp, then NEXT_STEP if REMOVE_CONFUSION/STAND_UP/STAND_UP_BLITZ
///
/// no-op: gameCache.queueDbUpdate — headless engine has no DB layer (confirmed intentional).
/// REMOVE_CONFUSION / STAND_UP / STAND_UP_BLITZ → NEXT_STEP path implemented.
/// no-op: game.isTimeoutEnforced() — headless engine has no turn timer; always treated as false.
pub struct StepInitSelecting {
    /// Java: fGotoLabelOnEnd (init param)
    pub goto_label_on_end: String,
    /// Java: fDispatchPlayerAction
    pub dispatch_player_action: Option<PlayerAction>,
    /// Java: fEndTurn
    pub end_turn: bool,
    /// Java: fEndPlayerAction
    pub end_player_action: bool,
    /// Java: fUpdatePersistence (transient)
    pub update_persistence: bool,
    /// Activation to report on the way out of `execute_step`.
    ///
    /// The BB2025 twin emits `GameEvent::PlayerAction` inline, but this arm has a dozen return
    /// paths (folded block/foul/pass/TTM targets, deselects), so the event is parked here and
    /// attached at the single `execute_step` exit instead. Without it BB2016 reported ZERO
    /// activations across 2,900 coverage games while plainly fouling and blocking — the whole
    /// edition's action coverage was unmeasurable. Purely a report: no state, no dice.
    pending_activation: Option<(String, PlayerAction)>,
}

impl StepInitSelecting {
    pub fn new(goto_label_on_end: String) -> Self {
        Self {
            goto_label_on_end,
            dispatch_player_action: None,
            end_turn: false,
            end_player_action: false,
            update_persistence: false,
            pending_activation: None,
        }
    }
}

impl Default for StepInitSelecting {
    fn default() -> Self { Self::new(String::new()) }
}

impl Step for StepInitSelecting {
    fn id(&self) -> StepId { StepId::InitSelecting }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: if (fUpdatePersistence) { fUpdatePersistence=false; gameCache.queueDbUpdate(...); }
        // no-op: headless engine has no DB layer; gameCache.queueDbUpdate is skipped
        self.update_persistence = false;
        // Java start() waits for a client CLIENT_ACTING_PLAYER. The Rust engine models that "wait"
        // as an AgentPrompt (same bridge every bb2025 step uses — CLAUDE.md Engine output channels),
        // so the agent can drive the bb2016 two-command activation. Mirror bb2025 StepInitSelecting:
        // record the previous activation as "acted" (Java UtilActingPlayer.changeActingPlayer /
        // hasActed()) so the eligible list shrinks, clear it, then emit the activation prompt.
        if game.acting_player.player_id.is_some() {
            let acted = game.acting_player.has_moved
                || game.acting_player.has_fouled
                || game.acting_player.has_blocked
                || game.acting_player.has_passed
                || game.acting_player.has_triggered_effect
                || game.acting_player.forgone;
            if acted {
                if let Some(pid) = game.acting_player.player_id.clone() {
                    let td = if game.home_playing { &mut game.turn_data_home } else { &mut game.turn_data_away };
                    if !td.acted_player_ids.contains(&pid) { td.acted_player_ids.push(pid); }
                }
            }
            game.acting_player.clear();
        }
        let eligible = crate::legal_actions::eligible_players_for_activation(game);
        StepOutcome::cont()
            .with_prompt(ffb_model::prompts::AgentPrompt::ActivatePlayer { eligible_players: eligible })
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_action = game.acting_player.player_action;
        let acting_pid = game.acting_player.player_id.clone().unwrap_or_default();

        match action {
            Action::Move { path } if !path.is_empty() => {
                // Java: CLIENT_MOVE → dispatchPlayerAction(MOVE) + publish MOVE_STACK
                self.dispatch_player_action = Some(PlayerAction::Move);
                let out = self.execute_step(game, rng);
                return out.publish(StepParameter::MoveStack(path.clone()));
            }
            Action::Foul { target_id } => {
                if game.turn_data().foul_used {
                    return StepOutcome::cont();
                }
                change_player_action(game, &acting_pid, PlayerAction::Foul, false);
                self.dispatch_player_action = Some(PlayerAction::Foul);
                return self.execute_step(game, rng)
                    .publish(StepParameter::FoulDefenderId(target_id.clone()));
            }
            Action::Block { defender_id } => {
                self.dispatch_player_action = Some(PlayerAction::Block);
                return self.execute_step(game, rng)
                    .publish(StepParameter::BlockDefenderId(defender_id.clone()));
            }
            Action::HypnoticGaze { target_id } => {
                // Java: CLIENT_GAZE → changePlayerAction(GAZE), dispatch
                change_player_action(game, &acting_pid, PlayerAction::Gaze, false);
                self.dispatch_player_action = Some(PlayerAction::Gaze);
                return self.execute_step(game, rng)
                    .publish(StepParameter::GazeVictimId(Some(target_id.clone())));
            }
            Action::Pass { coord } => {
                // Java: passAllowed = !isPassUsed || (action == THROW_BOMB || HAIL_MARY_BOMB)
                let is_bomb_action = matches!(player_action,
                    Some(PlayerAction::ThrowBomb) | Some(PlayerAction::HailMaryBomb));
                if game.turn_data().pass_used && !is_bomb_action {
                    return StepOutcome::cont();
                }
                // Java: if (game.isHomePlaying()) { publish(coord) } else { publish(coord.transform()) }
                let target = if game.home_playing { *coord } else { coord.transform() };
                let dispatch_action = if player_action == Some(PlayerAction::HailMaryPass) {
                    PlayerAction::HailMaryPass
                } else {
                    change_player_action(game, &acting_pid, PlayerAction::Pass, false);
                    PlayerAction::Pass
                };
                self.dispatch_player_action = Some(dispatch_action);
                return self.execute_step(game, rng)
                    .publish(StepParameter::TargetCoordinate(target));
            }
            Action::HandOff { receiver_id } => {
                if game.turn_data().hand_over_used {
                    return StepOutcome::cont();
                }
                let coord_opt = game.field_model.player_coordinate(receiver_id);
                change_player_action(game, &acting_pid, PlayerAction::HandOver, false);
                self.dispatch_player_action = Some(PlayerAction::HandOver);
                let out = self.execute_step(game, rng);
                if let Some(coord) = coord_opt {
                    return out.publish(StepParameter::TargetCoordinate(coord));
                }
                return out;
            }
            Action::ThrowTeamMate { player_id: thrown_id, coord } => {
                if game.turn_data().pass_used {
                    return StepOutcome::cont();
                }
                change_player_action(game, &acting_pid, PlayerAction::ThrowTeamMate, false);
                self.dispatch_player_action = Some(PlayerAction::ThrowTeamMate);
                // Java: if (game.isHomePlaying()) { publish(coord) } else { publish(coord.transform()) }
                let target = if game.home_playing { *coord } else { coord.transform() };
                return self.execute_step(game, rng)
                    .publish(StepParameter::TargetCoordinate(target))
                    .publish(StepParameter::ThrownPlayerId(Some(thrown_id.clone())));
            }
            Action::KickTeamMate { player_id: kicked_id, coord: _ } => {
                if game.turn_data().blitz_used {
                    return StepOutcome::cont();
                }
                change_player_action(game, &acting_pid, PlayerAction::KickTeamMate, false);
                self.dispatch_player_action = Some(PlayerAction::KickTeamMate);
                return self.execute_step(game, rng)
                    .publish(StepParameter::KickedPlayerId(Some(kicked_id.clone())));
            }
            Action::EndTurn => {
                self.end_turn = true;
                return self.execute_step(game, rng);
            }
            // Java: CLIENT_ACTING_PLAYER — changePlayerAction then executeStep (no dispatchPlayerAction)
            // This path fires for STAND_UP / STAND_UP_BLITZ / REMOVE_CONFUSION → NEXT_STEP in executeStep
            Action::ActivatePlayer { player_id, player_action, block_defender_id } => {
                let pa = pac_to_player_action(*player_action);
                if !game.is_active_team_player(player_id) {
                    self.end_player_action = true;
                    return self.execute_step(game, rng);
                }
                change_player_action(game, player_id, pa, false);
                // Report the activation (see `pending_activation`). Parked rather than returned so
                // every folded-target and deselect path below reports identically.
                self.pending_activation = Some((player_id.clone(), pa));
                // BB2016 two-command activation, folded on the RUST harness side: the parity agent
                // supplies the BLOCK/BLITZ/FOUL target folded into ActivatePlayer (identical RNG to
                // bb2025 — player, action, target picked at activation). Dispatch it here so the
                // bb2016 block/blitz/foul sequences run without needing a separate agent command,
                // keeping the Rust agent unchanged. Java drives this as a real 2nd client command
                // (CLIENT_BLOCK/CLIENT_FOUL) but produces identical state + dice. A MOVE has no folded
                // target → falls through to execute_step which emits the Move dispatch prompt (the
                // agent then supplies the path). A no-target block/blitz also falls through.
                if let Some(def) = block_defender_id {
                    match pa {
                        PlayerAction::Block | PlayerAction::Blitz => {
                            self.dispatch_player_action = Some(pa);
                            return self.execute_step(game, rng)
                                .publish(StepParameter::BlockDefenderId(def.clone()));
                        }
                        PlayerAction::Foul => {
                            self.dispatch_player_action = Some(PlayerAction::Foul);
                            return self.execute_step(game, rng)
                                .publish(StepParameter::FoulDefenderId(def.clone()));
                        }
                        // PASS / HAND-OVER: the folded agent supplies the RECEIVER (a player id) in
                        // block_defender_id (chosen from legal_pass_receivers / legal_handoff_receivers,
                        // 1 actionRng — mirroring ParityRunner.sendPassAction which picks a teammate
                        // COORD). Java drives CLIENT_PASS(coord); here we resolve the receiver's
                        // absolute coordinate and dispatch a Pass with TARGET_COORDINATE, exactly like
                        // the Action::Pass command arm. Without this arm a Pass fell through to a bare
                        // cont() and the bb2016 pass sequence never started (amazon seed1 i=201 stall).
                        // THROW_BOMB rides this arm too: ParityRunner routes it to the same
                        // sendPassAction, so it also arrives carrying a receiver id and must
                        // publish a TARGET_COORDINATE. Without it StepInitPassing parked with an
                        // unset thrower and no prompt, and the parity loop ends the game silently.
                        PlayerAction::Pass | PlayerAction::HandOver | PlayerAction::ThrowBomb
                        | PlayerAction::HailMaryPass => {
                            let coord_opt = game.field_model.player_coordinate(def);
                            self.dispatch_player_action = Some(pa);
                            let out = self.execute_step(game, rng);
                            if let Some(coord) = coord_opt {
                                return out.publish(StepParameter::TargetCoordinate(coord));
                            }
                            return out;
                        }
                        // THROW / KICK TEAM-MATE: the folded agent supplies the THROWN/KICKED teammate
                        // (a player id) in block_defender_id (chosen from legal_throw/kick_team_mate_targets,
                        // 1 actionRng — mirroring ParityRunner.sendThrowTeamMateAction). The TARGET SQUARE
                        // is chosen later, on the ThrowTeamMateTarget prompt (StepInitThrowTeamMate emits
                        // it). Dispatch here (goto END_SELECTING with DISPATCH_PLAYER_ACTION so
                        // StepEndSelecting pushes the TTM/KTM sequence), publishing the thrown/kicked
                        // player id — exactly like the Action::ThrowTeamMate/KickTeamMate command arms.
                        // Without this arm a folded TTM fell through to a bare execute_step that emitted
                        // NO prompt (prompt_after=None), stalling the game (human bb2016 seed1 i=9: the
                        // Ogre home_01's THROW_TEAM_MATE — Java throws & continues, Rust stalled).
                        PlayerAction::ThrowTeamMate => {
                            // Java bb2016 StepInitSelecting gates CLIENT_THROW_TEAM_MATE on
                            // `checkCommandWithActingPlayer(...) && !game.getTurnData().isPassUsed()`
                            // — a bb2016 TTM spends the team's PASS, so a SECOND TTM in one turn is
                            // rejected and the step stays put. The `Action::ThrowTeamMate` command arm
                            // above already honours this; this folded-target arm (the path the agent
                            // actually takes) did not, so Rust resolved a second TTM that stock Java
                            // refuses (ogre bb2016 seed 1: TTMs at i=2 and i=6).
                            if game.turn_data().pass_used {
                                return StepOutcome::cont();
                            }
                            self.dispatch_player_action = Some(PlayerAction::ThrowTeamMate);
                            return self.execute_step(game, rng)
                                .publish(StepParameter::ThrownPlayerId(Some(def.clone())));
                        }
                        PlayerAction::KickTeamMate => {
                            self.dispatch_player_action = Some(PlayerAction::KickTeamMate);
                            return self.execute_step(game, rng)
                                .publish(StepParameter::KickedPlayerId(Some(def.clone())));
                        }
                        _ => {}
                    }
                }
                // No-target block/blitz/foul (folded agent found no legal target) → DESELECT (end the
                // player action, NO turnover), matching Java's ParityRunner:
                //   sendBlockAction / sendFoulAction / a no-target BLITZ_MOVE all end with
                //   `ClientCommandActingPlayer(null, null, false)` when pickBlockTarget/pickFoulTarget
                //   returns null — a bare deselect that leaves the turn running so the runner picks the
                //   next player. (BLITZ is driven as a BLITZ_MOVE, a MOVE variant, so a no-target blitz
                //   likewise just moves/stands with no block.)
                //   amazon seed7 i=19: away_02 BLITZ, no adjacent target — Java continues away's turn.
                //   amazon seed12 i=20: home_01 BLOCK, its turn-start-snapshot target moved away so no
                //   adjacent opponent remains — Java sends ClientCommandActingPlayer(null) (JAVA_P2
                //   action=BLOCK, NO JAVA_BLOCK_PICK) and CONTINUES home's turn to Home9; Rust used to
                //   end the turn here (the earlier BLOCK→end_turn was an untested guess), turning over a
                //   full activation early. All these no-target action declarations deselect, never end
                //   the turn.
                //   PASS / HAND-OVER with no legal RECEIVER behave the same way: Java's
                //   sendPassAction / sendHandOverAction inject ClientCommandActingPlayer(null,null,false)
                //   when the receiver list is empty (a snapshot-offered PASS/HAND-OVER whose only
                //   adjacent teammate moved away, or a carrier with no reachable receiver). Without
                //   deselecting, a no-receiver hand-over fell through to a bare execute_step that
                //   emitted NO prompt, silently ending the team's turn early and desyncing the rest of
                //   the game (amazon bb2016 seed23 i=210: away_01 HAND_OFF, no adjacent receiver — Java
                //   deselects and continues away's turn to Away10, Rust stalled → 210 activations vs
                //   Java's 295).
                //   THROW / KICK TEAM-MATE with no legal thrown/kicked teammate deselect too: both the
                //   eligible-list builder (hasAdjacentTeammate, no Right-Stuff filter) and Java's
                //   sendThrowTeamMateAction can disagree — the action is OFFERED when any teammate is
                //   adjacent, but the thrown-player list requires canBeThrown (Right Stuff). A team
                //   with a Throw-Team-Mate Big Guy but no Right-Stuff teammate (e.g. a human Ogre)
                //   offers TTM, finds no canBeThrown target, and Java injects
                //   ClientCommandActingPlayer(null) (deselect, turn continues). human bb2016 seed1 i=9:
                //   the Ogre home_01's THROW_TEAM_MATE — Java deselects and continues, Rust stalled.
                if block_defender_id.is_none() {
                    match pa {
                        PlayerAction::Block | PlayerAction::Blitz | PlayerAction::Foul
                        | PlayerAction::Pass | PlayerAction::HandOver
                        | PlayerAction::ThrowTeamMate | PlayerAction::KickTeamMate => {
                            self.end_player_action = true;
                        }
                        _ => {}
                    }
                }
                return self.execute_step(game, rng);
            }
            _ => {}
        }
        StepOutcome::cont()
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            StepParameter::DispatchPlayerAction(v) => { self.dispatch_player_action = *v; true }
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            StepParameter::UpdatePersistence(v) => { self.update_persistence = *v; true }
            _ => false,
        }
    }
}

impl StepInitSelecting {
    /// Attaches the parked `GameEvent::PlayerAction` (see `pending_activation`) to whichever
    /// outcome the step produces. Report-only, so every return path can share one exit.
    pub fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let out = self.execute_step_inner(game, rng);
        match self.pending_activation.take() {
            Some((player_id, action)) => out.with_event(GameEvent::PlayerAction { player_id, action }),
            None => out,
        }
    }

    fn execute_step_inner(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let label = self.goto_label_on_end.clone();

        // no-op: headless engine has no turn timer; game.isTimeoutEnforced() always treated as false
        if self.end_turn {
            return StepOutcome::goto(&label)
                .publish(StepParameter::EndTurn(true));
        }

        if self.end_player_action {
            return StepOutcome::goto(&label)
                .publish(StepParameter::EndPlayerAction(true));
        }

        if let Some(dispatch_action) = self.dispatch_player_action {
            let player_id = game.acting_player.player_id.clone();
            let player_action = game.acting_player.player_action;
            if player_id.is_some() && player_action.is_some() {
                // Java: if (actingPlayer.isStandingUp()) { prepareStandingUp(); NEXT_STEP }
                //       else { GOTO_LABEL }
                if game.acting_player.standing_up {
                    self.prepare_standing_up(game);
                    return StepOutcome::next()
                        .publish(StepParameter::DispatchPlayerAction(Some(dispatch_action)));
                }
                return StepOutcome::goto(&label)
                    .publish(StepParameter::DispatchPlayerAction(Some(dispatch_action)));
            }
            // Java: fDispatchPlayerAction != null is an else-if branch — when the inner guard
            // (playerId provided && playerAction != null) fails, Java takes NO action at all
            // (does not fall through to the final else's prepareStandingUp()/NEXT_STEP logic).
            return StepOutcome::cont();
        }

        // Java: prepareStandingUp(); then NEXT_STEP if REMOVE_CONFUSION/STAND_UP/STAND_UP_BLITZ
        self.prepare_standing_up(game);
        let action = game.acting_player.player_action;
        if matches!(action, Some(PlayerAction::RemoveConfusion) | Some(PlayerAction::StandUp) | Some(PlayerAction::StandUpBlitz)) {
            return StepOutcome::next();
        }
        // BB2016 two-command activation: a player has been DECLARED (acting_player + action set)
        // but no DISPATCH command has arrived yet (dispatch_player_action == None). Java waits for a
        // CLIENT_MOVE/CLIENT_BLOCK/… from the client; the Rust engine models that "waiting" by
        // EMITTING a prompt so the agent can supply the 2nd command (the explicit move path / target).
        // Without a prompt the driver stalls (Continue + no pending_prompt) — this was the STUCK_STEP
        // on the first bb2016 activation. For a MOVE, emit AgentPrompt::Move with the legal one-step
        // destinations (same list StepInitMoving uses); the agent builds the full path and replies
        // Action::Move{path}. (BLOCK/BLITZ/FOUL dispatch prompts are wired in follow-up steps.)
        if action == Some(PlayerAction::Move) {
            if let Some(player_id) = game.acting_player.player_id.clone() {
                let squares = crate::legal_actions::legal_move_targets(game, &player_id);
                return StepOutcome::cont()
                    .with_prompt(ffb_model::prompts::AgentPrompt::Move { player_id, squares });
            }
        }
        StepOutcome::cont()
    }

    fn prepare_standing_up(&self, game: &mut Game) {
        let player_action = game.acting_player.player_action;

        if let Some(action) = player_action {
            // Java: if (BLITZ || BLITZ_MOVE || BLOCK || MULTIPLE_BLOCK) → updateDiceDecorations
            if matches!(action, PlayerAction::Blitz | PlayerAction::BlitzMove | PlayerAction::Block | PlayerAction::MultipleBlock) {
                ServerUtilBlock::update_dice_decorations(game);
            }

            // Java gates this on `actingPlayer.getPlayerAction().isMoving()`, evaluated against the
            // action the CLIENT declared — and both the GUI client and ParityRunner declare a Blitz as
            // **BLITZ_MOVE** (`declared = (action == BLITZ) ? BLITZ_MOVE : action`), which IS moving.
            // Rust stores that same declared action as `PlayerAction::Blitz` (renaming the variant
            // globally regresses the rest of the bb2016 blitz path), so `Blitz` must satisfy the gate
            // here. Without it a PRONE player's stand-up Blitz skipped the branch below that sets
            // `current_move = min(MINIMUM_MOVE_TO_STAND_UP, MA)` and
            // `goes_for_it = is_next_move_going_for_it(...)`, so the Rush d6 was never rolled (undead
            // bb2016 seed 1 step 187: a prone Mummy — MA 3, so standing up consumes the whole move —
            // blitzes; Java rolls rollGoingForIt then the 2 block dice, Rust rolled only the block
            // dice and every later die in the step shifted by one).
            if action.is_moving() || action == PlayerAction::Blitz {
                // Java: if (isStandingUp && !canStandUpForFree)
                //           setCurrentMove(min(MINIMUM_MOVE_TO_STAND_UP, movementWithModifiers))
                //           setGoingForIt(UtilPlayer.isNextMoveGoingForIt)
                if game.acting_player.standing_up {
                    let can_stand_up_for_free = game.acting_player.player_id.as_deref()
                        .and_then(|id| game.player(id))
                        .map(|p| p.has_skill_property(NamedProperties::CAN_STAND_UP_FOR_FREE))
                        .unwrap_or(false);
                    if !can_stand_up_for_free {
                        let movement_with_modifiers = game.acting_player.player_id.as_deref()
                            .and_then(|id| game.player(id))
                            .map(|p| p.movement_with_modifiers())
                            .unwrap_or(MINIMUM_MOVE_TO_STAND_UP);
                        game.acting_player.current_move = movement_with_modifiers.min(MINIMUM_MOVE_TO_STAND_UP);
                        game.acting_player.goes_for_it = UtilPlayer::is_next_move_going_for_it(game);
                    }
                }
                let jumping = game.acting_player.jumping;
                UtilServerPlayerMove::update_move_squares(game, jumping);
            }
        }
    }
}

fn pac_to_player_action(pac: PlayerActionChoice) -> PlayerAction {
    match pac {
        PlayerActionChoice::Move => PlayerAction::Move,
        PlayerActionChoice::Blitz => PlayerAction::Blitz,
        PlayerActionChoice::Block => PlayerAction::Block,
        PlayerActionChoice::Stab => PlayerAction::Stab,
        PlayerActionChoice::Foul => PlayerAction::Foul,
        PlayerActionChoice::Pass => PlayerAction::Pass,
        PlayerActionChoice::HandOff => PlayerAction::HandOver,
        PlayerActionChoice::StandUp => PlayerAction::StandUp,
        PlayerActionChoice::StandUpBlitz => PlayerAction::StandUpBlitz,
        PlayerActionChoice::ThrowTeamMate => PlayerAction::ThrowTeamMate,
        PlayerActionChoice::KickTeamMate => PlayerAction::KickTeamMate,
        PlayerActionChoice::HypnoticGaze => PlayerAction::Gaze,
        PlayerActionChoice::ThrowBomb => PlayerAction::ThrowBomb,
        PlayerActionChoice::HailMaryPass => PlayerAction::HailMaryPass,
        PlayerActionChoice::MultipleBlock => PlayerAction::MultipleBlock,
        PlayerActionChoice::RaidingParty => PlayerAction::RaidingParty,
        PlayerActionChoice::LookIntoMyEyes => PlayerAction::LookIntoMyEyes,
        PlayerActionChoice::BalefulHex => PlayerAction::BalefulHex,
        PlayerActionChoice::CatchOfTheDay => PlayerAction::CatchOfTheDay,
        PlayerActionChoice::ThenIStartedBlastin => PlayerAction::ThenIStartedBlastin,
        PlayerActionChoice::AllYouCanEat => PlayerAction::AllYouCanEat,
        PlayerActionChoice::Treacherous => PlayerAction::Treacherous,
        PlayerActionChoice::BlackInk => PlayerAction::BlackInk,
        PlayerActionChoice::Swoop => PlayerAction::Swoop,
        PlayerActionChoice::Punt => PlayerAction::Punt,
        PlayerActionChoice::BreatheFire => PlayerAction::BreatheFire,
        PlayerActionChoice::ProjectileVomit => PlayerAction::ProjectileVomit,
        PlayerActionChoice::SecureTheBall => PlayerAction::SecureTheBall,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::PlayerActionChoice;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2016)
    }

    /// Java `StepInitSelecting.prepareStandingUp()` gates its stand-up branch on
    /// `actingPlayer.getPlayerAction().isMoving()`, and both the GUI client and ParityRunner declare a
    /// Blitz as **BLITZ_MOVE** — which IS moving. That branch is what sets
    /// `currentMove = min(MINIMUM_MOVE_TO_STAND_UP, MA)` and
    /// `goingForIt = UtilPlayer.isNextMoveGoingForIt(game)`; for a prone player whose stand-up eats
    /// its whole movement (MA <= 3) the blitz's block then needs a Rush, so Java rolls a
    /// `rollGoingForIt` d6 BEFORE the block dice.
    ///
    /// Rust stores that declared action as `PlayerAction::Blitz`, so the gate must accept it. undead
    /// bb2016 seed 1 step 187: a prone Mummy (MA 3) blitzes — Java rolls GFI + 2 block dice + 2 armour
    /// + 2 injury (7 dice); without this Rust rolled no GFI, so its armour read the block's leftover
    /// die, held at 5 instead of breaking at 10, and no injury followed.
    #[test]
    fn prone_blitz_sets_going_for_it_when_standing_up_eats_the_move() {
        use ffb_model::enums::{PlayerState, PS_PRONE};
        use ffb_model::types::FieldCoordinate;

        fn setup(movement: i32) -> (Game, StepInitSelecting) {
            let mut game = make_game();
            let mut p = ffb_model::model::player::Player {
                id: "p1".into(), name: "p1".into(), nr: 1, position_id: "pos".into(),
                movement, strength: 3, agility: 3, passing: 4, armour: 9,
                ..Default::default()
            };
            p.starting_skills = vec![];
            game.team_home.players.push(p);
            game.home_playing = true;
            game.field_model.set_player_coordinate("p1", FieldCoordinate::new(10, 7));
            game.field_model.set_player_state("p1", PlayerState::new(PS_PRONE));
            game.acting_player.player_id = Some("p1".into());
            game.acting_player.player_action = Some(PlayerAction::Blitz);
            game.acting_player.standing_up = true;
            game.acting_player.has_acted = false;
            game.acting_player.current_move = 0;
            game.acting_player.goes_for_it = false;
            (game, StepInitSelecting::new("end".into()))
        }

        // MA 3 (Mummy): standing up costs the whole move → the blitz block is a Rush.
        let (mut game, step) = setup(3);
        step.prepare_standing_up(&mut game);
        assert_eq!(game.acting_player.current_move, 3,
            "the stand-up branch must charge MINIMUM_MOVE_TO_STAND_UP");
        assert!(game.acting_player.goes_for_it,
            "MA 3 stand-up leaves no movement, so the blitz block needs a Rush");

        // MA 6 (a normal player): movement remains after standing up → no Rush.
        let (mut game, step) = setup(6);
        step.prepare_standing_up(&mut game);
        assert_eq!(game.acting_player.current_move, 3);
        assert!(!game.acting_player.goes_for_it,
            "MA 6 stand-up still leaves movement, so the block is not a Rush");
    }

    /// Java bb2016 `ThrowTeamMateBehaviour` does `turnData.setPassUsed(true)`, and bb2016
    /// `StepInitSelecting.handleCommand` gates `CLIENT_THROW_TEAM_MATE` on
    /// `checkCommandWithActingPlayer(...) && !game.getTurnData().isPassUsed()` — so a SECOND Throw
    /// Team-Mate in one team turn is rejected and the step does not advance.
    ///
    /// The folded-target dispatch arm (the path the random agent actually takes, since it picks the
    /// thrown player at activation) skipped that gate, so Rust resolved a second TTM that stock Java
    /// refuses. ogre bb2016 seed 1 declared TTMs at i=2 AND i=6; Java rejected the second and
    /// ParityRunner re-declared it ~500× until `STUCK_STEP: INIT_SELECTING` killed the game.
    #[test]
    fn folded_throw_team_mate_is_rejected_once_the_pass_is_used() {
        use ffb_model::types::FieldCoordinate;
        use ffb_model::enums::{PlayerState, PS_STANDING};

        fn setup(pass_used: bool) -> (Game, StepInitSelecting) {
            let mut game = make_game();
            for id in ["thrower", "thrown"] {
                game.team_home.players.push(ffb_model::model::player::Player {
                    id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
                    movement: 6, strength: 5, agility: 3, passing: 4, armour: 9,
                    ..Default::default()
                });
            }
            game.home_playing = true;
            game.field_model.set_player_coordinate("thrower", FieldCoordinate::new(10, 7));
            game.field_model.set_player_state("thrower", PlayerState::new(PS_STANDING));
            game.field_model.set_player_coordinate("thrown", FieldCoordinate::new(11, 7));
            game.field_model.set_player_state("thrown", PlayerState::new(PS_STANDING));
            game.acting_player.player_id = Some("thrower".into());
            game.acting_player.player_action = Some(PlayerAction::ThrowTeamMate);
            game.turn_data_mut().pass_used = pass_used;
            (game, StepInitSelecting::new("end".into()))
        }

        // Pass already spent (a TTM earlier this turn) → the folded TTM must NOT be dispatched.
        let (mut game, mut step) = setup(true);
        let out = step.handle_command(
            &crate::action::Action::ActivatePlayer {
                player_id: "thrower".into(),
                player_action: PlayerActionChoice::ThrowTeamMate,
                block_defender_id: Some("thrown".into()),
            },
            &mut game, &mut GameRng::new(0),
        );
        assert!(matches!(out.action, StepAction::Continue),
            "a second bb2016 TTM in one turn is rejected, exactly as stock Java rejects the command");
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::ThrownPlayerId(Some(_)))),
            "no thrown player is published when the pass is already used");

        // Pass still available → the TTM dispatches normally.
        let (mut game, mut step) = setup(false);
        let out = step.handle_command(
            &crate::action::Action::ActivatePlayer {
                player_id: "thrower".into(),
                player_action: PlayerActionChoice::ThrowTeamMate,
                block_defender_id: Some("thrown".into()),
            },
            &mut game, &mut GameRng::new(0),
        );
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::ThrownPlayerId(Some(_)))),
            "the first TTM of the turn dispatches and publishes the thrown player");
    }

    #[test]
    fn start_waits_for_command() {
        let mut game = make_game();
        let mut step = StepInitSelecting::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn move_declare_emits_move_dispatch_prompt() {
        // BB2016 two-command activation: after a MOVE is DECLARED (acting_player + action set,
        // no dispatch yet), execute_step must EMIT AgentPrompt::Move so the agent can send the
        // 2nd command (the move path). Without it the driver stalled (STUCK_STEP on the first
        // bb2016 activation). Regression guard for the emit-prompt-after-declare piece.
        use ffb_model::prompts::AgentPrompt;
        use ffb_model::types::FieldCoordinate;
        use ffb_model::enums::{PS_STANDING, PlayerState};
        let mut game = make_game();
        game.team_home.players.push(ffb_model::model::player::Player {
            id: "mover".into(), name: "mover".into(), nr: 1, position_id: "lineman".into(),
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("mover", FieldCoordinate::new(13, 7));
        game.field_model.set_player_state("mover", PlayerState::new(PS_STANDING));
        game.home_playing = true;
        game.acting_player.player_id = Some("mover".into());
        game.acting_player.player_action = Some(PlayerAction::Move);
        game.acting_player.standing_up = false;
        let mut step = StepInitSelecting::new("end".into());
        let out = step.execute_step(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue, "declare must wait for the dispatch command");
        assert!(
            matches!(out.prompt, Some(AgentPrompt::Move { ref player_id, .. }) if player_id == "mover"),
            "MOVE declare must emit AgentPrompt::Move, got {:?}", out.prompt
        );
    }

    #[test]
    fn folded_block_activate_dispatches_with_target() {
        // BB2016 harness fold: a BLOCK declared via ActivatePlayer with a folded block target
        // (block_defender_id) must dispatch immediately (goto end label + publish BlockDefenderId +
        // DispatchPlayerAction=Block) so the bb2016 block sequence runs — the Rust agent stays
        // folded (same RNG as bb2025). Regression guard for the folded-dispatch piece.
        use ffb_model::types::FieldCoordinate;
        use ffb_model::enums::{PS_STANDING, PlayerState};
        let mut game = make_game();
        game.home_playing = true;
        game.team_home.players.push(ffb_model::model::player::Player {
            id: "att".into(), name: "att".into(), nr: 1, position_id: "lineman".into(),
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8, ..Default::default() });
        game.team_away.players.push(ffb_model::model::player::Player {
            id: "def".into(), name: "def".into(), nr: 2, position_id: "lineman".into(),
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8, ..Default::default() });
        game.field_model.set_player_coordinate("att", FieldCoordinate::new(13, 7));
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(14, 7));
        game.field_model.set_player_state("att", PlayerState::new(PS_STANDING));
        game.field_model.set_player_state("def", PlayerState::new(PS_STANDING));
        let mut step = StepInitSelecting::new("end".into());
        let out = step.handle_command(
            &Action::ActivatePlayer {
                player_id: "att".into(),
                player_action: PlayerActionChoice::Block,
                block_defender_id: Some("def".into()),
            },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(out.action, StepAction::GotoLabel, "folded block must dispatch (goto end)");
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::BlockDefenderId(d) if d == "def")),
            "must publish the folded block target, got {:?}", out.published);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::DispatchPlayerAction(Some(PlayerAction::Block)))),
            "must dispatch Block");
    }

    #[test]
    fn folded_no_target_action_deselects_not_turn() {
        // A BLOCK / BLITZ / FOUL / PASS / HAND-OVER declared with no currently-adjacent target or
        // receiver (block_defender_id=None) must END THE PLAYER ACTION (deselect), NOT end the turn
        // (and must not STALL) — matching Java's ParityRunner
        // sendBlockAction/sendFoulAction/sendPassAction/sendHandOverAction/no-target BLITZ_MOVE, which
        // all inject ClientCommandActingPlayer(null, null, false) (a bare deselect) when the
        // target/receiver pick returns null, leaving the turn running. amazon seed7 i=19 (no-target
        // BLITZ), seed12 i=20 (home_01 BLOCK whose snapshot target moved away → Java continues to
        // Home9, Rust turned over early), seed23 i=210 (away_01 HAND_OFF no adjacent receiver → Java
        // continues to Away10, Rust stalled with prompt_after=None → 210 activations vs Java's 295).
        for action in [PlayerActionChoice::Block, PlayerActionChoice::Blitz, PlayerActionChoice::Foul,
                       PlayerActionChoice::Pass, PlayerActionChoice::HandOff,
                       PlayerActionChoice::ThrowTeamMate, PlayerActionChoice::KickTeamMate] {
            let mut game = make_game();
            game.home_playing = true;
            game.team_home.players.push(ffb_model::model::player::Player {
                id: "actor".into(), name: "actor".into(), nr: 1, position_id: "lineman".into(),
                movement: 6, strength: 3, agility: 3, passing: 4, armour: 8, ..Default::default() });
            game.field_model.set_player_coordinate("actor", ffb_model::types::FieldCoordinate::new(5, 5));
            game.field_model.set_player_state("actor", ffb_model::enums::PlayerState::new(ffb_model::enums::PS_STANDING));
            let mut step = StepInitSelecting::new("end".into());
            let out = step.handle_command(
                &Action::ActivatePlayer {
                    player_id: "actor".into(),
                    player_action: action,
                    block_defender_id: None,
                },
                &mut game,
                &mut GameRng::new(0),
            );
            assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))),
                "no-target {action:?} must publish END_PLAYER_ACTION (deselect), got {:?}", out.published);
            assert!(!out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))),
                "no-target {action:?} must NOT end the turn");
        }
    }

    #[test]
    fn folded_pass_activate_dispatches_target_coordinate() {
        // BB2016 harness fold: a PASS declared via ActivatePlayer carries the RECEIVER
        // (block_defender_id). It must dispatch immediately, publishing the receiver's ABSOLUTE
        // coordinate as TARGET_COORDINATE + DispatchPlayerAction=Pass, so the bb2016 pass sequence
        // starts. Without this arm the Pass fell through to a bare cont() and stalled (i=201).
        use ffb_model::types::FieldCoordinate;
        use ffb_model::enums::{PS_STANDING, PlayerState};
        let mut game = make_game();
        game.home_playing = true;
        game.team_home.players.push(ffb_model::model::player::Player {
            id: "thrower".into(), name: "thrower".into(), nr: 1, position_id: "lineman".into(),
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8, ..Default::default() });
        game.team_home.players.push(ffb_model::model::player::Player {
            id: "rcv".into(), name: "rcv".into(), nr: 2, position_id: "lineman".into(),
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8, ..Default::default() });
        game.field_model.set_player_coordinate("thrower", FieldCoordinate::new(13, 8));
        game.field_model.set_player_coordinate("rcv", FieldCoordinate::new(16, 8));
        game.field_model.set_player_state("thrower", PlayerState::new(PS_STANDING));
        game.field_model.set_player_state("rcv", PlayerState::new(PS_STANDING));
        let mut step = StepInitSelecting::new("end".into());
        let out = step.handle_command(
            &Action::ActivatePlayer {
                player_id: "thrower".into(),
                player_action: PlayerActionChoice::Pass,
                block_defender_id: Some("rcv".into()),
            },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(out.action, StepAction::GotoLabel, "folded pass must dispatch (goto end)");
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::TargetCoordinate(c) if c.x == 16 && c.y == 8)),
            "must publish the receiver's absolute coordinate as TargetCoordinate, got {:?}", out.published);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::DispatchPlayerAction(Some(PlayerAction::Pass)))),
            "must dispatch Pass");
    }

    #[test]
    fn end_turn_command_gotos_label() {
        let mut game = make_game();
        let mut step = StepInitSelecting::new("end".into());
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn set_parameter_goto_label_on_end_accepted() {
        let mut step = StepInitSelecting::new("old".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("new".into())));
        assert_eq!(step.goto_label_on_end, "new");
    }

    #[test]
    fn set_parameter_update_persistence_accepted() {
        let mut step = StepInitSelecting::new("end".into());
        assert!(step.set_parameter(&StepParameter::UpdatePersistence(true)));
        assert!(step.update_persistence);
    }

    #[test]
    fn set_parameter_end_turn_accepted() {
        let mut step = StepInitSelecting::new("end".into());
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn set_parameter_dispatch_player_action_accepted() {
        let mut step = StepInitSelecting::new("end".into());
        assert!(step.set_parameter(&StepParameter::DispatchPlayerAction(Some(PlayerAction::Block))));
        assert_eq!(step.dispatch_player_action, Some(PlayerAction::Block));
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepInitSelecting::new("end".into());
        assert!(!step.set_parameter(&StepParameter::DodgeRoll(3)));
    }

    #[test]
    fn foul_command_blocked_when_foul_used() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::FoulMove);
        game.turn_data_home.foul_used = true;
        let mut step = StepInitSelecting::new("end".into());
        let out = step.handle_command(
            &Action::Foul { target_id: "def".into() },
            &mut game, &mut GameRng::new(0),
        );
        assert_eq!(out.action, StepAction::Continue, "foul should be blocked when foul_used=true");
    }

    #[test]
    fn foul_command_allowed_when_foul_not_used() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::FoulMove);
        game.turn_data_home.foul_used = false;
        let mut step = StepInitSelecting::new("end".into());
        let out = step.handle_command(
            &Action::Foul { target_id: "def".into() },
            &mut game, &mut GameRng::new(0),
        );
        assert_ne!(out.action, StepAction::Continue, "foul should be allowed when foul_used=false");
    }

    #[test]
    fn kick_team_mate_blocked_when_blitz_used() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::KickTeamMate);
        game.turn_data_home.blitz_used = true;
        let mut step = StepInitSelecting::new("end".into());
        let out = step.handle_command(
            &Action::KickTeamMate { player_id: "victim".into(), coord: ffb_model::types::FieldCoordinate::new(5, 5) },
            &mut game, &mut GameRng::new(0),
        );
        assert_eq!(out.action, StepAction::Continue, "kick_team_mate should be blocked when blitz_used=true");
    }

    #[test]
    fn pass_command_blocked_when_pass_used() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Pass);
        game.turn_data_home.pass_used = true;
        let mut step = StepInitSelecting::new("end".into());
        let out = step.handle_command(
            &Action::Pass { coord: ffb_model::types::FieldCoordinate::new(7, 5) },
            &mut game, &mut GameRng::new(0),
        );
        assert_eq!(out.action, StepAction::Continue, "pass should be blocked when pass_used=true");
    }

    #[test]
    fn hand_over_blocked_when_hand_over_used() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::HandOver);
        game.turn_data_home.hand_over_used = true;
        let mut step = StepInitSelecting::new("end".into());
        let out = step.handle_command(
            &Action::HandOff { receiver_id: "recv".into() },
            &mut game, &mut GameRng::new(0),
        );
        assert_eq!(out.action, StepAction::Continue, "hand_off should be blocked when hand_over_used=true");
    }

    #[test]
    fn update_persistence_cleared_on_start() {
        let mut game = make_game();
        let mut step = StepInitSelecting::new("end".into());
        step.update_persistence = true;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!step.update_persistence);
    }

    #[test]
    fn end_player_action_set_gotos_label() {
        let mut game = make_game();
        let mut step = StepInitSelecting::new("end".into());
        // Set end_player_action via parameter, then start triggers it
        step.set_parameter(&StepParameter::EndPlayerAction(true));
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        // EndTurn sets end_turn → GotoLabel with EndTurn
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn standing_up_player_with_dispatch_returns_next_step() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        use std::collections::HashSet;
        let mut game = make_game();
        game.team_home.players.push(Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
                    ..Default::default()
});
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Move);
        game.acting_player.standing_up = true;
        let mut step = StepInitSelecting::new("end".into());
        step.dispatch_player_action = Some(PlayerAction::Move);
        // standing_up + dispatch → prepareStandingUp + NEXT_STEP
        let out = step.execute_step(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn standing_up_sets_current_move_to_min_of_ma_and_minimum_stand_up() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        use std::collections::HashSet;
        let mut game = make_game();
        // Player with MA=4 (> MINIMUM_MOVE_TO_STAND_UP=3)
        game.team_home.players.push(Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
                    ..Default::default()
});
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Move);
        game.acting_player.standing_up = true;
        let mut step = StepInitSelecting::new("end".into());
        step.dispatch_player_action = Some(PlayerAction::Move);
        step.execute_step(&mut game, &mut GameRng::new(0));
        // MA=4 but capped to MINIMUM_MOVE_TO_STAND_UP=3
        assert_eq!(game.acting_player.current_move, 3);
    }

    #[test]
    fn block_command_publishes_block_defender_and_dispatch() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Block);
        let mut step = StepInitSelecting::new("end".into());
        let action = Action::Block { defender_id: "def1".into() };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::BlockDefenderId(_))));
    }

    fn activate_player_action(game: &mut Game, step: &mut StepInitSelecting, pac: PlayerActionChoice) -> StepOutcome {
        let rng = &mut GameRng::new(0);
        let action = Action::ActivatePlayer { player_id: "p1".into(), player_action: pac, block_defender_id: None
};
        step.handle_command(&action, game, rng)
    }

    #[test]
    fn stand_up_action_returns_next_step() {
        let mut game = make_game();
        game.home_playing = true;
        game.acting_player.player_id = Some("p1".into());
        game.team_home.players.push(ffb_model::model::player::Player { id: "p1".into(), ..Default::default() });
        let mut step = StepInitSelecting::new("end".into());
        let out = activate_player_action(&mut game, &mut step, PlayerActionChoice::StandUp);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn remove_confusion_via_activate_returns_next_step() {
        let mut game = make_game();
        game.home_playing = true;
        game.acting_player.player_id = Some("p1".into());
        game.team_home.players.push(ffb_model::model::player::Player { id: "p1".into(), ..Default::default() });
        let mut step = StepInitSelecting::new("end".into());
        // RemoveConfusion doesn't have a PlayerActionChoice variant — use StandUp as a proxy
        // for the else-NEXT_STEP branch; RemoveConfusion is set via set_parameter
        game.acting_player.player_action = Some(PlayerAction::RemoveConfusion);
        let rng = &mut GameRng::new(0);
        let out = step.execute_step(&mut game, rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn stand_up_blitz_action_returns_next_step() {
        let mut game = make_game();
        game.home_playing = true;
        game.acting_player.player_id = Some("p1".into());
        game.team_home.players.push(ffb_model::model::player::Player { id: "p1".into(), ..Default::default() });
        let mut step = StepInitSelecting::new("end".into());
        let out = activate_player_action(&mut game, &mut step, PlayerActionChoice::StandUpBlitz);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn move_action_via_activate_returns_cont_waiting_for_move_command() {
        let mut game = make_game();
        game.home_playing = true;
        game.acting_player.player_id = Some("p1".into());
        game.team_home.players.push(ffb_model::model::player::Player { id: "p1".into(), ..Default::default() });
        let mut step = StepInitSelecting::new("end".into());
        let out = activate_player_action(&mut game, &mut step, PlayerActionChoice::Move);
        // MOVE via ActivatePlayer → executeStep with no dispatch → cont (waits for CLIENT_MOVE)
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn report_list_empty_after_end_turn() {
        // Java bb2016 StepInitSelecting has no addReport calls — verify report_list stays empty.
        let mut game = make_game();
        let mut step = StepInitSelecting::new("end".into());
        step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert!(game.report_list.is_empty(),
            "bb2016 StepInitSelecting has no addReport calls — report_list should remain empty");
    }

    #[test]
    fn end_player_action_param_then_end_turn_publishes_end_turn() {
        let mut game = make_game();
        let mut step = StepInitSelecting::new("end".into());
        step.set_parameter(&StepParameter::EndPlayerAction(true));
        // EndTurn overrides end_player_action because it's checked first
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn dispatch_set_with_failed_guard_does_not_fall_through_to_standing_up_branch() {
        // Java: `else if (fDispatchPlayerAction != null) { if (playerId provided && playerAction
        // != null) {...} }` — when fDispatchPlayerAction is set but the inner guard fails, Java
        // takes NO action (falls out of the else-if chain entirely). It must NOT fall through to
        // the final `else { prepareStandingUp(); NEXT_STEP if REMOVE_CONFUSION/STAND_UP/... }`
        // branch, even if game.acting_player.player_action happens to be STAND_UP.
        let mut game = make_game();
        game.acting_player.player_id = None; // guard fails: playerId not provided
        game.acting_player.player_action = Some(PlayerAction::StandUp);
        let mut step = StepInitSelecting::new("end".into());
        step.dispatch_player_action = Some(PlayerAction::Move);
        let out = step.execute_step(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue,
            "dispatch_player_action set with failed guard must not fall through to the standing-up NEXT_STEP branch");
    }

    #[test]
    fn pass_target_coordinate_transformed_for_away_team() {
        // Java: CLIENT_PASS → if (game.isHomePlaying()) publish(coord) else publish(coord.transform())
        let mut game = make_game();
        game.home_playing = false;
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Pass);
        let mut step = StepInitSelecting::new("end".into());
        let coord = ffb_model::types::FieldCoordinate::new(10, 7);
        let out = step.handle_command(&Action::Pass { coord }, &mut game, &mut GameRng::new(0));
        let published = out.published.iter().find(|p| matches!(p, StepParameter::TargetCoordinate(_)));
        if let Some(StepParameter::TargetCoordinate(c)) = published {
            assert_eq!(*c, coord.transform(), "away-team pass target coordinate must be mirrored");
        } else {
            panic!("TargetCoordinate not published");
        }
    }

    #[test]
    fn pass_target_coordinate_not_transformed_for_home_team() {
        let mut game = make_game();
        game.home_playing = true;
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Pass);
        let mut step = StepInitSelecting::new("end".into());
        let coord = ffb_model::types::FieldCoordinate::new(10, 7);
        let out = step.handle_command(&Action::Pass { coord }, &mut game, &mut GameRng::new(0));
        let published = out.published.iter().find(|p| matches!(p, StepParameter::TargetCoordinate(_)));
        if let Some(StepParameter::TargetCoordinate(c)) = published {
            assert_eq!(*c, coord, "home-team pass target coordinate must not be mirrored");
        } else {
            panic!("TargetCoordinate not published");
        }
    }

    #[test]
    fn throw_team_mate_target_coordinate_transformed_for_away_team() {
        // Java: CLIENT_THROW_TEAM_MATE → if (game.isHomePlaying()) publish(coord) else publish(coord.transform())
        let mut game = make_game();
        game.home_playing = false;
        game.acting_player.player_id = Some("p1".into());
        let mut step = StepInitSelecting::new("end".into());
        let coord = ffb_model::types::FieldCoordinate::new(4, 9);
        let out = step.handle_command(
            &Action::ThrowTeamMate { player_id: "tm1".into(), coord },
            &mut game, &mut GameRng::new(0),
        );
        let published = out.published.iter().find(|p| matches!(p, StepParameter::TargetCoordinate(_)));
        if let Some(StepParameter::TargetCoordinate(c)) = published {
            assert_eq!(*c, coord.transform(), "away-team throw-team-mate target coordinate must be mirrored");
        } else {
            panic!("TargetCoordinate not published");
        }
    }
}
