use ffb_model::enums::PlayerAction;
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::prompts::AgentPrompt;
use ffb_model::report::mixed::report_staller_detected::ReportStallerDetected;
use super::stalling_extension::StallingExtension;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::util::rng::GameRng;
use crate::action::{Action, PlayerActionChoice};
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_steps;

/// Initialises the player-selection phase: waits for `ActivatePlayer` or `EndTurn` commands,
/// then dispatches to the appropriate action sequence via GOTO_LABEL or NEXT_STEP.
///
/// Java executeStep routing:
///   end_turn → GotoLabel(end) + publish EndTurn + CheckForgo
///   end_player_action → GotoLabel(end) + publish EndPlayerAction
///   dispatch_player_action set + acting player present:
///     publish DispatchPlayerAction
///     if standing_up && !force_goto → NextStep (proceed to JumpUp/StandUp)
///     else → GotoLabel(end)
///   otherwise → Continue (waiting for command)
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.shared.StepInitSelecting`.
pub struct StepInitSelecting {
    /// Java: fGotoLabelOnEnd (init param)
    pub goto_label_on_end: String,
    /// Java: fDispatchPlayerAction
    pub dispatch_player_action: Option<PlayerAction>,
    /// Java: fEndTurn
    pub end_turn: bool,
    /// Java: fEndPlayerAction
    pub end_player_action: bool,
    /// Java: forceGotoOnDispatch
    pub force_goto_on_dispatch: bool,
    /// Java: fUpdatePersistence (mandatory init param UPDATE_PERSISTENCE) — Java stores it
    /// purely to decide whether to persist the game state on start(); persistence is not
    /// modeled in this crate, so the value is stored but unused.
    pub update_persistence: bool,
}

impl StepInitSelecting {
    pub fn new(goto_label_on_end: String) -> Self {
        Self {
            goto_label_on_end,
            dispatch_player_action: None,
            end_turn: false,
            end_player_action: false,
            force_goto_on_dispatch: false,
            update_persistence: false,
        }
    }
}

impl Step for StepInitSelecting {
    fn id(&self) -> StepId { StepId::InitSelecting }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Post-special continuation (Treacherous bridging): the acting player still carries
        // PASS_MOVE and must now throw the declared pass — Java's select-sequence InitSelecting
        // leaves the acting player in place and the client sends CLIENT_PASS (ParityRunner
        // phase 2 → sendPassAction). Without this guard the activation-retire bridging below
        // CLEARED the acting player and re-opened the team-wide activation prompt, so the agent
        // activated a DIFFERENT player while Java threw the pass (renegades bb2020 seed 91
        // i=94). BombRethrow is the generic "acting player must pass now" window — both agents
        // answer it with sendPassAction's exact contract.
        let acted = game.acting_player.has_moved
            || game.acting_player.has_fouled
            || game.acting_player.has_blocked
            || game.acting_player.has_passed
            || game.acting_player.has_triggered_effect
            || game.acting_player.forgone;
        // Post-BLITZ_SELECT continuation. StepSelectBlitzTargetEnd sets the acting action back to
        // BLITZ_MOVE and pushes the Select sequence, which re-enters this step with the target
        // already chosen. Java's :114 guard is `BLITZ_MOVE && targetSelectionState == null`, so on
        // this SECOND pass the state is non-null and Java falls through to the ordinary dispatch,
        // running the real move + block. Crucially Java does NOT consult hasActed anywhere on this
        // path: a mid-flight blitz is CONTINUED, never re-activated.
        //
        // This check therefore has to sit ABOVE the `!acted` gate below, which is a Rust-side
        // construct with no Java counterpart. While it was nested inside that gate it swallowed
        // the blitz of any player who had legally MOVED before declaring it - has_moved makes
        // `acted` true, so the step asked for a fresh activation and the blitz died having done
        // nothing. Measured on lineman bb2025 seed 3: 8 blitzes reached SelectBlitzTargetEnd but
        // only the 5 with has_moved=false were continued; the other 3 were lost exactly here.
        if game.acting_player.player_action == Some(PlayerAction::BlitzMove)
            && game.field_model.target_selection_state.is_some()
        {
            self.dispatch_player_action = Some(PlayerAction::BlitzMove);
            self.force_goto_on_dispatch = true;
            // Java `StepInitSelecting:238`, case CLIENT_BLOCK - the command that carries a blitz's
            // SECOND phase - publishes USE_ALTERNATE_LABEL=true. The Select sequence that
            // StepSelectBlitzTargetEnd pushes begins
            //     ... GOTO_LABEL(goto=NEXT, alternate=END_SELECTING) BONE_HEAD[NEXT] REALLY_STUPID ...
            // so that flag makes the GOTO_LABEL jump straight to END_SELECTING and SKIP the whole
            // negatrait block. Without it the activation runs in BOTH passes of the chain and a
            // negatrait carrier rolls Bone Head / Really Stupid twice per blitz - measured as +3
            // actionRng calls on one goblin Troll blitz (goblin seed 1 i=47: main 52->53,
            // branch 52->56). Java queues the activation twice exactly as Rust does; this publish
            // is what stops the second one from rolling.
            return self.execute_step(game, _rng)
                .publish(StepParameter::UseAlternateLabel(true));
        }

        if !acted {
            match game.acting_player.player_action {
                Some(PlayerAction::PassMove) => {
                    if let Some(pid) = game.acting_player.player_id.clone() {
                        return StepOutcome::cont().with_prompt(AgentPrompt::BombRethrow { player_id: pid });
                    }
                }
                // MULTIPLE_BLOCK continuation: the declared player must now pick TWO targets —
                // Java waits for CLIENT_SYNCHRONOUS_MULTI_BLOCK; surface the window as a prompt.
                Some(PlayerAction::MultipleBlock) => {
                    if let Some(pid) = game.acting_player.player_id.clone() {
                        if let Some(coord) = game.field_model.player_coordinate(&pid) {
                            let inactive = game.inactive_team();
                            let mut elig: Vec<String> =
                                ffb_model::util::util_player::UtilPlayer::find_adjacent_blockable_players(
                                    game, inactive, coord,
                                ).into_iter().cloned().collect();
                            elig.sort_by_key(|id| {
                                game.field_model.player_coordinate(id)
                                    .map(|c| (c.x, c.y)).unwrap_or((i32::MAX, i32::MAX))
                            });
                            return StepOutcome::cont().with_prompt(
                                AgentPrompt::MultiBlockTargets { player_id: pid, eligible_players: elig });
                        }
                    }
                }
                // Post-Black-Ink continuation: the acting player still carries MOVE and Java's
                // phase 2 simply moves them (sendMoveAction). Re-dispatch the MOVE so
                // EndSelecting pushes the Move sequence and the agent gets the ordinary Move
                // prompt — clearing here handed the activation to a different player.
                Some(PlayerAction::Move) => {
                    self.dispatch_player_action = Some(PlayerAction::Move);
                    return self.execute_step(game, _rng);
                }
                // Post-BLITZ_SELECT continuation. StepSelectBlitzTargetEnd sets the acting action
                // back to BLITZ_MOVE and pushes the Select sequence, which lands here with the
                // target already chosen. Java's StepInitSelecting :114 guard is
                // `BLITZ_MOVE && targetSelectionState == null`, so on this SECOND pass the state
                // is non-null and it falls through to the ordinary dispatch, running the real
                // move + block. Without this arm the re-entered step asked for a NEW activation
                // instead and the blitz ended having done nothing (lineman bb2025 seed 1 i=1:
                // Java blocks and spends a die, Rust spends none).
                _ => {}
            }
        }
        // Rust bridging for Java's UtilActingPlayer.changeActingPlayer: Java deactivates the
        // previous acting player when the NEXT one is selected and leaves "already activated
        // this turn" tracking to the coach; here the engine records the finished activation
        // (turn_data.acted_player_ids feeds legal_activate_player_actions) when selection
        // resumes, so the eligible list shrinks for every agent. hasActed() is Java's
        // computed getter (moved || fouled || blocked || passed || triggeredEffect || forgone).
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
        // Java start() only updates persistence — it does NOT call executeStep().
        // Emit the activation prompt so the agent knows which players are available.
        let eligible = crate::legal_actions::eligible_players_for_activation(game);
        StepOutcome::cont()
            .with_prompt(AgentPrompt::ActivatePlayer { eligible_players: eligible })
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::EndTurn => {
                self.end_turn = true;
            }
            // Java: CLIENT_ACTING_PLAYER(null, null, false) — the DESELECT. The agents send it
            // when a declared two-phase action can no longer proceed (e.g. a MULTIPLE_BLOCK
            // whose second target died between the turn-start snapshot and the activation —
            // ParityRunner's sendSynchronousMultiBlock injects the same null ActingPlayer).
            // Without this arm the command fell through and the MultiBlockTargets window
            // re-prompted forever (dark_elf bb2020 seed 2 i=6).
            Action::EndPlayerAction => {
                util_server_steps::change_player_action_to_none(game);
            }
            Action::ActivatePlayer { player_id, player_action, block_defender_id } => {
                if std::env::var_os("FFB_TRACE").is_some() {
                    eprintln!("RACT pid={player_id} pac={player_action:?} bdef={block_defender_id:?} game_def={:?}",
                        game.defender_id);
                }
                // Rust bridging: a TREACHEROUS "declaration" stands for the client's two
                // commands — CLIENT_ACTING_PLAYER(PASS_MOVE) then CLIENT_USE_SKILL(treacherous)
                // (bb2025 SelectLogicModule pairs sendUseSkill with the ball-action cases;
                // Treacherous ADDS those ball actions to a non-carrier's menu). Mirror both:
                // set the acting player with PASS_MOVE, then take the UseSkill chain's
                // dispatch (Java StepInitSelecting:354 canStabTeamMateForBall → TREACHEROUS,
                // forceGotoOnDispatch).
                // MULTIPLE_BLOCK is two-phase like Java: the declaration only sets the acting
                // player (CLIENT_ACTING_PLAYER); the engine then waits for
                // CLIENT_SYNCHRONOUS_MULTI_BLOCK with both targets (the MultiBlockTargets
                // continuation prompt below).
                if *player_action == PlayerActionChoice::MultipleBlock {
                    game.original_bombardier = None;
                    util_server_steps::change_player_action(game, player_id, PlayerAction::MultipleBlock, false);
                    game.defender_id = None;
                    Self::check_for_staller(game);
                    return self.execute_step(game, rng)
                        .with_event(GameEvent::PlayerAction { player_id: player_id.clone(), action: PlayerAction::MultipleBlock });
                }
                // Rust bridging: BLACK_INK stands for ActingPlayer(MOVE) + UseSkill(blackInk)
                // — the client offers the ink from the ordinary action modules and the player
                // CONTINUES the declared move after the gaze (see the Move continuation in
                // start()).
                if *player_action == PlayerActionChoice::BlackInk {
                    game.original_bombardier = None;
                    util_server_steps::change_player_action(game, player_id, PlayerAction::Move, false);
                    game.defender_id = None;
                    self.dispatch_player_action = Some(PlayerAction::BlackInk);
                    self.force_goto_on_dispatch = true;
                    Self::check_for_staller(game);
                    return self.execute_step(game, rng)
                        .with_event(GameEvent::PlayerAction { player_id: player_id.clone(), action: PlayerAction::BlackInk });
                }
                // Rust bridging: WISDOM_OF_THE_WHITE_DWARF stands for
                // ActingPlayer(MOVE) + ClientCommandUseTeamMatesWisdom. Java's StepInitSelecting
                // handles CLIENT_USE_TEAM_MATES_WISDOM by setting ONLY the dispatch action
                // (fDispatchPlayerAction = WISDOM_OF_THE_WHITE_DWARF, forceGotoOnDispatch = true)
                // — it never calls changeActingPlayer, so the declared action stays MOVE. Same
                // shape as the BLACK_INK bridging above.
                // Rust bridging: AUTO_GAZE_ZOAT stands for ActingPlayer(MOVE) +
                // UseSkill("Excuse Me, Are You a Zoat?"), exactly like BLACK_INK — Java's
                // CLIENT_USE_SKILL chain dispatches AUTO_GAZE_ZOAT with forceGotoOnDispatch and
                // never calls changeActingPlayer, so the declared action stays MOVE.
                // Java StepInitSelecting :114 —
                //   if (playerAction == BLITZ_MOVE && targetSelectionState == null) {
                //     fDispatchPlayerAction = BLITZ_SELECT;
                //     changeActingPlayer(pid, BLITZ_MOVE, jumping);   // acting action stays BLITZ_MOVE
                //     forceGotoOnDispatch = true;
                //   }
                // This branch was entirely missing, which is why Rust never dispatched
                // SelectBlitzTarget/SelectBlitzTargetEnd on ANY blitz while Java runs them ~750
                // times per 100 games (docs/BACKLOG.md §12). The target is no longer folded into
                // the declaration: StepSelectBlitzTarget asks for it, at the same point in the
                // actionRng stream (before the negatrait rolls) as the old activation-time pick.
                if *player_action == PlayerActionChoice::Blitz
                    && game.field_model.target_selection_state.is_none()
                {
                    game.original_bombardier = None;
                    util_server_steps::change_player_action(game, player_id, PlayerAction::BlitzMove, false);
                    game.defender_id = None;
                    self.dispatch_player_action = Some(PlayerAction::BlitzSelect);
                    self.force_goto_on_dispatch = true;
                    // Java's :114 arm does NOT return - it falls through to the shared tail of the
                    // CLIENT_ACTING_PLAYER case. Run that tail here (see the helper's note): without
                    // it a prone blitzer skips the stand-up movement cost and owes no GO FOR IT.
                    Self::apply_moving_activation_updates(game, player_id, PlayerAction::BlitzMove);
                    Self::check_for_staller(game);
                    return self.execute_step(game, rng)
                        .with_event(GameEvent::PlayerAction { player_id: player_id.clone(), action: PlayerAction::BlitzMove });
                }
                if *player_action == PlayerActionChoice::AutoGazeZoat {
                    game.original_bombardier = None;
                    util_server_steps::change_player_action(game, player_id, PlayerAction::Move, false);
                    game.defender_id = None;
                    self.dispatch_player_action = Some(PlayerAction::AutoGazeZoat);
                    self.force_goto_on_dispatch = true;
                    Self::check_for_staller(game);
                    return self.execute_step(game, rng)
                        .with_event(GameEvent::PlayerAction { player_id: player_id.clone(), action: PlayerAction::AutoGazeZoat });
                }
                if *player_action == PlayerActionChoice::WisdomOfTheWhiteDwarf {
                    game.original_bombardier = None;
                    util_server_steps::change_player_action(game, player_id, PlayerAction::Move, false);
                    game.defender_id = None;
                    self.dispatch_player_action = Some(PlayerAction::WisdomOfTheWhiteDwarf);
                    self.force_goto_on_dispatch = true;
                    Self::check_for_staller(game);
                    return self.execute_step(game, rng)
                        .with_event(GameEvent::PlayerAction { player_id: player_id.clone(), action: PlayerAction::WisdomOfTheWhiteDwarf });
                }
                if *player_action == PlayerActionChoice::ThenIStartedBlastin {
                    // Java client: ActingPlayer + sendUseSkill(blastin); the server's
                    // CLIENT_USE_SKILL chain dispatches THEN_I_STARTED_BLASTIN with forceGoto.
                    game.original_bombardier = None;
                    util_server_steps::change_player_action(game, player_id, PlayerAction::Move, false);
                    game.defender_id = None;
                    self.dispatch_player_action = Some(PlayerAction::ThenIStartedBlastin);
                    self.force_goto_on_dispatch = true;
                    Self::check_for_staller(game);
                    return self.execute_step(game, rng)
                        .with_event(GameEvent::PlayerAction { player_id: player_id.clone(), action: PlayerAction::ThenIStartedBlastin });
                }
                if *player_action == PlayerActionChoice::CatchOfTheDay {
                    // Java client: ActingPlayer + sendUseSkill(catchOfTheDay); the server's
                    // CLIENT_USE_SKILL chain dispatches CATCH_OF_THE_DAY with forceGoto.
                    game.original_bombardier = None;
                    util_server_steps::change_player_action(game, player_id, PlayerAction::Move, false);
                    game.defender_id = None;
                    self.dispatch_player_action = Some(PlayerAction::CatchOfTheDay);
                    self.force_goto_on_dispatch = true;
                    Self::check_for_staller(game);
                    return self.execute_step(game, rng)
                        .with_event(GameEvent::PlayerAction { player_id: player_id.clone(), action: PlayerAction::CatchOfTheDay });
                }
                if *player_action == PlayerActionChoice::BalefulHex {
                    // Java client: ActingPlayer + sendUseSkill(balefulHex); the server's
                    // CLIENT_USE_SKILL chain dispatches BALEFUL_HEX with forceGoto.
                    game.original_bombardier = None;
                    util_server_steps::change_player_action(game, player_id, PlayerAction::Move, false);
                    game.defender_id = None;
                    self.dispatch_player_action = Some(PlayerAction::BalefulHex);
                    self.force_goto_on_dispatch = true;
                    Self::check_for_staller(game);
                    return self.execute_step(game, rng)
                        .with_event(GameEvent::PlayerAction { player_id: player_id.clone(), action: PlayerAction::BalefulHex });
                }
                if *player_action == PlayerActionChoice::LookIntoMyEyes {
                    // Java client: ActingPlayer + sendUseSkill(lookIntoMyEyes); the server's
                    // CLIENT_USE_SKILL chain dispatches LOOK_INTO_MY_EYES with forceGoto.
                    game.original_bombardier = None;
                    util_server_steps::change_player_action(game, player_id, PlayerAction::Move, false);
                    game.defender_id = None;
                    self.dispatch_player_action = Some(PlayerAction::LookIntoMyEyes);
                    self.force_goto_on_dispatch = true;
                    Self::check_for_staller(game);
                    return self.execute_step(game, rng)
                        .with_event(GameEvent::PlayerAction { player_id: player_id.clone(), action: PlayerAction::LookIntoMyEyes });
                }
                if *player_action == PlayerActionChoice::RaidingParty {
                    // Java client: sendActingPlayer(player, MOVE) then sendUseSkill(raidingSkill)
                    // (BlockLogicExtension/MoveLogicModule); the server's CLIENT_USE_SKILL chain
                    // dispatches RAIDING_PARTY with forceGotoOnDispatch.
                    game.original_bombardier = None;
                    util_server_steps::change_player_action(game, player_id, PlayerAction::Move, false);
                    game.defender_id = None;
                    self.dispatch_player_action = Some(PlayerAction::RaidingParty);
                    self.force_goto_on_dispatch = true;
                    Self::check_for_staller(game);
                    return self.execute_step(game, rng)
                        .with_event(GameEvent::PlayerAction { player_id: player_id.clone(), action: PlayerAction::RaidingParty });
                }
                if *player_action == PlayerActionChoice::Treacherous {
                    game.original_bombardier = None;
                    util_server_steps::change_player_action(game, player_id, PlayerAction::PassMove, false);
                    game.defender_id = None;
                    self.dispatch_player_action = Some(PlayerAction::Treacherous);
                    self.force_goto_on_dispatch = true;
                    Self::check_for_staller(game);
                    return self.execute_step(game, rng)
                        .with_event(GameEvent::PlayerAction { player_id: player_id.clone(), action: PlayerAction::Treacherous });
                }
                let pa = pac_to_player_action(*player_action);
                // Java CLIENT_ACTING_PLAYER: `if (playerAction.isBomb()) {
                //   passState.setOriginalBombardier(playerId); ... } else { passState.reset(); }`
                // — the Rust home of PassState.originalBombardier is game.original_bombardier
                // (read by StepSpecialEffect's bomber-turnover reset and StepEndBomb's
                // acting-player restore). Without the reset a bomber id lingered across
                // unrelated later actions.
                if pa.is_bomb() {
                    game.original_bombardier = Some(player_id.clone());
                    // Java: `if (playerAction == PlayerAction.ALL_YOU_CAN_EAT)
                    //          passState.setThrowTwoBombs(true);` — commit to two bombs.
                    if pa == PlayerAction::AllYouCanEat {
                        game.throw_two_bombs = Some(true);
                    }
                } else {
                    // Java: passState.reset() on a non-bomb declaration.
                    game.original_bombardier = None;
                    game.throw_two_bombs = None;
                }
                util_server_steps::change_player_action(game, player_id, pa, false);
                if let Some(def_id) = block_defender_id {
                    game.defender_id = Some(def_id.clone());
                } else {
                    // No target chosen this activation (e.g. a HandOver/Pass whose receiver list is
                    // empty, or a no-defender block). Clear any stale defender from an earlier
                    // activation so the dispatch below sees a reliable "no target" signal instead of
                    // publishing a stale coordinate/defender.
                    game.defender_id = None;
                }
                // Block/Blitz variants: go directly to label (forceGotoOnDispatch)
                self.force_goto_on_dispatch = matches!(
                    player_action,
                    PlayerActionChoice::Block | PlayerActionChoice::Blitz
                );
                // Java dispatch reads actingPlayer.getPlayerAction() (the DELEGATE) unless
                // forceDispatch set the raw action; ALL_YOU_CAN_EAT therefore dispatches as
                // THROW_BOMB (EndSelecting has no ALL_YOU_CAN_EAT case in Java either).
                self.dispatch_player_action = Some(pa.delegate().unwrap_or(pa));
                // A prone Blitz/Block declared with NO target: Java resolves the target BEFORE the
                // stand-up (Blitz via SelectBlitzTarget; a plain Block reaches StepInitBlocking's
                // no-defender branch), so a null-target block/blitz ends the turn with the player still
                // PRONE. Rust stands up in the Select sequence first, so suppress the stand-up here —
                // the no-defender branch then ends the turn, leaving the player prone to match Java
                // (Blitz: seed 7 i=39 away_01; Block: halfling seed 5 i=25 — a thrown prone Treeman
                // home_03 at (17,8) with no adjacent opponent stayed Prone in Java, stood up in Rust).
                if matches!(player_action, PlayerActionChoice::Blitz | PlayerActionChoice::Block)
                    && block_defender_id.is_none()
                {
                    game.acting_player.standing_up = false;
                }
                // A prone player activated for Throw/Kick Team-Mate must NOT stand up. Java's
                // StepInitSelecting gates its whole stand-up block on
                // `playerAction.isMoving() || playerAction.isStandingUp()`, and THROW_TEAM_MATE /
                // KICK_TEAM_MATE are neither (only the *_MOVE variants count as "moving"), so Java
                // never pre-stands here. A prone player cannot legally TTM, so the action deselects
                // (see the empty-target ThrowTeamMate branch in execute_step) and the player stays
                // PRONE. Rust otherwise pre-stands via the `standing_up` flag (renegades seed 11 i=188 /
                // underworld seed 7: away_03, a prone Animal-Savagery lash-out victim, was activated for
                // ThrowTeamMate → Rust stood it up while Java left it prone). For a legal (standing)
                // thrower `standing_up` is already false, so this is a no-op.
                if matches!(player_action, PlayerActionChoice::ThrowTeamMate | PlayerActionChoice::KickTeamMate) {
                    game.acting_player.standing_up = false;
                }
                // Java: if (playerAction.isMoving() || playerAction.isStandingUp())
                //   UtilServerPlayerMove.updateMoveSquares(getGameState(), actingPlayer.isJumping())
                // — computes per-square dodging/GFI flags for the fresh activation. Without this
                // the MoveSquare table stays empty, StepInitMoving never sets
                // acting_player.dodging, and no dodge is ever rolled.
                Self::apply_moving_activation_updates(game, player_id, pa);
                // Java: checkForStaller() called after CLIENT_ACTIVATE_PLAYER
                Self::check_for_staller(game);
                // Java: UtilServerGame.changePlayerAction syncs the activation to clients;
                // coverage counts activations per action type via GameEvent::PlayerAction.
                return self.execute_step(game, rng)
                    .with_event(GameEvent::PlayerAction { player_id: player_id.clone(), action: pa });
            }
            // Java: CLIENT_SYNCHRONOUS_MULTI_BLOCK (bb2025 shared StepInitSelecting:328-334) —
            // publish BLOCK_TARGETS, changePlayerAction(MULTIPLE_BLOCK), dispatch, EXECUTE.
            Action::MultiBlock { targets } => {
                if let Some(pid) = game.acting_player.player_id.clone() {
                    util_server_steps::change_player_action(game, &pid, PlayerAction::MultipleBlock, false);
                    self.dispatch_player_action = Some(PlayerAction::MultipleBlock);
                    return self.execute_step(game, rng)
                        // Java: publishParameter(BLOCK_TARGETS, command.getSelectedTargets()) -
                        // the client's list, verbatim, kinds and original states included.
                        .publish(StepParameter::BlockTargets(targets.clone()));
                }
                return self.execute_step(game, rng);
            }
            // Java: CLIENT_PASS (bb2025 shared StepInitSelecting:256-277) — publish the target
            // coordinate (transformed for the away side), change the action to PASS (unless the
            // acting action is already HAIL_MARY_PASS / THROW_BOMB / HAIL_MARY_BOMB, which keep
            // their own dispatch), dispatch, EXECUTE_STEP. NO passUsed gate — Java throws the
            // post-Treacherous pass even though markActionUsed already set passUsed. This arm is
            // the phase-2 continuation of the special-skill flow: after the stab the acting
            // player still carries PASS_MOVE and the client sends CLIENT_PASS.
            Action::Pass { coord } => {
                if let Some(pid) = game.acting_player.player_id.clone() {
                    // NO away-side transform: Java's CLIENT_PASS handler un-transforms the WIRE
                    // coordinate the away client sent, but the Rust agents answer prompts with
                    // REAL board coordinates (the BombRethrow/StepInitPassing::handle_command
                    // precedent). Transforming here threw the post-Treacherous pass at the
                    // mirrored square — (19,6) became (6,6), 15 squares out of range, and the
                    // turn ended with no roll (renegades bb2020 seed 85 i=142).
                    let target = *coord;
                    let dispatch = match game.acting_player.player_action {
                        Some(pa @ (PlayerAction::HailMaryPass
                            | PlayerAction::ThrowBomb
                            | PlayerAction::HailMaryBomb)) => pa,
                        _ => {
                            util_server_steps::change_player_action(game, &pid, PlayerAction::Pass, false);
                            PlayerAction::Pass
                        }
                    };
                    self.dispatch_player_action = Some(dispatch);
                    // Dispatch DIRECTLY: execute_step's no-defender deselect belongs to the
                    // FOLDED declaration model (a Pass declared with no receiver); this arm is
                    // Java's CLIENT_PASS, which always carries a target coordinate and always
                    // dispatches (routing it through execute_step ended the player action —
                    // renegades bb2020 seed 91: the post-Treacherous pass turned into the
                    // EndPlayerAction sequence).
                    return StepOutcome::goto(&self.goto_label_on_end)
                        .publish(StepParameter::DispatchPlayerAction(Some(dispatch)))
                        .publish(StepParameter::TargetCoordinate(target));
                }
                return self.execute_step(game, rng);
            }
            // Java: CLIENT_USE_SKILL — selected skills that are resolved immediately (SKIP_STEP).
            Action::UseSkill { skill_id, use_skill: true } => {
                // Java StepInitSelecting:354-378 — the star-special dispatch chain: a
                // CLIENT_USE_SKILL whose skill carries one of six properties turns into a
                // special-action dispatch (fDispatchPlayerAction = X, EXECUTE_STEP,
                // forceGotoOnDispatch). This is how the client's declare-action-then-
                // sendUseSkill pairing reaches the special sequences; there is no declared
                // TREACHEROUS/RAIDING_PARTY/... client action. The property check is on the
                // SKILL itself (each property is registered by exactly one mixed/special
                // skill in every edition, so the edition-agnostic union is safe here —
                // unlike the Ball & Chain case).
                {
                    use ffb_model::model::property::named_properties::NamedProperties as NP;
                    let props = skill_id.properties();
                    let special = if props.contains(&NP::CAN_STAB_TEAM_MATE_FOR_BALL) {
                        Some(PlayerAction::Treacherous)
                    } else if props.contains(&NP::CAN_MOVE_OPEN_TEAM_MATE) {
                        Some(PlayerAction::RaidingParty)
                    } else if props.contains(&NP::CAN_STEAL_BALL_FROM_OPPONENT) {
                        Some(PlayerAction::LookIntoMyEyes)
                    } else if props.contains(&NP::CAN_MAKE_OPPONENT_MISS_TURN) {
                        Some(PlayerAction::BalefulHex)
                    } else if props.contains(&NP::CAN_GET_BALL_ON_GROUND) {
                        Some(PlayerAction::CatchOfTheDay)
                    } else if props.contains(&NP::CAN_BLAST_REMOTE_PLAYER) {
                        Some(PlayerAction::ThenIStartedBlastin)
                    } else if props.contains(&NP::CAN_GAZE_AUTOMATICALLY) {
                        // Java :399 — Black Ink sits later in the chain, after canAddBlockDie.
                        Some(PlayerAction::BlackInk)
                    } else if props.contains(&NP::CAN_GAZE_AUTOMATICALLY_THREE_SQUARES_AWAY) {
                        Some(PlayerAction::AutoGazeZoat)
                    } else {
                        None
                    };
                    if let Some(pa) = special {
                        self.dispatch_player_action = Some(pa);
                        self.force_goto_on_dispatch = true;
                        return self.execute_step(game, rng);
                    }
                }
                let acting_player_id = game.acting_player.player_id.clone();
                // Collect skill property booleans before any mutable borrow of game.
                let (gain_hail_mary, avoid_dodging, add_block_die) = {
                    let p = acting_player_id.as_deref().and_then(|id| game.player(id));
                    p.map(|player| (
                        player.has_skill_property(NamedProperties::CAN_GAIN_HAIL_MARY) && player.has_skill(*skill_id),
                        player.has_skill_property(NamedProperties::CAN_AVOID_DODGING) && player.has_skill(*skill_id),
                        player.has_skill_property(NamedProperties::CAN_ADD_BLOCK_DIE) && player.has_skill(*skill_id),
                    )).unwrap_or((false, false, false))
                };
                if gain_hail_mary {
                    // Java: getResult().addReport(new ReportSkillUse(actingPlayer.getPlayerId(), skill, true, GAIN_HAIL_MARY))
                    game.report_list.add(ReportSkillUse::new(
                        acting_player_id.clone(),
                        *skill_id,
                        true,
                        SkillUse::GAIN_HAIL_MARY,
                    ));
                } else if avoid_dodging {
                    // Java: getResult().addReport(new ReportSkillUse(actingPlayer.getPlayerId(), skill, true, AVOID_DODGING))
                    game.report_list.add(ReportSkillUse::new(
                        acting_player_id.clone(),
                        *skill_id,
                        true,
                        SkillUse::AVOID_DODGING,
                    ));
                } else if add_block_die {
                    // Java: getResult().addReport(new ReportSkillUse(skill, true, ADD_BLOCK_DIE)) — no player_id
                    game.report_list.add(ReportSkillUse::new(
                        None,
                        *skill_id,
                        true,
                        SkillUse::ADD_BLOCK_DIE,
                    ));
                }
                return self.execute_step(game, rng);
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            // Java init(): UPDATE_PERSISTENCE (mandatory; stored for persistence only)
            StepParameter::UpdatePersistence(v) => { self.update_persistence = *v; true }
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            _ => false,
        }
    }
}

