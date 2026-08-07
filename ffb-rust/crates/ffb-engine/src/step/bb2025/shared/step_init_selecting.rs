use ffb_model::enums::PlayerAction;
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::prompts::AgentPrompt;
use ffb_model::report::mixed::report_staller_detected::ReportStallerDetected;
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
            Action::ActivatePlayer { player_id, player_action, block_defender_id } => {
                let pa = pac_to_player_action(*player_action);
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
                self.dispatch_player_action = Some(pa);
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
                        if has_free || ma >= 3 {
                            if let Some(ps) = game.field_model.player_state(player_id) {
                                game.field_model.set_player_state(player_id, ps.change_base(ffb_model::enums::PS_STANDING));
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
                // Java: checkForStaller() called after CLIENT_ACTIVATE_PLAYER
                Self::check_for_staller(game);
                // Java: UtilServerGame.changePlayerAction syncs the activation to clients;
                // coverage counts activations per action type via GameEvent::PlayerAction.
                return self.execute_step(game, rng)
                    .with_event(GameEvent::PlayerAction { player_id: player_id.clone(), action: pa });
            }
            // Java: CLIENT_USE_SKILL — selected skills that are resolved immediately (SKIP_STEP).
            Action::UseSkill { skill_id, use_skill: true } => {
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
    /// Java: `checkForStaller()` — if game is already marked stalling (game.stalling==true),
    /// emit `ReportStallerDetected` for the acting player (unless they are forgone).
    fn check_for_staller(game: &mut Game) {
        if game.stalling {
            let player_id = game.acting_player.player_id.clone();
            if let Some(pid) = player_id {
                let forgo = game.acting_player.forgone;
                if !forgo {
                    // Java: if (actingPlayer.getPlayerAction() != PlayerAction.FORGO)
                    game.report_list.add(ReportStallerDetected::new(Some(pid)));
                }
            }
        }
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
                if matches!(dispatch,
                        PlayerAction::HandOver | PlayerAction::Pass
                        | PlayerAction::ThrowTeamMate | PlayerAction::KickTeamMate)
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
                        PlayerAction::Pass | PlayerAction::HandOver => {
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
                        _ => {}
                    }
                }
                // A prone player (standing_up) must run the Select sequence's StandUp before its
                // action, so it always proceeds via next() — even for Block/Blitz where
                // force_goto_on_dispatch is set. Java reaches the StandUp via SelectBlitzTarget (which
                // BLITZ_SELECT dispatches to); Rust skips SelectBlitzTarget and dispatches Blitz
                // straight to the block, so without this a prone blitzer with an adjacent target (no
                // move) never stood up. force_goto only skips StandUp for an already-standing player.
                let mut outcome = if standing_up {
                    StepOutcome::next()
                } else {
                    StepOutcome::goto(label)
                };
                outcome = outcome.publish(StepParameter::DispatchPlayerAction(Some(dispatch)));
                for p in target_params { outcome = outcome.publish(p); }
                return outcome;
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
        PlayerActionChoice::StandUp => PlayerAction::StandUp,
        PlayerActionChoice::StandUpBlitz => PlayerAction::StandUpBlitz,
        PlayerActionChoice::ThrowTeamMate => PlayerAction::ThrowTeamMate,
        PlayerActionChoice::KickTeamMate => PlayerAction::KickTeamMate,
        PlayerActionChoice::HypnoticGaze => PlayerAction::Gaze,
        PlayerActionChoice::ThrowBomb => PlayerAction::ThrowBomb,
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

    #[test]
    fn standing_blitz_with_no_target_ends_the_turn_without_dispatch() {
        // Regression (human seed 7 i=196): a STANDING player that declares a Blitz with no adjacent
        // target must END THE TURN before any block sequence — so no Bone-head/negatrait die is
        // rolled — mirroring Java's SelectBlitzTarget → BLITZ_TARGET_NONE → EndTurn. Rust otherwise
        // dispatched the block sequence whose ACTIVATION rolled an extra Bone-head, desyncing the RNG.
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

        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))),
            "a standing no-target Blitz must publish EndTurn (no block sequence / Bone-head)");
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::DispatchPlayerAction(_))),
            "must NOT dispatch a block sequence for a standing no-target Blitz");

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
        assert_eq!(out2.action, StepAction::GotoLabel);
        assert!(out2.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))),
            "a PRONE no-target Blitz must also publish EndTurn (no stand-up / Bone-head)");
        assert!(!out2.published.iter().any(|p| matches!(p, StepParameter::DispatchPlayerAction(_))),
            "must NOT dispatch a block sequence for a prone no-target Blitz");
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
    fn activate_prone_player_blitz_with_target_runs_standup_via_next() {
        use ffb_model::enums::{PS_PRONE, PlayerState};
        // A prone player activated for a Blitz WITH a target must proceed via next() so the Select
        // sequence's StandUp runs (Java reaches it through SelectBlitzTarget). Without this the
        // force_goto for Block/Blitz jumped straight to the block and the prone blitzer never stood up.
        let mut game = make_game();
        game.field_model.set_player_state("p1", PlayerState::new(PS_PRONE));
        game.acting_player.player_id = Some("p1".into());
        let mut step = StepInitSelecting::new("end_label".into());
        let action = Action::ActivatePlayer {
            player_id: "p1".into(),
            player_action: PlayerActionChoice::Blitz,
            block_defender_id: Some("def".into()),
        };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(game.acting_player.standing_up, "prone blitz with a target sets standing_up");
        assert_eq!(out.action, StepAction::NextStep, "prone blitz proceeds to StandUp via next()");
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::DispatchPlayerAction(_))));
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
        game.acting_player.player_id = Some("p1".into());
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

    #[test]
    fn staller_detected_report_added_when_stalling() {
        use ffb_model::enums::{PlayerType, PlayerGender};
        use ffb_model::model::player::Player;
        use ffb_model::report::report_id::ReportId;
        use std::collections::HashSet;
        let mut game = make_game();
        game.stalling = true;
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.forgone = false;
        game.team_home.players.push(Player {
            id: "p1".into(), name: "P1".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("p1", ffb_model::types::FieldCoordinate::new(5, 5));
        let mut step = StepInitSelecting::new("end".into());
        let action = Action::ActivatePlayer {
            player_id: "p1".into(),
            player_action: PlayerActionChoice::Move,
            block_defender_id: None,
        };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::STALLER_DETECTED),
            "expected STALLER_DETECTED report when game.stalling is true");
    }

    #[test]
    fn pac_to_player_action_all_variants() {
        assert_eq!(pac_to_player_action(PlayerActionChoice::Move), PlayerAction::Move);
        assert_eq!(pac_to_player_action(PlayerActionChoice::Block), PlayerAction::Block);
        assert_eq!(pac_to_player_action(PlayerActionChoice::Foul), PlayerAction::Foul);
        assert_eq!(pac_to_player_action(PlayerActionChoice::HandOff), PlayerAction::HandOver);
    }
}
