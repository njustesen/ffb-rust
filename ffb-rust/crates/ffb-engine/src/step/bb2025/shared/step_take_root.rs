use ffb_model::enums::{PlayerAction, PlayerState, PS_STANDING, ReRollSource};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_mechanics::mechanics::{is_skill_roll_successful, minimum_roll_confusion};
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.shared.StepTakeRoot
/// + com.fumbbl.ffb.server.skillbehaviour.bb2025.TakeRootBehaviour.
///
/// The Java step body is entirely delegated to `executeStepHooks(this, state)`.
/// The BB2025 TakeRootBehaviour hook (inlined here) does:
///   1. If player started STANDING and is not yet rooted → roll d6 vs. minimumRollConfusion(true)=2.
///   2. On success → NEXT_STEP (player may move freely).
///   3. On failure → set player rooted, cancel player action.
///   4. TODO: re-roll dialog path (WAITING_FOR_RE_ROLL). For now random agent never re-rolls.
///
/// `cancelPlayerAction()` (Java StepTakeRoot): adjusts the acting player's action enum back to its
/// base action (BLITZ_MOVE→BLITZ etc.), sets going_for_it=true, dodging=false, then sets the
/// player state to rooted=true in the field model.
///
/// `old_player_state` — receives StepParameter::OldPlayerState published by the activation
/// generator (Java: actingPlayer.getOldPlayerState()). Used to decide whether the player started
/// STANDING. Defaults to None (= conservatively treat as STANDING for safety).
pub struct StepTakeRoot {
    /// Java: state.status — ActionStatus. WAITING_FOR_RE_ROLL defers driver; else NEXT_STEP.
    pub status: TakeRootStatus,
    /// Receives OldPlayerState from the activation generator (actingPlayer.getOldPlayerState()).
    pub old_player_state: Option<PlayerState>,
    // AbstractStepWithReRoll stubs
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
}

/// Mirrors Java ActionStatus for TakeRoot: SUCCESS or WAITING_FOR_RE_ROLL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TakeRootStatus {
    Success,
    /// Set when a re-roll offer is outstanding. Driver stays on Continue.
    WaitingForReRoll,
}

impl StepTakeRoot {
    pub fn new() -> Self {
        Self {
            status: TakeRootStatus::Success,
            old_player_state: None,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }
}

impl Default for StepTakeRoot {
    fn default() -> Self { Self::new() }
}

impl Step for StepTakeRoot {
    fn id(&self) -> StepId { StepId::TakeRoot }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::UseReRoll { use_reroll: false } => {
                self.re_roll_source = None;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::OldPlayerState(v) => { self.old_player_state = Some(*v); true }
            _ => false,
        }
    }
}

impl StepTakeRoot {
    /// Java: executeStep() → checks turn mode → executeStepHooks(this, state)
    /// → TakeRootBehaviour.handleExecuteStepHook
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: if (!game.getTurnMode().checkNegatraits()) → NEXT_STEP
        if !game.turn_mode.check_negatraits() {
            return StepOutcome::next();
        }

        // Java: TakeRootBehaviour.handleExecuteStepHook
        // -----------------------------------------------------------------
        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        // Java: actingPlayer.getOldPlayerState().getBase() == PlayerState.STANDING
        // If old_player_state is unknown (not yet set), conservatively assume STANDING.
        let started_standing = self.old_player_state
            .map(|s| s.base() == PS_STANDING)
            .unwrap_or(true);

        let is_rooted = game.field_model.player_state(&player_id)
            .map(|s| s.is_rooted())
            .unwrap_or(false);

        if !started_standing || is_rooted {
            // Java: hook returns false → executeStep falls through to NEXT_STEP
            return StepOutcome::next();
        }

        // Java: if (reRolledAction == step.getReRolledAction()) { if (source == null || !useReRoll) cancel }
        //       else { doRoll = hasUnusedSkill(actingPlayer, TakeRoot) }
        let mut do_roll;
        if self.re_rolled_action.as_deref() == Some("TAKE_ROOT") {
            if let Some(ref source_name) = self.re_roll_source.clone() {
                let source = ReRollSource::new(source_name.as_str());
                if use_reroll(game, &source, &player_id) {
                    do_roll = true;
                } else {
                    self.status = TakeRootStatus::Success;
                    return cancel_take_root_player_action(game, &player_id);
                }
            } else {
                // Player declined — cancel
                self.status = TakeRootStatus::Success;
                return cancel_take_root_player_action(game, &player_id);
            }
        } else {
            do_roll = game.player(&player_id)
                .map(|p| p.has_skill(ffb_model::enums::SkillId::TakeRoot))
                .unwrap_or(false);
        }

        if !do_roll {
            return StepOutcome::next();
        }