impl StepInitSelecting {
    /// Java `checkForStaller()`:
    ///
    /// ```java
    /// if (enableStallingCheck && !gameState.isStalling() && isConsideredStalling()) {
    ///     if (actingPlayer.getPlayerAction() != PlayerAction.FORGO) {
    ///         addReport(new ReportStallerDetected(actingPlayer.getPlayerId()));
    ///     }
    ///     gameState.stallingDetected();
    /// }
    /// ```
    ///
    /// This DETECTS a staller and raises the flag. Rust had the condition inverted -- it required
    /// `game.stalling` to be true already and never set it -- so the flag was never raised, and
    /// `StepStallingPlayer` (which is a faithful port, and reads exactly that flag) could never
    /// roll. Together with the two other gaps in the same rule that ITER65 fixed, the whole BB2025
    /// stalling check was dead: Java rolls a d6 for a lone carrier with an open path to the endzone
    /// and Rust rolled nothing, shifting every later die by one.
    /// Java `StepInitSelecting`, shared tail of the CLIENT_ACTING_PLAYER case: after the
    /// declaration branch (whichever it took) Java runs `updateMoveSquares` and, for a
    /// standing-up activation, charges the stand-up its movement.
    ///
    /// Extracted so the BLITZ_SELECT branch can run it too. That branch RETURNS EARLY, and Java
    /// does not - its `:114` arm sets the dispatch and falls through to exactly this code. The
    /// early return skipped the stand-up cost, so a prone blitzer reached its block with
    /// `current_move = 0` instead of `min(3, MA)`, owed no GO FOR IT, and the die Java spends on
    /// that rush was consumed by the block instead (khemri seed 38: block dice [6] vs [3], i.e.
    /// Pow vs Pushback and a defender left Standing).
    fn apply_moving_activation_updates(game: &mut Game, player_id: &str, pa: PlayerAction) {
        if pa.is_moving() || game.acting_player.standing_up {
            // Java StepInitSelecting: on a standing-up (was PRONE) activation without
            // canStandUpForFree, the stand-up consumes min(MINIMUM_MOVE_TO_STAND_UP=3, MA) of
            // movement — set current_move so the player can only move MA-3 more squares. Without
            // this a stood-up ball carrier ran its FULL MA (3 extra squares), diverging its final
            // position (seed 11 i=237: home_07 ended (14,2) in Rust vs (11,2) in Java).
            if game.acting_player.standing_up {
                let has_free = game.player(player_id)
                    .map(|p| p.has_skill_property(NamedProperties::CAN_STAND_UP_FOR_FREE))
                    .unwrap_or(false);
                let ma = game.player(player_id).map(|p| p.movement_with_modifiers()).unwrap_or(0);
                // A free stand-up (MA >= MINIMUM_MOVE_TO_STAND_UP=3, or canStandUpForFree) always
                // succeeds — apply STANDING now, BEFORE the activation's negatrait rolls (Bloodlust
                // etc.), so a failed negatrait that gotos the failure label and skips StepStandUp does
                // not leave the player prone. Java rolls the negatrait with the player already standing
                // (Bloodlust prone=false for a MA6 Vampire; vampire seed 1 i=43: a prone Vampire failing
                // Bloodlust ends STANDING in Java but stayed PRONE in Rust). MA<3 players still roll to
                // stand up in StepStandUp, so they are not pre-stood here.
                // The base is MOVING, not STANDING: Java's changeActingPlayer already put the
                // freshly activated player in MOVING ("show acting player as moving") and its
                // StepStandUp never writes STANDING on success — only a FAILED stand-up writes
                // PRONE. MOVING is Java's "standing" for every isStanding() test, and the acting
                // player must still read MOVING later in the activation: StepEndBlocking's
                // canMoveAfterBlock (Hit and Run) and canFoulAfterBlock (Pile Driver) both gate on
                // `playerState.getBase() == MOVING`, so writing STANDING here silently disabled
                // both for every player that stood up (amazon bb2020 seed 3 i=7: home_03's Hit
                // and Run window never opened in Rust while Java drove the move).
                if has_free || ma >= 3 {
                    if let Some(ps) = game.field_model.player_state(player_id) {
                        game.field_model.set_player_state(player_id, ps.change_base(ffb_model::enums::PS_MOVING));
                    }
                }
                if !has_free {
                    game.acting_player.current_move = 3.min(ma);
                    // Java: actingPlayer.setGoingForIt(UtilPlayer.isNextMoveGoingForIt(game)).
                    // When standing up consumes all remaining MA (e.g. MA-3 player), the next
                    // move is a rush; without this flag update_move_squares' is_next_move_possible
                    // returns false (extra_move=0 → current_move < MA) and CLEARS the freshly
                    // computed move squares — so StepInitMoving reads an empty table and never
                    // sets dodging/going_for_it (necromantic seed 38 i=116: away_03's stand-up
                    // rush+dodge was skipped, desyncing the RNG stream).
                    game.acting_player.goes_for_it =
                        ffb_model::util::util_player::UtilPlayer::is_next_move_going_for_it(game);
                }
            }
            crate::util::UtilServerPlayerMove::update_move_squares(game, game.acting_player.jumping);
        }
    }

