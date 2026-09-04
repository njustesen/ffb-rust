use ffb_model::enums::{PlayerAction, PlayerState, PS_STANDING, ReRollSource};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_mechanics::mechanics::{is_skill_roll_successful, minimum_roll_confusion};
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

// ── Hook state ────────────────────────────────────────────────────────────────

/// Java: StepTakeRoot.StepState (extended with AbstractStepWithReRoll fields).
/// Used by TakeRootBehaviour.handleExecuteStepHook via dispatch::execute_step_hooks.
#[derive(Debug)]
pub struct StepTakeRootHookState {
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
    pub old_player_state: Option<PlayerState>,
    pub outcome: Option<StepOutcome>,
    pub updated_re_rolled_action: Option<String>,
    pub updated_re_roll_source: Option<String>,
}

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

        // BB2016 ONLY. `bb2016.StepTakeRoot.executeStep` recovers the acting player's tacklezones
        // before running the hooks:
        //   `PlayerState playerState = ...getPlayerState(actingPlayer.getPlayer()).recoverTacklezones();
        //    game.getFieldModel().setPlayerState(actingPlayer.getPlayer(), playerState);`
        // Neither `bb2020.shared.StepTakeRoot` nor `bb2025.shared.StepTakeRoot` has that line.
        if game.rules == ffb_model::enums::Rules::Bb2016 {
            if let Some(state) = game.field_model.player_state(&player_id) {
                game.field_model.set_player_state(&player_id, state.recover_tacklezones());
            }
        }

        // Java: actingPlayer.getOldPlayerState().getBase() == PlayerState.STANDING. Read it from the
        // ActingPlayer (captured at activation in change_player_action), falling back to the
        // step-parameter copy, then the conservative STANDING default. A prone Treeman blitzing has
        // old_player_state=PRONE → started_standing=false → NO Take Root roll (matching Java; wood_elf
        // seed 1 i=49: Rust was defaulting to STANDING and rolling Take Root twice, shifting block dice).
        // `startedStanding` is a **BB2025-ONLY** condition. Java's three TakeRootBehaviours differ:
        //   bb2025: `if (startedStanding && !playerState.isRooted())`
        //   bb2020: `if (!playerState.isRooted())`
        //   bb2016: `if (!playerState.isRooted())`
        // So in bb2016/bb2020 a PRONE player standing up STILL rolls Take Root. Applying the bb2025
        // condition to every edition silently dropped that d6 for a prone bb2016 Treeman (wood_elf
        // bb2016 seed 1 step 46: Java rolls Take Root then the stand-up d6; Rust rolled only the
        // stand-up, shifting every later die in the game).
        let started_standing = if game.rules == ffb_model::enums::Rules::Bb2025 {
            game.acting_player.old_player_state
                .or(self.old_player_state)
                .map(|s| s.base() == PS_STANDING)
                .unwrap_or(true)
        } else {
            true
        };

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
                if use_reroll(game, &source, &player_id, rng) {
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
            // Java (ALL editions): `doRoll = UtilCards.hasUnusedSkill(actingPlayer, skill)` — the
            // ACTING PLAYER's per-activation used-skill set. This matters now that bb2016/bb2020 no
            // longer short-circuit on `startedStanding`: TAKE_ROOT appears in BOTH the Select and the
            // Move sequence, so without the unused-skill check one activation would roll it TWICE.
            do_roll = game.player(&player_id)
                .map(|p| p.has_skill(ffb_model::enums::SkillId::TakeRoot))
                .unwrap_or(false)
                && !game.acting_player.used_skills.contains(&ffb_model::enums::SkillId::TakeRoot);
        }

        if !do_roll {
            return StepOutcome::next();
        }

        // Java: int roll = rollSkill(); int minimumRoll = DiceInterpreter.minimumRollConfusion(true)
        let roll = rng.d6();
        // Java: actingPlayer.markSkillUsed(skill) — per-activation, pairing with hasUnusedSkill above.
        game.acting_player.used_skills.insert(ffb_model::enums::SkillId::TakeRoot);
        let minimum_roll = minimum_roll_confusion(true);
        let successful = is_skill_roll_successful(roll, minimum_roll);

        // Java: boolean reRolled = (reRolledAction != null) && (reRolledAction == step.getReRolledAction())
        //         && (step.getReRollSource() != null);
        //       getResult().addReport(new ReportConfusionRoll(playerId, successful, roll, minimumRoll, reRolled, skill));
        let report_rerolled = self.re_rolled_action.as_deref() == Some("TAKE_ROOT")
            && self.re_roll_source.is_some();
        game.report_list.add(ffb_model::report::report_confusion_roll::ReportConfusionRoll::new(
            Some(player_id.clone()),
            successful,
            roll,
            minimum_roll,
            report_rerolled,
            Some(ffb_model::enums::SkillId::TakeRoot.class_name().to_string()),
        ));

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
    //
    // …except in BB2016, where the very same line reads `actingPlayer.setGoingForIt(FALSE)`
    // (`bb2016.StepTakeRoot.cancelPlayerAction`, first statement). bb2020 and bb2025 both pass
    // `true`. This file is the LIVE StepTakeRoot for all three editions (driver.rs has no bb2016
    // override for StepId::TakeRoot — `bb2016/step_take_root.rs` is a dead twin), so the edition
    // differences have to be gated here.
    let bb2016 = game.rules == ffb_model::enums::Rules::Bb2016;
    game.acting_player.goes_for_it = !bb2016;
    // Clearing `dodging` is essential: StepInitMoving runs BEFORE StepTakeRoot in the move sequence
    // and may already have set dodging=true for the queued destination (a tackle-zone square). Without
    // this, the rooted player — whose StepMove is skipped via isPinned — would still reach
    // StepMoveDodge, whose only guard is `if (!isDodging()) NEXT_STEP`, and roll a spurious dodge die
    // (wood_elf seed 1 i=193: rooted away_01 Treeman rolled a MoveDodge Java never rolled, shifting the
    // whole RNG stream). Java's cancelPlayerAction clears it here.
    game.acting_player.dodging = false;

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
            // BB2020/BB2025 only: `game.setThrowerId(...); game.setThrowerAction(PASS);`
            // `bb2016.StepTakeRoot.cancelPlayerAction`'s PASS_MOVE arm is the
            // `changeActingPlayer` call and NOTHING else — it leaves the thrower alone.
            if !bb2016 {
                let pid = player_id.to_string();
                game.thrower_id = Some(pid.clone());
                game.thrower_action = Some(PlayerAction::Pass);
            }
        }
        Some(PlayerAction::ThrowTeamMateMove) => {
            game.acting_player.player_action = Some(PlayerAction::ThrowTeamMate);
        }
        Some(PlayerAction::KickTeamMateMove) => {
            game.acting_player.player_action = Some(PlayerAction::KickTeamMate);
        }
        Some(PlayerAction::HandOverMove) => {
            game.acting_player.player_action = Some(PlayerAction::HandOver);
            // BB2020/BB2025 only, exactly as the PASS_MOVE arm above. Setting the thrower under
            // BB2016 is what left `StepInitPassing` with `thrower_id = the Treeman` and
            // `pass_coordinate = None`: it then refused instead of parking, and the whole team turn
            // ended a move early (halfling bb2016 seed 1 idx 55, seed 6 @1e6 idx 52).
            if !bb2016 {
                let pid = player_id.to_string();
                game.thrower_id = Some(pid.clone());
                game.thrower_action = Some(PlayerAction::HandOver);
            }
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

    /// A game whose acting player is a standing Take Root player, under the given ruleset.
    fn make_game_with_rules(rules: Rules) -> Game {
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), rules);
        game.home_playing = true;
        game.turn_mode = ffb_model::enums::TurnMode::Regular;
        add_player_with_take_root(&mut game, "p1");
        game.field_model.set_player_coordinate("p1", ffb_model::types::FieldCoordinate::new(5, 5));
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(ffb_model::enums::PlayerAction::Move);
        game.acting_player.old_player_state = Some(PlayerState::new(PS_STANDING));
        game
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
            is_big_guy: false,
            ..Default::default()
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
    /// Java's three `TakeRootBehaviour`s gate the roll differently, and this step is shared:
    ///   bb2025: `if (startedStanding && !playerState.isRooted())`
    ///   bb2020: `if (!playerState.isRooted())`
    ///   bb2016: `if (!playerState.isRooted())`
    /// So a PRONE player standing up STILL rolls Take Root under bb2016/bb2020 and must NOT under
    /// bb2025. Applying the bb2025 condition everywhere dropped the d6 for every prone bb2016 Treeman
    /// activation (wood_elf bb2016 seed 1 step 46: Java rolls Take Root then the stand-up d6; Rust
    /// rolled only the stand-up).
    #[test]
    fn prone_take_root_rolls_in_bb2016_but_not_bb2025() {
        use ffb_model::enums::{Rules, SkillId, PS_PRONE};

        fn run(rules: Rules) -> u64 {
            let mut game = make_game_with_rules(rules);
            game.field_model.set_player_state("p1", PlayerState::new(PS_PRONE));
            // Activation captured a PRONE old state — i.e. the player is standing up this activation.
            game.acting_player.old_player_state = Some(PlayerState::new(PS_PRONE));
            game.acting_player.used_skills.clear();
            let mut rng = GameRng::new(0);
            let before = rng.call_count;
            let _ = StepTakeRoot::new().start(&mut game, &mut rng);
            let _ = SkillId::TakeRoot;
            rng.call_count - before
        }

        assert_eq!(run(Rules::Bb2016), 1, "bb2016 rolls Take Root for a player standing up");
        assert_eq!(run(Rules::Bb2020), 1, "bb2020 also has no startedStanding condition");
        assert_eq!(run(Rules::Bb2025), 0, "bb2025 skips Take Root unless the player started standing");
    }

    /// Java (all editions) gates on `UtilCards.hasUnusedSkill(actingPlayer, skill)` and calls
    /// `actingPlayer.markSkillUsed(skill)`. TAKE_ROOT is in BOTH the Select and Move sequences, so
    /// without that pairing one activation would roll it twice — which is what previously kept
    /// bb2016 accidentally correct via the bb2025 `startedStanding` short-circuit.
    #[test]
    fn take_root_rolls_once_per_activation() {
        use ffb_model::enums::{Rules, SkillId};

        let mut game = make_game_with_rules(Rules::Bb2016);
        game.acting_player.used_skills.clear();
        let mut rng = GameRng::new(0);

        let before = rng.call_count;
        let _ = StepTakeRoot::new().start(&mut game, &mut rng);
        let first = rng.call_count - before;
        assert_eq!(first, 1, "the Select-sequence Take Root rolls");
        assert!(game.acting_player.used_skills.contains(&SkillId::TakeRoot),
            "the roll marks the skill used on the ACTING PLAYER (per activation)");

        let before = rng.call_count;
        let _ = StepTakeRoot::new().start(&mut game, &mut rng);
        assert_eq!(rng.call_count - before, 0,
            "the Move-sequence Take Root in the SAME activation must not roll again");
    }

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

    /// `bb2016.StepTakeRoot.cancelPlayerAction` differs from the bb2020/bb2025 twin in two places,
    /// and this file is the LIVE step for all three editions:
    ///
    /// ```java
    /// // bb2016                                   // bb2020 / bb2025
    /// actingPlayer.setGoingForIt(false);          actingPlayer.setGoingForIt(true);
    /// case PASS_MOVE:                             case PASS_MOVE:
    ///   changeActingPlayer(..., PASS, ...);         changeActingPlayer(..., PASS, ...);
    ///   break;                                      game.setThrowerId(actingPlayer.getPlayerId());
    ///                                               game.setThrowerAction(PlayerAction.PASS);
    /// case HAND_OVER_MOVE:                          break;
    ///   changeActingPlayer(..., HAND_OVER, ...);  case HAND_OVER_MOVE:
    ///   break;                                      changeActingPlayer(..., HAND_OVER, ...);
    ///                                               game.setThrowerId(...); setThrowerAction(HAND_OVER);
    /// ```
    #[test]
    fn bb2016_cancel_leaves_the_thrower_alone_and_clears_going_for_it() {
        for (rules, expect_thrower, expect_gfi) in [
            (Rules::Bb2016, false, false),
            (Rules::Bb2020, true, true),
            (Rules::Bb2025, true, true),
        ] {
            for (declared, base) in [
                (PlayerAction::HandOverMove, PlayerAction::HandOver),
                (PlayerAction::PassMove, PlayerAction::Pass),
            ] {
                let mut game = make_game_with_rules(rules);
                add_player_with_take_root(&mut game, "p1");
                game.acting_player.player_id = Some("p1".into());
                game.acting_player.player_action = Some(declared);
                game.thrower_id = None;
                game.thrower_action = None;
                game.acting_player.goes_for_it = !expect_gfi;

                cancel_take_root_player_action(&mut game, "p1");

                assert_eq!(game.acting_player.player_action, Some(base),
                    "{rules:?} {declared:?}: the *_MOVE action reverts to its base form");
                assert_eq!(game.acting_player.goes_for_it, expect_gfi,
                    "{rules:?}: setGoingForIt({expect_gfi})");
                assert_eq!(game.thrower_id.is_some(), expect_thrower,
                    "{rules:?} {declared:?}: only bb2020/bb2025 set the thrower here");
                assert_eq!(game.thrower_action.is_some(), expect_thrower,
                    "{rules:?} {declared:?}: throwerAction follows throwerId");
            }
        }
    }

    /// `bb2016.StepTakeRoot.executeStep` recovers the acting player's tacklezones before running
    /// the behaviour hooks; the bb2020 and bb2025 steps do not.
    #[test]
    fn bb2016_recovers_tacklezones_before_the_roll() {
        for (rules, expect_confused) in [(Rules::Bb2016, false), (Rules::Bb2020, true)] {
            let mut game = make_game_with_rules(rules);
            add_player_with_take_root(&mut game, "p1");
            game.acting_player.player_id = Some("p1".into());
            game.acting_player.player_action = Some(PlayerAction::Move);
            let st = game.field_model.player_state("p1").unwrap();
            game.field_model.set_player_state("p1", st.change_confused(true));

            let mut step = StepTakeRoot::new();
            let _ = step.start(&mut game, &mut GameRng::new(1));

            assert_eq!(game.field_model.player_state("p1").unwrap().is_confused(), expect_confused,
                "{rules:?}: only bb2016 calls recoverTacklezones() here");
        }
    }

    /// Java: TakeRootBehaviour.handleExecuteStepHook always adds a ReportConfusionRoll when
    /// a roll is made (doRoll branch), regardless of success/failure.
    #[test]
    fn rolling_adds_confusion_roll_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        add_player_with_take_root(&mut game, "p1");
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Move);

        let mut step = StepTakeRoot::new();
        step.old_player_state = Some(PlayerState::new(PS_STANDING));

        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::CONFUSION_ROLL),
            "expected CONFUSION_ROLL report after TakeRoot skill roll");
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