        // Java: int roll = rollSkill(); int minimumRoll = DiceInterpreter.minimumRollConfusion(true)
        let roll = rng.d6();
        let minimum_roll = minimum_roll_confusion(true);
        let successful = is_skill_roll_successful(roll, minimum_roll);

        if successful {
            self.status = TakeRootStatus::Success;
            StepOutcome::next()
        } else {
            self.status = TakeRootStatus::Success; // may be overridden to WaitingForReRoll below
            // Java: if (reRolledAction != current && askForReRollIfAvailable) → WAITING_FOR_RE_ROLL
            if self.re_rolled_action.is_none() {
                if let Some(prompt) = ask_for_reroll_if_available(game, "TAKE_ROOT", minimum_roll, false) {
                    self.re_rolled_action = Some("TAKE_ROOT".into());
                    self.re_roll_source = Some("TRR".into());
                    self.status = TakeRootStatus::WaitingForReRoll;
                    return StepOutcome::cont().with_prompt(prompt);
                }
            }
            cancel_take_root_player_action(game, &player_id)
        }
    }
}

/// Java: StepTakeRoot.cancelPlayerAction()
/// Adjusts the acting player's action back to its base form (BLITZ_MOVE→BLITZ, etc.),
/// sets going_for_it=true, dodging=false, then marks player rooted in the field model.
/// Returns the StepOutcome (NEXT_STEP, no extra published parameters).
fn cancel_take_root_player_action(game: &mut Game, player_id: &str) -> StepOutcome {
    // Java: actingPlayer.setGoingForIt(true); actingPlayer.setDodging(false);
    game.acting_player.goes_for_it = true;
    // (Dodging flag not currently tracked on ActingPlayer in Rust; no-op.)

    // Java: switch on playerAction → change to base action
    match game.acting_player.player_action {
        Some(PlayerAction::BlitzMove) => {
            game.acting_player.player_action = Some(PlayerAction::Blitz);
        }
        Some(PlayerAction::KickEmBlitz) => {
            game.acting_player.player_action = Some(PlayerAction::KickEmBlock);
        }
        Some(PlayerAction::PassMove) => {
            game.acting_player.player_action = Some(PlayerAction::Pass);
            let pid = player_id.to_string();
            game.thrower_id = Some(pid.clone());
            game.thrower_action = Some(PlayerAction::Pass);
        }
        Some(PlayerAction::ThrowTeamMateMove) => {
            game.acting_player.player_action = Some(PlayerAction::ThrowTeamMate);
        }
        Some(PlayerAction::KickTeamMateMove) => {
            game.acting_player.player_action = Some(PlayerAction::KickTeamMate);
        }
        Some(PlayerAction::HandOverMove) => {
            game.acting_player.player_action = Some(PlayerAction::HandOver);
            let pid = player_id.to_string();
            game.thrower_id = Some(pid.clone());
            game.thrower_action = Some(PlayerAction::HandOver);
        }
        Some(PlayerAction::FoulMove) => {
            game.acting_player.player_action = Some(PlayerAction::Foul);
        }
        Some(PlayerAction::SecureTheBall) => {
            // Java: publishParameter(END_PLAYER_ACTION, true)  // fall through to PUNT_MOVE/MOVE
            // then → UtilServerPlayerMove.updateMoveSquares(false)
            // (move-squares update is a server-state op; stubbed)
        }
        Some(PlayerAction::PuntMove) | Some(PlayerAction::Move) => {
            // Java: UtilServerPlayerMove.updateMoveSquares(getGameState(), false)
            // Stubbed — move squares are not yet tracked in Rust.
        }
        _ => {}
    }

    // Java: game.getFieldModel().setPlayerState(actingPlayer.getPlayer(), playerState.changeRooted(true))
    if let Some(state) = game.field_model.player_state(player_id) {
        game.field_model.set_player_state(player_id, state.change_rooted(true));
    }

    // Java: getResult().setSound(SoundId.ROOT) — sound not tracked in Rust engine; no-op.

    // Java: state.status = FAILURE → executeStep falls through to NEXT_STEP
    // (WAITING_FOR_RE_ROLL path is stubbed; always NEXT_STEP here)
    let out = StepOutcome::next();

    // Java: if (playerAction == SECURE_THE_BALL) publishParameter(END_PLAYER_ACTION, true)
    if matches!(game.acting_player.player_action, Some(PlayerAction::SecureTheBall)) {
        out.publish(StepParameter::EndPlayerAction(true))
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{Rules, SkillId, TurnMode, PS_STANDING, PS_PRONE,
                           PlayerType, PlayerGender};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    fn bare_player(id: &str) -> Player {
        Player {
            id: id.into(), name: id.into(), nr: 0, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 3, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
        }
    }

    fn add_player_with_take_root(game: &mut Game, pid: &str) {
        let mut p = bare_player(pid);
        p.starting_skills.push(SkillWithValue::new(SkillId::TakeRoot));
        game.team_home.players.push(p);
        let state = PlayerState::new(PS_STANDING);
        game.field_model.set_player_state(pid, state);
    }

    /// No acting player → NEXT_STEP, no crash.
    #[test]
    fn no_acting_player_returns_next() {
        let mut game = make_game();
        let mut step = StepTakeRoot::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    /// Turn mode = KICKOFF_RETURN does not check negatraits → NEXT_STEP.
    #[test]
    fn kickoff_return_skips_roll() {
        let mut game = make_game();
        game.turn_mode = TurnMode::KickoffReturn;
        let mut step = StepTakeRoot::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    /// set_parameter accepts OldPlayerState.
    #[test]
    fn set_parameter_old_player_state_accepted() {
        let mut step = StepTakeRoot::new();
        let state = PlayerState::new(PS_STANDING);
        assert!(step.set_parameter(&StepParameter::OldPlayerState(state)));
        assert!(step.old_player_state.is_some());
    }

    /// Unrecognised parameter is rejected.
    #[test]
    fn set_parameter_returns_false_for_unknown() {
        let mut step = StepTakeRoot::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(false)));
    }

    /// Player has TakeRoot, started STANDING, not rooted — should roll and return NEXT_STEP.
    #[test]
    fn player_with_take_root_rolls_and_returns_next() {
        let mut game = make_game();
        add_player_with_take_root(&mut game, "p1");
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Move);

        let mut step = StepTakeRoot::new();
        step.old_player_state = Some(PlayerState::new(PS_STANDING));

        let out = step.start(&mut game, &mut GameRng::new(0));
        // Always NEXT_STEP regardless of roll result (re-roll dialog stubbed)
        assert_eq!(out.action, StepAction::NextStep);
    }

    /// Player already rooted → hook exits early, NEXT_STEP, no state change.
    #[test]
    fn already_rooted_skips_roll() {
        let mut game = make_game();
        add_player_with_take_root(&mut game, "p1");
        // Root the player
        let state = game.field_model.player_state("p1").unwrap().change_rooted(true);
        game.field_model.set_player_state("p1", state);
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Move);

        let mut step = StepTakeRoot::new();
        step.old_player_state = Some(PlayerState::new(PS_STANDING));

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    /// Player did NOT start standing → no roll (same as already-rooted guard).
    #[test]
    fn did_not_start_standing_skips_roll() {
        let mut game = make_game();
        add_player_with_take_root(&mut game, "p1");
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Move);

        let mut step = StepTakeRoot::new();
        // Player started PRONE
        step.old_player_state = Some(PlayerState::new(PS_PRONE));

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    /// cancel_take_root_player_action roots the player in the field model.
    #[test]
    fn cancel_action_roots_player() {
        let mut game = make_game();
        add_player_with_take_root(&mut game, "p1");
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Move);

        cancel_take_root_player_action(&mut game, "p1");

        let state = game.field_model.player_state("p1").unwrap();
        assert!(state.is_rooted());
    }

    /// cancel_take_root_player_action: BLITZ_MOVE → BLITZ.
    #[test]
    fn cancel_action_blitz_move_to_blitz() {
        let mut game = make_game();
        add_player_with_take_root(&mut game, "p1");
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::BlitzMove);

        cancel_take_root_player_action(&mut game, "p1");

        assert_eq!(game.acting_player.player_action, Some(PlayerAction::Blitz));
    }

    /// cancel_take_root_player_action: PASS_MOVE → PASS + sets thrower_id.
    #[test]
    fn cancel_action_pass_move_to_pass() {
        let mut game = make_game();
        add_player_with_take_root(&mut game, "p1");
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::PassMove);

        cancel_take_root_player_action(&mut game, "p1");

        assert_eq!(game.acting_player.player_action, Some(PlayerAction::Pass));
        assert_eq!(game.thrower_id.as_deref(), Some("p1"));
        assert_eq!(game.thrower_action, Some(PlayerAction::Pass));
    }

    /// Multiple seeds — always returns NEXT_STEP (re-roll dialog stubbed).
    #[test]
    fn always_returns_next_step_action() {
        for seed in 0..20u64 {
            let mut game = make_game();
            add_player_with_take_root(&mut game, "p1");
            game.acting_player.player_id = Some("p1".into());
            game.acting_player.player_action = Some(PlayerAction::Move);

            let mut step = StepTakeRoot::new();
            step.old_player_state = Some(PlayerState::new(PS_STANDING));

            let out = step.start(&mut game, &mut GameRng::new(seed));
            assert_eq!(out.action, StepAction::NextStep, "seed={seed}");
        }
    }
}