    fn check_for_staller(game: &mut Game) {
        if !game.options.is_enabled("enableStallingCheck") || game.stalling {
            return;
        }
        let Some(pid) = game.acting_player.player_id.clone() else {
            return;
        };
        if !StallingExtension::new().is_considered_stalling(game, &pid) {
            return;
        }
        // Java: if (actingPlayer.getPlayerAction() != PlayerAction.FORGO)
        if !game.acting_player.forgone {
            game.report_list.add(ReportStallerDetected::new(Some(pid)));
        }
        // Java: gameState.stallingDetected()
        game.stalling = true;
    }

    fn execute_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let label = &self.goto_label_on_end;

        if self.end_turn {
            return StepOutcome::goto(label)
                .publish(StepParameter::EndTurn(true))
                .publish(StepParameter::CheckForgo(true));
        }
        if self.end_player_action {
            return StepOutcome::goto(label)
                .publish(StepParameter::EndPlayerAction(true));
        }
        if let Some(dispatch) = self.dispatch_player_action {
            if game.acting_player.player_id.is_some() {
                // Java ParityRunner Phase 2 (sendHandOverAction / sendPassAction): a PASS or
                // HAND_OVER activation whose target list is empty is deselected at the passing step
                // (ClientCommandActingPlayer(null,null,false)) — the player is activated but does
                // nothing, and the turn continues with the next player. Rust chooses the target at
                // activation time, so a no-receiver hand-over arrives here with no defender; without
                // this, StepInitPassing would run with no target coordinate, set no thrower, and
                // return Continue with no prompt — the drive stalls and the game ends early (seed 22
                // i=184: ball carrier away_04 whose only turn-start-adjacent teammate had moved off).
                // Same no-target deselect for Throw/Kick Team-Mate: an activation whose thrown-player
                // list is empty (no adjacent standing Right Stuff teammate) is deselected — the
                // reference harness does the same, and StepInitThrowTeamMate would otherwise Continue
                // with no player and stall.
                // Same for THROW_BOMB / HAIL_MARY_BOMB (Bombardier secret weapon): the reference
                // ParityRunner has no THROW_BOMB dispatch case, so it hits its `default:` branch and
                // deselects (ClientCommandActingPlayer(null,null,false)) — the player is activated
                // (recorded as a step) then does nothing, 0 dice, turn continues. Rust's random agent
                // likewise supplies no bomb target (a bomb targets an empty square, which the
                // block_defender_id/player-id channel cannot carry), so a bomb always arrives here with
                // no defender; without this it routes into the Pass sequence and StepInitPassing returns
                // Continue with no target coordinate and no prompt → the drive stalls (goblin seed 1
                // i=56: away_03 Bombardier ThrowBomb). Deselecting matches Java exactly.
                // Same for THROW_KEG: targets are opponents within 3 (ThrowKegLogicModule.
                // isValidTarget); when none exists the coach has no square to click, so the
                // declaration never completes and ParityRunner (both agent paths) deselects with
                // NO target draw. Rust's fold likewise returns None; without this the keg ROLLED
                // with a null target — a fumble (roll 1) even ended the thrower's team turn
                // (dwarf bb2025 seed 2 i=34, seed 3 i=22).
                if matches!(dispatch,
                        PlayerAction::HandOver | PlayerAction::Pass
                        | PlayerAction::ThrowTeamMate | PlayerAction::KickTeamMate
                        | PlayerAction::ThrowBomb | PlayerAction::HailMaryBomb
                        | PlayerAction::HailMaryPass | PlayerAction::ThrowKeg)
                    && game.defender_id.is_none()
                {
                    return StepOutcome::goto(label)
                        .publish(StepParameter::EndPlayerAction(true));
                }
                // A player that declared a BLITZ but has NO adjacent target: Java's SelectBlitzTarget
                // resolves the target (blockTarget == null → "BLITZ_TARGET_NONE") and ParityRunner ends
                // the turn (ClientCommandEndTurn) BEFORE any block sequence — so NO Bone-head / negatrait
                // is rolled, whether the blitzer is STANDING or PRONE (the EndTurn happens at target
                // selection, before the stand-up ACTIVATION either way). Rust otherwise dispatches the
                // block sequence, whose ACTIVATION rolls Bone-head — an extra game die that desyncs the
                // RNG stream for a negatrait carrier (human seed 7 i=196 standing Ogre → surfaced i=217;
                // human seed 36 i=170 PRONE Ogre → surfaced i=250). A skill-less lineman's ACTIVATION
                // rolls nothing here so this only ever removes a stray negatrait die. (Earlier this was
                // guarded to standing-only on the mistaken assumption a prone no-target blitz rolls
                // Bone-head during stand-up; seed 36 disproved it — Java rolls 0 dice in both cases.)
                if matches!(dispatch, PlayerAction::Blitz) && game.defender_id.is_none() {
                    return StepOutcome::goto(label)
                        .publish(StepParameter::EndTurn(true))
                        .publish(StepParameter::CheckForgo(true));
                }
                let standing_up = game.acting_player.standing_up;
                // Java `StepInitSelecting.executeStep` PARKS on a plain MOVE declaration: with
                // `fDispatchPlayerAction == null` it falls into the final `else`, runs
                // `prepareStandingUp()` and sets NO next action for a moving PlayerAction (only
                // REMOVE_CONFUSION / STAND_UP / STAND_UP_BLITZ get `NEXT_STEP`). The Select
                // sequence's activation block — BONE_HEAD, REALLY_STUPID, TAKE_ROOT,
                // UNCHANNELLED_FURY, BLOOD_LUST, JUMP_UP, STAND_UP — therefore runs only once a
                // CLIENT_MOVE arrives (`:185` then sets `fDispatchPlayerAction = MOVE`). A player
                // with no square to step into never gets that command: the client answers the move
                // window with `ClientCommandActingPlayer(null, null, false)`, which is
                // `fEndPlayerAction` → `GOTO END_SELECTING`, so Java rolls NO negatrait die at all
                // for that activation.
                //
                // Rust folds declaration and dispatch into one `ActivatePlayer`. For an
                // ALREADY-STANDING player that still matches Java: `goto(label)` pushes the Move
                // sequence and `StepInitMoving.execute_step` takes its own `end_player_action`
                // branch to END_MOVING, which is BEFORE `StepId::BoneHead` in the Move sequence.
                // The `standing_up` carve-out below is the one path that runs the activation block
                // immediately, so a PRONE negatrait carrier boxed in on all eight sides rolled a
                // Bone Head die Java never rolls (human bb2020 @0 seed 45 i=201: the away Ogre at
                // (13,7) with all eight neighbours occupied — one extra d6 that shifted every
                // later die and cost the seed).
                if standing_up
                    && matches!(dispatch, PlayerAction::Move)
                    && game.acting_player.player_id.as_deref()
                        .map(|pid| crate::legal_actions::legal_move_targets(game, pid).is_empty())
                        .unwrap_or(false)
                {
                    return StepOutcome::goto(label)
                        .publish(StepParameter::EndPlayerAction(true));
                }
                // Rust bridging: the agent chose its target at activation time
                // (Action::ActivatePlayer.block_defender_id → game.defender_id), whereas Java's
                // client sends it later via CLIENT_BLOCK/CLIENT_FOUL/CLIENT_PASS/CLIENT_HAND_OVER.
                // Thread it through to StepEndSelecting so the pushed action sequence starts with
                // the target already set instead of waiting forever for a command nothing sends.
                let mut target_params: Vec<StepParameter> = Vec::new();
                if let Some(def) = game.defender_id.clone() {
                    match dispatch {
                        PlayerAction::Block | PlayerAction::Blitz => {
                            target_params.push(StepParameter::BlockDefenderId(def));
                        }
                        PlayerAction::Foul => {
                            target_params.push(StepParameter::FoulDefenderId(def));
                        }
                        // A bomb travels the same route as a pass: it dispatches into the Pass
                        // sequence and `StepInitPassing` takes its target from TARGET_COORDINATE
                        // (Rust's stand-in for Java's CLIENT_PASS). Without ThrowBomb here the
                        // bomb reached that step with no coordinate, so thrower/throwerAction were
                        // never set and the step parked on `cont()` forever — both engines stalled
                        // three steps into goblin bb2025 seed 1.
                        PlayerAction::Pass | PlayerAction::HandOver | PlayerAction::ThrowBomb
                        | PlayerAction::HailMaryPass => {
                            if let Some(coord) = game.field_model.player_coordinate(&def) {
                                target_params.push(StepParameter::TargetCoordinate(coord));
                            }
                        }
                        // Throw/Kick Team-Mate: the agent chose the thrown player at activation time
                        // (block_defender_id → game.defender_id). Hand it to StepInitThrowTeamMate,
                        // which picks it up and then prompts for the target square.
                        PlayerAction::ThrowTeamMate => {
                            target_params.push(StepParameter::ThrownPlayerId(Some(def)));
                        }
                        PlayerAction::KickTeamMate => {
                            target_params.push(StepParameter::ThrownPlayerId(Some(def)));
                            target_params.push(StepParameter::IsKickedPlayer(true));
                        }
                        // Java: CLIENT_THROW_KEG publishes TARGET_PLAYER_ID (NOT defenderId) and
                        // sets fDispatchPlayerAction = THROW_KEG; StepEndSelecting reads
                        // target_player_id straight into ThrowKeg.SequenceParams. Java declares the
                        // keg in TWO commands — ActingPlayer(THROW_KEG) then
                        // ClientCommandThrowKeg(target) — and Rust folds them into one
                        // ActivatePlayer, so unfold the target back onto TARGET_PLAYER_ID here.
                        PlayerAction::ThrowKeg => {
                            target_params.push(StepParameter::TargetPlayerId(Some(def)));
                        }
                        _ => {}
                    }
                }
                // A prone player (standing_up) must run the Select sequence's StandUp before its
                // action, so it always proceeds via next() — even for Block/Blitz where
                // force_goto_on_dispatch is set. Java reaches the StandUp via SelectBlitzTarget (which
                // BLITZ_SELECT dispatches to); Rust skips SelectBlitzTarget and dispatches Blitz
                // straight to the block, so without this a prone blitzer with an adjacent target (no
                // move) never stood up. force_goto only skips StandUp for an already-standing player.
                //
                // §12: this standing_up carve-out is one more bridge built on "Rust skips
                // SelectBlitzTarget", and BLITZ_SELECT must be exempt from it. Java's :114 sets
                // forceGotoOnDispatch = true unconditionally. A prone blitzer still stands up:
                // NOT inside the SelectBlitzTarget sequence (the BB2025 ActivationSequenceBuilder
                // deliberately omits JUMP_UP/STAND_UP) but in the full Select sequence that
                // StepSelectBlitzTargetEnd pushes on the second pass, which carries both. Leaving
                // BLITZ_SELECT
                // inside the carve-out returned next(), the ordinary Select sequence carried on
                // into InitActivation, the chain was never pushed, and that blitz spent no target
                // draw - the single missing actionRng call behind the 26-red gate.
                let mut outcome = if standing_up && dispatch != PlayerAction::BlitzSelect {
                    StepOutcome::next()
                } else {
                    StepOutcome::goto(label)
                };
                outcome = outcome.publish(StepParameter::DispatchPlayerAction(Some(dispatch)));
                for p in target_params { outcome = outcome.publish(p); }
                return outcome;
            }
        }
        // Post-special continuation: the select sequence re-enters with the acting player
        // still set and carrying PASS_MOVE (the Treacherous bridging's declared action). Java
        // shows no dialog here — its client just sends CLIENT_PASS, and ParityRunner's phase 2
        // calls sendPassAction. Emit the BombRethrow prompt: despite the name it is the generic
        // "acting player must pass now" window, and both agents answer it with sendPassAction's
        // exact contract (all on-pitch teammates coordinate-sorted, 1 actionRng; empty list →
        // 2 decisionRng for a random square) → Action::Pass, which the CLIENT_PASS arm above
        // dispatches. Without this the step rebuilt the team-wide activation prompt and the
        // agent activated a DIFFERENT player while Java threw the pass (renegades bb2020 seed
        // 91 i=94: Java home_11 PASS_MOVE rolls 3 dice, Rust home_07 Move).
        if game.acting_player.player_action == Some(PlayerAction::PassMove) {
            if let Some(pid) = game.acting_player.player_id.clone() {
                return StepOutcome::cont()
                    .with_prompt(AgentPrompt::BombRethrow { player_id: pid });
            }
        }
        // MULTIPLE_BLOCK declaration window: the declared player must pick TWO targets — Java
        // waits for CLIENT_SYNCHRONOUS_MULTI_BLOCK. (Same window also guarded in start() for
        // the select-sequence re-entry.)
        if game.acting_player.player_action == Some(PlayerAction::MultipleBlock) {
            if let Some(pid) = game.acting_player.player_id.clone() {
                if let Some(coord) = game.field_model.player_coordinate(&pid) {
                    let inactive = game.inactive_team();
                    let mut elig: Vec<String> =
                        ffb_model::util::util_player::UtilPlayer::find_adjacent_blockable_players(
                            game, inactive, coord,
                        ).into_iter().cloned().collect();
                    elig.sort_by_key(|id| {
                        game.field_model.player_coordinate(id)
                            .map(|c| (c.x, c.y)).unwrap_or((i32::MAX, i32::MAX))
                    });
                    return StepOutcome::cont().with_prompt(
                        AgentPrompt::MultiBlockTargets { player_id: pid, eligible_players: elig });
                }
            }
        }
        // Waiting: build activation prompt
        let eligible = crate::legal_actions::eligible_players_for_activation(game);
        StepOutcome::cont()
            .with_prompt(AgentPrompt::ActivatePlayer { eligible_players: eligible })
    }
}

/// Maps `PlayerActionChoice` (Rust engine action) to `PlayerAction` (model enum).
fn pac_to_player_action(pac: PlayerActionChoice) -> PlayerAction {
    match pac {
        PlayerActionChoice::Move => PlayerAction::Move,
        PlayerActionChoice::Blitz => PlayerAction::Blitz,
        PlayerActionChoice::Block => PlayerAction::Block,
        PlayerActionChoice::Stab => PlayerAction::Stab,
        PlayerActionChoice::Foul => PlayerAction::Foul,
        PlayerActionChoice::Pass => PlayerAction::Pass,
        PlayerActionChoice::HandOff => PlayerAction::HandOver,
        // The real client's declarations: move first, then give/throw.
        PlayerActionChoice::HandOffMove => PlayerAction::HandOverMove,
        PlayerActionChoice::PassMove => PlayerAction::PassMove,
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
        PlayerActionChoice::FuriousOutburst => PlayerAction::FuriousOutburst,
        PlayerActionChoice::ThrowKeg => PlayerAction::ThrowKeg,
        PlayerActionChoice::WisdomOfTheWhiteDwarf => PlayerAction::WisdomOfTheWhiteDwarf,
        PlayerActionChoice::AutoGazeZoat => PlayerAction::AutoGazeZoat,
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
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    /// Java `StepInitSelecting.executeStep` sets NO next action for a plain MOVE declaration (only
    /// REMOVE_CONFUSION / STAND_UP / STAND_UP_BLITZ get `NEXT_STEP`), so the Select sequence's
    /// activation block — BONE_HEAD, REALLY_STUPID, TAKE_ROOT, UNCHANNELLED_FURY, BLOOD_LUST,
    /// JUMP_UP, STAND_UP — runs only once a CLIENT_MOVE arrives (`:185`). A player with no square
    /// to step into never sends one: the client answers the move window with
    /// `ClientCommandActingPlayer(null, null, false)` → `fEndPlayerAction` → `GOTO END_SELECTING`,
    /// and Java rolls NO negatrait die for the activation.
    ///
    /// Rust folds declaration and dispatch into one `ActivatePlayer`, and the `standing_up`
    /// carve-out runs that block immediately — one extra Bone Head die for a prone Ogre boxed in on
    /// all eight sides (human bb2020 @0 seed 45 i=201).
    #[test]
    fn prone_move_with_no_move_targets_deselects_instead_of_running_the_activation_block() {
        use ffb_model::enums::{PS_PRONE, PS_STANDING, PlayerState as PSt, PlayerType, PlayerGender};
        use ffb_model::model::player::Player;
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        let mut mk = |id: &str, nr: i32, home: bool, x: i32, y: i32, prone: bool, g: &mut Game| {
            let p = Player {
                id: id.into(), name: id.into(), nr, position_id: "pos".into(),
                player_type: PlayerType::Regular, gender: PlayerGender::Male,
                movement: 5, strength: 3, agility: 3, passing: 4, armour: 9,
                ..Default::default()
            };
            if home { g.team_home.players.push(p); } else { g.team_away.players.push(p); }
            g.field_model.set_player_coordinate(id, FieldCoordinate::new(x, y));
            g.field_model.set_player_state(id, PSt::new(if prone { PS_PRONE } else { PS_STANDING }));
        };
        mk("mover", 1, true, 13, 7, true, &mut game);
        let mut nr = 2;
        for (dx, dy) in [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)] {
            mk(&format!("box{nr}"), nr, nr % 2 == 0, 13 + dx, 7 + dy, false, &mut game);
            nr += 1;
        }
        game.home_playing = true;
        game.acting_player.set_player("mover".into(), PlayerAction::Move);
        game.acting_player.standing_up = true;
        let mut step = StepInitSelecting::new("end".into());
        step.dispatch_player_action = Some(PlayerAction::Move);
        let out = step.execute_step(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel,
            "Java's deselect is GOTO END_SELECTING, not NEXT_STEP into the activation block");
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))),
            "the deselect publishes END_PLAYER_ACTION, got {:?}", out.published);
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::DispatchPlayerAction(_))),
            "no action is dispatched: the declaration never completed");
    }

    #[test]
    fn start_returns_cont() {
        let mut game = make_game();
        let mut step = StepInitSelecting::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn set_parameter_end_turn_accepted() {
        let mut step = StepInitSelecting::new("end".into());
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn set_parameter_end_player_action_accepted() {
        let mut step = StepInitSelecting::new("end".into());
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
        assert!(step.end_player_action);
    }

    #[test]
    fn start_emits_activate_player_prompt() {
        let mut game = make_game();
        let mut step = StepInitSelecting::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.prompt, Some(AgentPrompt::ActivatePlayer { .. })));
    }

    /// §9 UseSkill special chain (Java StepInitSelecting:354-378): a CLIENT_USE_SKILL whose
    /// skill carries canStabTeamMateForBall dispatches TREACHEROUS (forceGotoOnDispatch), and
    /// the TREACHEROUS "declaration" bridges the client's two-command pair (PASS_MOVE acting
    /// player + UseSkill).
    #[test]
    fn use_skill_with_stab_property_dispatches_treacherous() {
        use ffb_model::enums::{PlayerType, PlayerGender, PlayerState, PS_STANDING, SkillId};
        use ffb_model::model::player::Player;
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        game.home_playing = true;
        game.team_home.players.push(Player {
            id: "h1".into(), name: "h1".into(), nr: 1, position_id: "star".into(),
            player_type: PlayerType::Star, gender: PlayerGender::Male,
            movement: 9, strength: 3, agility: 2, passing: 3, armour: 8,
            starting_skills: vec![SkillWithValue::new(SkillId::Treacherous)],
            ..Default::default()
        });
        game.field_model.set_player_coordinate("h1", FieldCoordinate::new(12, 7));
        game.field_model.set_player_state("h1", PlayerState::new(PS_STANDING));

        // Route A: the bridged declaration.
        let mut step = StepInitSelecting::new("end".into());
        let out = step.handle_command(&Action::ActivatePlayer {
            player_id: "h1".into(),
            player_action: PlayerActionChoice::Treacherous,
            block_defender_id: None,
        }, &mut game, &mut GameRng::new(0));
        assert_eq!(step.dispatch_player_action, Some(PlayerAction::Treacherous));
        assert!(out.published.iter().any(|p| matches!(p,
            StepParameter::DispatchPlayerAction(Some(PlayerAction::Treacherous)))),
            "the outcome must publish the TREACHEROUS dispatch");
        assert_eq!(game.acting_player.player_action, Some(PlayerAction::PassMove),
            "the acting player carries the client's PASS_MOVE declaration");

        // Route B: the raw UseSkill command (what ParityRunner injects).
        let mut step2 = StepInitSelecting::new("end".into());
        crate::step::util_server_steps::change_player_action(
            &mut game, "h1", PlayerAction::PassMove, false);
        let out2 = step2.handle_command(&Action::UseSkill {
            skill_id: SkillId::Treacherous, use_skill: true,
        }, &mut game, &mut GameRng::new(0));
        assert_eq!(step2.dispatch_player_action, Some(PlayerAction::Treacherous));
        assert!(step2.force_goto_on_dispatch);
        let _ = out2;
    }

    /// The activation pre-stand writes MOVING, not STANDING. Java's `changeActingPlayer` puts the
    /// freshly activated player in MOVING ("show acting player as moving") and its `StepStandUp`
    /// never writes STANDING on success. MOVING satisfies every `isStanding()` test and the
    /// activation-end `changeActingPlayer` reverts it to STANDING, so the compared state hash is
    /// unaffected — but MID-activation `StepEndBlocking` gates both `canMoveAfterBlock` (Hit and
    /// Run) and `canFoulAfterBlock` (Pile Driver) on `base == MOVING`, so STANDING here silently
    /// disabled both for any player that stood up (amazon bb2020 seed 3 i=7).
    #[test]
    fn a_prone_player_activated_for_a_move_is_left_moving_not_standing() {
        use ffb_model::enums::{PlayerType, PlayerGender, PlayerState, PS_PRONE, PS_MOVING};
        use ffb_model::model::player::Player;
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        game.home_playing = true;
        game.team_home.players.push(Player {
            id: "h1".into(), name: "h1".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("h1", FieldCoordinate::new(12, 7));
        game.field_model.set_player_state("h1", PlayerState::new(PS_PRONE));

        let mut step = StepInitSelecting::new("end".into());
        let out = step.handle_command(&Action::ActivatePlayer {
            player_id: "h1".into(),
            player_action: PlayerActionChoice::Move,
            block_defender_id: None,
        }, &mut game, &mut GameRng::new(0));
        let _ = out;

        assert!(game.acting_player.standing_up, "a PRONE activation is standing up");
        assert_eq!(game.field_model.player_state("h1").unwrap().base(), PS_MOVING,
            "the pre-stand must leave the acting player MOVING, not STANDING");
    }

    #[test]
    fn blitz_declaration_dispatches_blitz_select_not_a_folded_end_turn() {
        // Rewritten for §12. This test used to assert that a no-target Blitz publishes EndTurn
        // FROM THIS STEP (regressions: human seed 7 i=196 standing, seed 36 i=170 prone). That was
        // correct only while Rust folded the blitz target into the declaration and skipped Java's
        // two-phase chain entirely. Java's :114 branch dispatches BLITZ_SELECT on every untargeted
        // blitz; the no-target turn-end is then decided far downstream, by the agent answering
        // StepSelectBlitzTarget's dialog the way ParityRunner.sendBlitzTargetSelection does
        // (BLITZ_TARGET_NONE -> ClientCommandEndTurn).
        //
        // So the ORIGINAL INTENT still holds - a no-target blitz ends the turn without rolling a
        // stray Bone-head - but it is now enforced where Java enforces it. That half is pinned by
        // random_agent::tests::blitz_target_prompt_with_no_candidates_ends_the_turn and by
        // step_select_blitz_target's two predicate tests. What belongs HERE is only the routing:
        // the declaration must reach BLITZ_SELECT and must NOT resolve the blitz itself.
        use ffb_model::enums::{PlayerType, PlayerGender, PlayerState, PS_STANDING};
        use ffb_model::model::player::Player;
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        game.home_playing = true;
        game.team_home.players.push(Player {
            id: "h1".into(), name: "h1".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("h1", FieldCoordinate::new(12, 7));
        game.field_model.set_player_state("h1", PlayerState::new(PS_STANDING));

        let mut step = StepInitSelecting::new("end".into());
        let action = Action::ActivatePlayer {
            player_id: "h1".into(),
            player_action: PlayerActionChoice::Blitz,
            block_defender_id: None, // no adjacent target
        };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        let _ = out;

        assert_eq!(game.acting_player.player_action, Some(PlayerAction::BlitzMove),
            "Java's :114 branch keeps the ACTING action BLITZ_MOVE while dispatching BLITZ_SELECT");
        assert!(!game.turn_data().blitz_used,
            "the team blitz is consumed by StepSelectBlitzTargetEnd, not by the declaration");

        // Regression (human seed 36 i=170): a PRONE no-target Blitz must ALSO EndTurn before the block
        // sequence — Java rolls 0 dice in both cases (the earlier standing-only guard was wrong; a
        // prone Ogre otherwise rolled a stray Bone-head that desynced the RNG at i=250).
        use ffb_model::enums::PS_PRONE;
        let mut game2 = make_game();
        game2.home_playing = true;
        game2.team_home.players.push(Player {
            id: "h1".into(), name: "h1".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        });
        game2.field_model.set_player_coordinate("h1", FieldCoordinate::new(12, 7));
        game2.field_model.set_player_state("h1", PlayerState::new(PS_PRONE));
        let mut step2 = StepInitSelecting::new("end".into());
        let out2 = step2.handle_command(&Action::ActivatePlayer {
            player_id: "h1".into(), player_action: PlayerActionChoice::Blitz, block_defender_id: None,
        }, &mut game2, &mut GameRng::new(0));
        let _ = out2;
        assert_eq!(game2.acting_player.player_action, Some(PlayerAction::BlitzMove),
            "a PRONE blitz declaration routes through BLITZ_SELECT too");
        assert!(!game2.turn_data().blitz_used);
    }

    #[test]
    fn end_turn_returns_goto_label_with_end_turn_param() {
        let mut game = make_game();
        let mut step = StepInitSelecting::new("end_label".into());
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end_label"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CheckForgo(true))));
    }

    #[test]
    fn end_player_action_returns_goto_label() {
        let mut game = make_game();
        let mut step = StepInitSelecting::new("end_label".into());
        step.end_player_action = true;
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end_label"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))));
    }

    #[test]
    fn activate_player_move_sets_dispatch_and_returns_goto_label() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        let mut step = StepInitSelecting::new("end_label".into());
        let action = Action::ActivatePlayer {
            player_id: "p1".into(),
            player_action: PlayerActionChoice::Move,
            block_defender_id: None,
        };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end_label"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::DispatchPlayerAction(_))));
    }

    /// A hand-over's receiver IS parked in `game.defender_id` by the activation bridge, and must
    /// stay there — the pass sequence reads it downstream. Clearing it here was tried and measured
    /// goblin 100/100 -> 4/100. The leak it caused (Animal Savagery lashing out at the receiver) is
    /// fixed where Java's `getDefender()` would be null instead: see
    /// `step_animal_savagery::adjacent_targets`.
    #[test]
    fn hand_over_keeps_its_receiver_as_the_bridged_defender() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        let mut step = StepInitSelecting::new("end_label".into());
        let action = Action::ActivatePlayer {
            player_id: "p1".into(),
            player_action: PlayerActionChoice::HandOff,
            block_defender_id: Some("p2".into()),
        };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(game.defender_id.as_deref(), Some("p2"));
    }

    /// A BLOCK genuinely has a defender and must keep it.
    #[test]
    fn block_keeps_its_defender() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        let mut step = StepInitSelecting::new("end_label".into());
        let action = Action::ActivatePlayer {
            player_id: "p1".into(),
            player_action: PlayerActionChoice::Block,
            block_defender_id: Some("p2".into()),
        };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(game.defender_id.as_deref(), Some("p2"));
    }

    #[test]
    fn hand_over_activation_without_receiver_deselects() {
        use ffb_model::enums::PlayerAction;
        // Java ParityRunner.sendHandOverAction: a HAND_OVER whose adjacent-teammate list is empty is
        // deselected (ClientCommandActingPlayer(null,null,false)). Rust picks the receiver at
        // activation time, so a no-receiver hand-off arrives with block_defender_id == None; the
        // dispatch must EndPlayerAction rather than push a passing sequence StepInitPassing can
        // never complete (it would stall with no target coordinate — seed 22 i=184).
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        // A stale defender from an earlier activation must not resurrect the hand-over.
        game.defender_id = Some("stale".into());
        let mut step = StepInitSelecting::new("end_label".into());
        let action = Action::ActivatePlayer {
            player_id: "p1".into(),
            player_action: PlayerActionChoice::HandOff,
            block_defender_id: None,
        };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end_label"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))),
            "no-receiver hand-over must deselect");
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::DispatchPlayerAction(Some(PlayerAction::HandOver)))),
            "no-receiver hand-over must NOT dispatch a passing sequence");
    }

    #[test]
    fn throw_bomb_activation_without_target_deselects() {
        use ffb_model::enums::PlayerAction;
        // Bombardier THROW_BOMB: the reference ParityRunner has no THROW_BOMB dispatch case, so it
        // deselects (default branch). Rust's random agent supplies no bomb target, so the activation
        // arrives with block_defender_id == None; it must EndPlayerAction rather than route into the
        // Pass sequence (StepInitPassing would stall with no target coordinate — goblin seed 1 i=56).
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.defender_id = Some("stale".into());
        let mut step = StepInitSelecting::new("end_label".into());
        let action = Action::ActivatePlayer {
            player_id: "p1".into(),
            player_action: PlayerActionChoice::ThrowBomb,
            block_defender_id: None,
        };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end_label"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))),
            "target-less ThrowBomb must deselect");
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::DispatchPlayerAction(Some(PlayerAction::ThrowBomb)))),
            "target-less ThrowBomb must NOT dispatch a passing sequence");
    }

    #[test]
    fn throw_keg_activation_without_target_deselects() {
        use ffb_model::enums::PlayerAction;
        // Java: a THROW_KEG whose valid-target list (opponents within 3, STANDING) is empty never
        // completes — the coach has no square to click and ParityRunner (both agent paths)
        // deselects with NO target draw. Rust's fold likewise returns None; the activation must
        // EndPlayerAction rather than roll a targetless keg (a fumble even ended the thrower's
        // team turn — dwarf bb2025 seed 2 i=34 / seed 3 i=22).
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.defender_id = None;
        let mut step = StepInitSelecting::new("end_label".into());
        let action = Action::ActivatePlayer {
            player_id: "p1".into(),
            player_action: PlayerActionChoice::ThrowKeg,
            block_defender_id: None,
        };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end_label"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))),
            "target-less ThrowKeg must deselect");
        assert!(out.pushes.is_empty(), "no keg sequence may be pushed for a targetless keg");
    }

    /// Rewritten for §12. This used to assert that a prone Blitz proceeds via `next()` so the
    /// ordinary Select sequence's StandUp runs, which was right only while Rust skipped
    /// SelectBlitzTarget and dispatched the block directly. Java's :114 sets
    /// forceGotoOnDispatch = true unconditionally, so a prone blitzer GOTOs like any other and
    /// stands up later - in the full Select sequence StepSelectBlitzTargetEnd pushes on the
    /// second pass (the BB2025 SelectBlitzTarget sequence itself omits JUMP_UP/STAND_UP).
    ///
    /// Keeping the old carve-out returned next(), so the ordinary Select sequence carried on into
    /// InitActivation, the chain was never pushed, and the blitz spent no target draw - the one
    /// missing actionRng call behind the 26-red bb2025 gate (chaos_dwarf 79 -> 94 when fixed).
    #[test]
    fn activate_prone_player_blitz_dispatches_blitz_select_and_gotos() {
        use ffb_model::enums::{PS_PRONE, PlayerState};
        let mut game = make_game();
        game.field_model.set_player_state("p1", PlayerState::new(PS_PRONE));
        // NOTE: the acting player must NOT already be "p1" — Java sets standingUp only inside
        // `if (newPlayer != oldPlayer)`, so pre-assigning the same id would make this a re-dispatch.
        game.acting_player.player_id = None;
        let mut step = StepInitSelecting::new("end_label".into());
        let action = Action::ActivatePlayer {
            player_id: "p1".into(),
            player_action: PlayerActionChoice::Blitz,
            block_defender_id: Some("def".into()),
        };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(game.acting_player.standing_up, "prone blitz with a target sets standing_up");
        assert_eq!(out.action, StepAction::GotoLabel,
            "BLITZ_SELECT gotos even when standing_up - the chain does the stand-up");
        assert!(out.published.iter().any(|p|
            matches!(p, StepParameter::DispatchPlayerAction(Some(PlayerAction::BlitzSelect)))),
            "a prone blitz must still route through BLITZ_SELECT");
    }

    #[test]
    fn activate_prone_player_blitz_no_target_suppresses_standup() {
        use ffb_model::enums::{PS_PRONE, PlayerState};
        // A prone Blitz with NO target must NOT stand up: Java resolves the blitz target before the
        // stand-up, so a null-target blitz (BLITZ_TARGET_NONE) ends the turn with the player prone.
        let mut game = make_game();
        game.field_model.set_player_state("p1", PlayerState::new(PS_PRONE));
        game.acting_player.player_id = Some("p1".into());
        let mut step = StepInitSelecting::new("end_label".into());
        let action = Action::ActivatePlayer {
            player_id: "p1".into(),
            player_action: PlayerActionChoice::Blitz,
            block_defender_id: None,
        };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(!game.acting_player.standing_up, "no-target blitz must not stand the player up");
        assert_eq!(out.action, StepAction::GotoLabel, "no-target blitz force-gotos to the (no-defender) block");
    }

    #[test]
    fn prone_move_activation_sets_current_move_to_stand_up_cost() {
        use ffb_model::enums::{PS_PRONE, PlayerState, PlayerType, PlayerGender};
        use ffb_model::model::player::Player;
        use ffb_model::types::FieldCoordinate;
        // Java StepInitSelecting: a prone (standing_up) player activated for a Move without
        // canStandUpForFree gets current_move = min(MINIMUM_MOVE_TO_STAND_UP=3, MA). The stand-up
        // costs 3 movement, so an MA-4 player can move only 1 more square. Without this the stood-up
        // carrier ran its full MA (3 extra squares), diverging its final position (seed 11 i=237).
        let mut game = make_game();
        game.team_home.players.push(Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("p1", PlayerState::new(PS_PRONE));
        // NOTE: the acting player must NOT already be "p1" — Java sets standingUp only inside
        // `if (newPlayer != oldPlayer)`, so pre-assigning the same id would make this a re-dispatch.
        game.acting_player.player_id = None;
        let mut step = StepInitSelecting::new("end_label".into());
        let action = Action::ActivatePlayer {
            player_id: "p1".into(),
            player_action: PlayerActionChoice::Move,
            block_defender_id: None,
        };
        let _ = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(game.acting_player.standing_up, "prone Move activation stands the player up");
        assert_eq!(game.acting_player.current_move, 3, "stand-up costs min(3, MA=4) = 3 movement");
    }

    #[test]
    fn activate_standing_player_blitz_force_gotos() {
        use ffb_model::enums::{PS_STANDING, PlayerState};
        // A standing player's Blitz still force-gotos (skips StandUp) — the fix only changes prone.
        let mut game = make_game();
        game.field_model.set_player_state("p1", PlayerState::new(PS_STANDING));
        game.acting_player.player_id = Some("p1".into());
        let mut step = StepInitSelecting::new("end_label".into());
        let action = Action::ActivatePlayer {
            player_id: "p1".into(),
            player_action: PlayerActionChoice::Blitz,
            block_defender_id: None,
        };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(!game.acting_player.standing_up);
        assert_eq!(out.action, StepAction::GotoLabel, "standing blitz force-gotos to the block");
    }

    #[test]
    fn activate_player_block_sets_force_goto_on_dispatch() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        let mut step = StepInitSelecting::new("end".into());
        let action = Action::ActivatePlayer {
            player_id: "p1".into(),
            player_action: PlayerActionChoice::Block,
            block_defender_id: Some("def".into()),
        };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(step.force_goto_on_dispatch);
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn use_skill_hail_mary_adds_report() {
        use ffb_model::enums::{PlayerType, PlayerGender};
        use ffb_model::model::player::Player;
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::report::report_id::ReportId;
        use std::collections::HashSet;
        let mut game = make_game();
        // Add a player with ShotToNothing (canGainHailMary property — NOT HailMaryPass,
        // which registers canPassToAnySquare instead; see skill_id.rs properties()).
        game.team_home.players.push(Player {
            id: "hm1".into(), name: "HM".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue { skill_id: ffb_model::enums::SkillId::ShotToNothing, value: None }],
            extra_skills: vec![], temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.home_playing = true;
        game.acting_player.player_id = Some("hm1".into());
        let mut step = StepInitSelecting::new("end".into());
        step.handle_command(
            &Action::UseSkill { skill_id: ffb_model::enums::SkillId::ShotToNothing, use_skill: true },
            &mut game,
            &mut GameRng::new(0),
        );
        assert!(game.report_list.has_report(ReportId::SKILL_USE),
            "expected SKILL_USE report for ShotToNothing (GAIN_HAIL_MARY)");
    }

    /// `checkForStaller` DETECTS a staller and raises the flag; it does not react to a flag that
    /// is already up.
    ///
    /// This test previously asserted the opposite — it set `game.stalling = true` and expected a
    /// report — which is what the Rust code did and Java does not. Java's guard is
    /// `!gameState.isStalling() && isConsideredStalling()`, and the body ENDS with
    /// `gameState.stallingDetected()`. With the condition inverted the flag was never raised, so
    /// `StepStallingPlayer` (a faithful port that reads exactly that flag) could never roll, and
    /// the whole BB2025 stalling rule was dead.
    #[test]
    fn check_for_staller_raises_the_flag_for_a_lone_carrier() {
        use ffb_model::enums::{PlayerState, PS_STANDING};
        use ffb_model::model::player::Player;
        use ffb_model::report::report_id::ReportId;
        use ffb_model::types::FieldCoordinate;

        let mut game = make_game();
        game.options.set("enableStallingCheck", "true");
        game.home_playing = true;
        game.team_home.players.push(Player {
            id: "p1".into(), name: "P1".into(), nr: 1, position_id: "pos".into(),
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        });
        // A lone carrier with nobody near him and a clear run at the away endzone: stalling.
        // WITHIN his movement -- Java's `hasOpenPathToEndzone` asks the pathfinder for a route of
        // at most MA squares (`getShortestPath(..., player, 0)`), so "open path" means he could
        // score THIS turn and chose not to. From (5,5) the endzone is 20 squares away and no
        // carrier is ever stalling.
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(20, 7));
        game.field_model
            .set_player_state("p1", PlayerState::new(PS_STANDING).change_active(true));
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(20, 7));
        game.field_model.ball_in_play = true;
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.forgone = false;

        assert!(!game.stalling, "the flag starts down");
        StepInitSelecting::check_for_staller(&mut game);
        assert!(game.stalling, "detecting a staller must RAISE the flag");
        assert!(game.report_list.has_report(ReportId::STALLER_DETECTED));

        // Already flagged: Java's `!isStalling()` guard makes a second call a no-op.
        let reports_before = game.report_list.size();
        StepInitSelecting::check_for_staller(&mut game);
        assert_eq!(game.report_list.size(), reports_before,
            "a second call must not re-report while the flag is up");

        // The option off is the other half of Java's guard.
        let mut off = make_game();
        off.home_playing = true;
        off.acting_player.player_id = Some("p1".into());
        StepInitSelecting::check_for_staller(&mut off);
        assert!(!off.stalling, "with enableStallingCheck off nothing is detected");
    }

    #[test]
    fn pac_to_player_action_all_variants() {
        assert_eq!(pac_to_player_action(PlayerActionChoice::Move), PlayerAction::Move);
        assert_eq!(pac_to_player_action(PlayerActionChoice::Block), PlayerAction::Block);
        assert_eq!(pac_to_player_action(PlayerActionChoice::Foul), PlayerAction::Foul);
        assert_eq!(pac_to_player_action(PlayerActionChoice::HandOff), PlayerAction::HandOver);
    }
}
