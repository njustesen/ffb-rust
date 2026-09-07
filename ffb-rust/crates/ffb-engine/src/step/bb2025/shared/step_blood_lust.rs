use ffb_mechanics::mechanics::minimum_roll_blood_lust_with;
use ffb_model::enums::{PlayerAction, ReRollSource, SkillId};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::prompts::agent_prompt::AgentPrompt;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

// ── Hook state ────────────────────────────────────────────────────────────────

/// Java: StepBloodLust.StepState (extended with AbstractStepWithReRoll fields).
/// Used by BloodLustBehaviour.handleExecuteStepHook via dispatch::execute_step_hooks.
#[derive(Debug)]
pub struct StepBloodLustHookState {
    pub goto_label_on_failure: Option<String>,
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
    pub bloodlust_action: Option<PlayerAction>,
    pub wait_for_action_change: bool,
    pub outcome: Option<StepOutcome>,
    pub updated_re_rolled_action: Option<String>,
    pub updated_re_roll_source: Option<String>,
}

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.shared.StepBloodLust.
///
/// Handles the Vampire blood-lust mechanic (BB2025).
///
/// Identical to the BB2020 version: on failure for non-MOVE actions the server shows
/// a `DialogBloodlustActionParameter` dialog offering the player an action change.
///
/// Java state fields: goToLabelOnFailure, bloodlustAction, status (ActionStatus).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BloodLustStatus {
    None,
    WaitingForReRoll,
    WaitForActionChange,
    Success,
    Failure,
}

pub struct StepBloodLust {
    /// Java: state.goToLabelOnFailure (init param GOTO_LABEL_ON_FAILURE)
    pub goto_label_on_failure: Option<String>,
    /// Java: state.bloodlustAction — alternate action chosen by the client
    pub bloodlust_action: Option<PlayerAction>,
    /// Java: state.status (ActionStatus)
    pub status: BloodLustStatus,
    /// Java: AbstractStepWithReRoll.reRolledAction
    pub re_rolled_action: Option<String>,
    /// Java: AbstractStepWithReRoll.reRollSource
    pub re_roll_source: Option<String>,
}

impl StepBloodLust {
    pub fn new(goto_label_on_failure: impl Into<String>) -> Self {
        let label = goto_label_on_failure.into();
        Self {
            goto_label_on_failure: if label.is_empty() { None } else { Some(label) },
            bloodlust_action: None,
            status: BloodLustStatus::None,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }
}

impl Default for StepBloodLust {
    fn default() -> Self { Self::new("") }
}

impl Step for StepBloodLust {
    fn id(&self) -> StepId { StepId::BloodLust }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if let Action::UseReRoll { use_reroll: false } = action {
            self.re_roll_source = None;
        }
        // Java: CLIENT_BLOODLUST_ACTION → if (change) state.bloodlustAction = getAlternateAction(currentAction)
        if let Action::BloodlustAction { change } = action {
            if *change {
                if let Some(current) = game.acting_player.player_action {
                    self.bloodlust_action = Some(Self::get_alternate_action(current));
                }
            }
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => {
                self.goto_label_on_failure = Some(v.clone());
                true
            }
            StepParameter::BloodLustAction(v) => {
                self.bloodlust_action = *v;
                true
            }
            _ => false,
        }
    }
}

impl StepBloodLust {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: state.status == WAIT_FOR_ACTION_CHANGE branch (after the action-change dialog response).
        // The player is already suffering blood lust; route the sequence. Crucially, unless a failure
        // label applies, Java returns NEXT_STEP so the declared action (e.g. a Block) STILL EXECUTES
        // while suffering — the vampire feeds afterwards. The previous Rust code always jumped to the
        // failure label, silently dropping the block (vampire seed 1 i=23: a failed-bloodlust BLOCK was
        // skipped, so no block dice / armour / knockdowns and the RNG desynced from Java).
        if self.status == BloodLustStatus::WaitForActionChange {
            let player_action = game.acting_player.player_action;
            let is_standing_up = player_action.map(|a| a.is_standing_up()).unwrap_or(false);
            let is_passing = player_action.map(|a| a.is_passing()).unwrap_or(false);
            let has_label = self.goto_label_on_failure.as_deref().map(|l| !l.is_empty()).unwrap_or(false);
            // Java: goToLabel iff (label provided) && (bloodlustAction != null || playerAction.isPassing())
            let mut out = if has_label && (self.bloodlust_action.is_some() || is_passing) {
                StepOutcome::goto(self.goto_label_on_failure.as_deref().unwrap())
            } else {
                StepOutcome::next()
            };
            // Java: if playerAction.isStandingUp() → DISPATCH_PLAYER_ACTION(playerAction); else MOVE_STACK null
            if is_standing_up {
                out = out.publish(StepParameter::DispatchPlayerAction(player_action));
            } else {
                out = out.publish(StepParameter::MoveStack(vec![]));
                // Java's `MOVE_STACK, null` here throws away the path the client delivered at
                // `StepInitSelecting` phase 2. Rust sometimes asks for that path LATER, at
                // `StepInitMoving` — and then this publish discards nothing, so the discard is
                // owed to the first path that arrives. `has_moved` is the discriminator, because
                // it is `StepInitMoving` popping a delivered square that sets it (Java
                // `StepInitMoving.executeStep`: `actingPlayer.setHasMoved(true)`); when it is
                // already true the publish above has a real stack to throw away and nothing is
                // owed. See `ActingPlayer::blood_lust_discards_move_stack`.
                if !game.acting_player.has_moved {
                    game.acting_player.blood_lust_discards_move_stack = true;
                }
            }
            out = out.publish(StepParameter::BloodLustAction(self.bloodlust_action));
            return out;
        }

        if !game.turn_mode.check_negatraits() {
            return StepOutcome::next();
        }

        let acting_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        let re_rolled = self.re_rolled_action.as_deref() == Some("BLOOD_LUST");
        let do_roll;

        if re_rolled {
            if let Some(ref source_str) = self.re_roll_source.clone() {
                let source = ReRollSource::new(source_str.as_str());
                if use_reroll(game, &source, &acting_id, rng) {
                    do_roll = true;
                } else {
                    game.acting_player.suffering_blood_lust = true;
                    return self.fail_blood_lust_for_action(game, &acting_id);
                }
            } else {
                game.acting_player.suffering_blood_lust = true;
                return self.fail_blood_lust_for_action(game, &acting_id);
            }
        } else {
            do_roll = game.player(&acting_id)
                // Java (BOTH editions' BloodLustBehaviour):
                //     doRoll = UtilCards.hasUnusedSkill(actingPlayer, skill);
                // The used-set is the ACTING PLAYER's, which `changeActingPlayer` clears at every
                // activation, so a Vampire rolls Blood Lust ONCE PER ACTIVATION. Reading
                // `Player.used_skills` (whole GAME) let a Vampire roll Blood Lust exactly once ever.
                // Same per-activation shape as the Bone-head / Really Stupid fix.
                .map(|p| p.has_skill(SkillId::BloodLust))
                .map(|has| has && !game.acting_player.used_skills.contains(&SkillId::BloodLust))
                .unwrap_or(false);
        }

        if !do_roll {
            return StepOutcome::next();
        }

        // Java: goodConditions = BLITZ_MOVE | isKickingDowned | BLITZ | isBlockAction | MULTIPLE_BLOCK | STAND_UP_BLITZ
        let good_conditions = game.acting_player.player_action.map(|a|
            a == PlayerAction::BlitzMove
                || a.is_kicking_downed()
                || a == PlayerAction::Blitz
                || a.is_block_action()
                || a == PlayerAction::MultipleBlock
                || a == PlayerAction::StandUpBlitz
        ).unwrap_or(false);
        // Java: minimumRoll = max(2, getSkillIntValue(Bloodlust) - (goodConditions ? 1 : 0)).
        // Bloodlust's default skill value is 2 (Java Bloodlust ctor); the vampire roster overrides to 3.
        let skill_value = game.player(&acting_id)
            .map(|p| p.get_skill_value_int(SkillId::BloodLust, 2))
            .unwrap_or(2);
        let roll = rng.d6();
        let min_roll = minimum_roll_blood_lust_with(skill_value, good_conditions);
        // Java `BloodLustBehaviour` uses DiceInterpreter.isSkillRollSuccessful: a natural 6 always succeeds and a
        // natural 1 always fails, whatever the target. Only differs from a bare `>=` when the target
        // leaves 2..6, which is exactly when it matters.
        let successful = crate::dice_interpreter::DiceInterpreter::is_skill_roll_successful(roll, min_roll);

        // Java: `actingPlayer.markSkillUsed(skill)` — per-ACTIVATION, not on the Player.
        game.acting_player.used_skills.insert(SkillId::BloodLust);

        let event = GameEvent::BloodLustRoll { player_id: acting_id.clone(), roll, success: successful };

        if !successful {
            if !re_rolled {
                if let Some(prompt) = ask_for_reroll_if_available(game, "BLOOD_LUST", min_roll, false) {
                    self.re_rolled_action = Some("BLOOD_LUST".into());
                    self.re_roll_source = Some("TRR".into());
                    self.status = BloodLustStatus::WaitingForReRoll;
                    return StepOutcome::cont().with_event(event).with_prompt(prompt);
                }
            }
            // Java: setSufferingBloodLust(true) at failure when no re-roll is taken (set before the
            // action-change dialog for BLOCK/etc, so the flag holds through WAIT_FOR_ACTION_CHANGE).
            game.acting_player.suffering_blood_lust = true;
            return self.fail_blood_lust_for_action(game, &acting_id).with_event(event);
        }

        self.status = BloodLustStatus::Success;
        StepOutcome::next().with_event(event)
    }

    fn fail_blood_lust_for_action(&mut self, game: &mut Game, acting_id: &str) -> StepOutcome {
        // EDITION-SPECIFIC. `failBloodLustForAction` — the dialog that offers to convert the declared
        // action into its Blood Lust alternate — is a BB2020/BB2025 addition.
        // `bb2016/BloodLustBehaviour` has NO such branch: a failed roll goes straight to the failure
        // path, so the declared Block/Blitz is simply CANCELLED —
        //     actingPlayer.setSufferingBloodLust(true);
        //     step.publishParameter(new StepParameter(StepParameterKey.MOVE_STACK, null));
        //     step.getResult().setNextAction(GOTO_LABEL, state.goToLabelOnFailure);  // END_BLOCKING
        // Running the bb2025 dialog for bb2016 let a Vampire that failed Blood Lust still throw its
        // block: Java drew ONE die and passed the turn over, Rust drew thirteen
        // (vampire bb2016 seed 1 i=100 — Java 77 rng calls vs Rust 89).
        let current_action = game.acting_player.player_action;
        let needs_dialog = current_action
            .map(|a| shows_bloodlust_action_dialog(game.rules, a))
            .unwrap_or(false);

        if needs_dialog {
            self.status = BloodLustStatus::WaitForActionChange;
            let player_id = acting_id.to_string();
            return StepOutcome::cont()
                .with_prompt(AgentPrompt::BloodlustAction { player_id });
        }

        self.fail_blood_lust(game)
    }

    fn fail_blood_lust(&mut self, game: &mut Game) -> StepOutcome {
        self.status = BloodLustStatus::Failure;
        game.acting_player.suffering_blood_lust = true;

        let label = self.goto_label_on_failure.clone();
        let bloodlust_param = self.bloodlust_action;

        let base = match label {
            Some(ref l) if !l.is_empty() => StepOutcome::goto(l),
            _ => StepOutcome::next(),
        };
        // Java: `publishParameter(new StepParameter(MOVE_STACK, null))` — the discard of the path
        // the client delivered at `StepInitSelecting` phase 2. When Rust has not been handed a
        // path yet (`!has_moved`, i.e. no square has been popped) that publish discards nothing,
        // so the discard is owed to the first path that arrives — see
        // `ActingPlayer::blood_lust_discards_move_stack`. bb2020 seed 7 i=29/i=30 is the pair that
        // separates the two: the same failure with `has_moved` true already had a real stack.
        if !game.acting_player.has_moved {
            game.acting_player.blood_lust_discards_move_stack = true;
        }
        let out = base.publish(StepParameter::MoveStack(vec![]));
        match bloodlust_param {
            Some(action) => out.publish(StepParameter::BloodLustAction(Some(action))),
            None => out,
        }
    }

    /// Java: private PlayerAction getAlternateAction(PlayerAction currentAction)
    fn get_alternate_action(current: PlayerAction) -> PlayerAction {
        match current {
            PlayerAction::Pass => PlayerAction::PassMove,
            PlayerAction::HandOver => PlayerAction::HandOverMove,
            PlayerAction::Foul => PlayerAction::FoulMove,
            PlayerAction::StandUpBlitz => PlayerAction::BlitzSelect,
            PlayerAction::ThrowTeamMate => PlayerAction::ThrowTeamMateMove,
            PlayerAction::KickTeamMate => PlayerAction::KickTeamMateMove,
            _ => PlayerAction::Move,
        }
    }
}

/// Does a FAILED Blood Lust roll open the action-change dialog for this declared action?
///
/// Java is an explicit `Arrays.asList(...).contains(actingPlayer.getPlayerAction())` test, and the
/// membership list differs per edition — this is the whole edition split:
///
/// * `bb2025/BloodLustBehaviour`: `{VICIOUS_VINES, BLOCK, PASS, HAND_OVER, THROW_BOMB,
///   THROW_TEAM_MATE, KICK_TEAM_MATE, FOUL, STAND_UP, STAND_UP_BLITZ, MULTIPLE_BLOCK,
///   SECURE_THE_BALL, PUNT}` — note BLITZ_MOVE and GAZE_MOVE are **absent**.
/// * `bb2020/BloodLustBehaviour`: the same list with `SECURE_THE_BALL`/`PUNT` replaced by
///   `BLITZ_MOVE` and `GAZE_MOVE`.
/// * `bb2016/BloodLustBehaviour`: there is no such branch at all — every failure goes straight to
///   `MOVE_STACK null` + the failure label.
///
/// Rust used to approximate the list as "any action whose `getAlternateAction` differs", i.e.
/// everything except MOVE. That is a superset in bb2025: a vampire who failed Blood Lust on a
/// BLITZ_MOVE got an extra `BloodlustAction` prompt Java never shows, and the resulting
/// `MOVE_STACK`/`BLOOD_LUST_ACTION` publish ended the drive where Java carried the blitzer on
/// (bb2025 vampire seed 3 step 54: same pre-hash, same declaration, different post-hash).
pub fn shows_bloodlust_action_dialog(rules: ffb_model::enums::Rules, action: PlayerAction) -> bool {
    use PlayerAction::*;
    match rules {
        // bb2016 has no action-change dialog on a failed Blood Lust.
        ffb_model::enums::Rules::Bb2016 => false,
        ffb_model::enums::Rules::Bb2020 => matches!(
            action,
            ViciousVines | Block | Pass | HandOver | ThrowBomb | ThrowTeamMate | KickTeamMate
                | Foul | StandUp | StandUpBlitz | BlitzMove | GazeMove | MultipleBlock
        ),
        // `Rules::Common` is the edition-agnostic bucket; the bb2025 list is the current one.
        ffb_model::enums::Rules::Bb2025 | ffb_model::enums::Rules::Common => matches!(
            action,
            ViciousVines | Block | Pass | HandOver | ThrowBomb | ThrowTeamMate | KickTeamMate
                | Foul | StandUp | StandUpBlitz | MultipleBlock | SecureTheBall | Punt
        ),
    }
}

/// Java: `boolean changeToMove = Arrays.asList(...).contains(actingPlayer.getPlayerAction())` — the
/// flag carried by `DialogBloodlustActionParameter`. It only pre-selects the client's default
/// button, so no engine state depends on it; kept here so the two lists stay together and so a
/// future prompt that carries it does not have to re-derive it.
pub fn bloodlust_dialog_change_to_move(rules: ffb_model::enums::Rules, action: PlayerAction) -> bool {
    use PlayerAction::*;
    match rules {
        ffb_model::enums::Rules::Bb2016 => false,
        ffb_model::enums::Rules::Bb2020 => matches!(
            action,
            ViciousVines | Block | ThrowBomb | StandUp | BlitzMove | GazeMove | MultipleBlock
        ),
        ffb_model::enums::Rules::Bb2025 | ffb_model::enums::Rules::Common => matches!(
            action,
            ViciousVines | Block | ThrowBomb | StandUp | MultipleBlock | SecureTheBall
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::{Rules, TurnMode};
    use ffb_model::model::skill_def::SkillWithValue;

    fn add_player(team: &mut ffb_model::model::team::Team, id: &str, skills: Vec<SkillId>) {
        team.players.push(ffb_model::model::player::Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: skills.into_iter()
                .map(|s| SkillWithValue { skill_id: s, value: None }).collect(),
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
    }

    fn make_game(skills: Vec<SkillId>, action: Option<PlayerAction>) -> Game {
        let mut home = test_team("home", 0);
        add_player(&mut home, "vamp", skills);
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        game.acting_player.player_id = Some("vamp".into());
        game.acting_player.player_action = action;
        game.turn_mode = TurnMode::Regular;
        game
    }

    fn seed_for_d6(target: i32) -> u64 {
        for s in 0u64..10_000 {
            if GameRng::new(s).d6() == target { return s; }
        }
        panic!("no seed for d6={}", target);
    }

    #[test]
    fn negatraits_disabled_skips_roll() {
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Block));
        game.turn_mode = TurnMode::KickoffReturn;
        let out = StepBloodLust::new("fail").start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn no_blood_lust_skill_returns_next() {
        let mut game = make_game(vec![], Some(PlayerAction::Block));
        let out = StepBloodLust::new("fail").start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn successful_roll_returns_next_no_suffering() {
        let seed = seed_for_d6(3);
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Block));
        let out = StepBloodLust::new("fail").start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!game.acting_player.suffering_blood_lust);
    }

    #[test]
    fn failed_roll_block_action_shows_dialog() {
        let seed = seed_for_d6(1);
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Block));
        let mut step = StepBloodLust::new("fail");
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::Continue);
        assert!(matches!(out.prompt, Some(AgentPrompt::BloodlustAction { .. })));
        // Java sets setSufferingBloodLust(true) at the failure, BEFORE showing the action-change dialog.
        assert!(game.acting_player.suffering_blood_lust);
    }

    #[test]
    fn failed_roll_move_action_direct_failure() {
        let seed = seed_for_d6(1);
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Move));
        let out = StepBloodLust::new("fail").start(&mut game, &mut GameRng::new(seed));
        assert_ne!(out.action, StepAction::Continue);
        assert!(game.acting_player.suffering_blood_lust);
    }

    /// Java `BloodLustBehaviour.handleExecuteStepHook`, FAILURE branch:
    /// ```java
    /// step.publishParameter(new StepParameter(StepParameterKey.MOVE_STACK, null));
    /// getResult().setNextAction(StepAction.GOTO_LABEL, state.goToLabelOnFailure);
    /// ```
    /// The stack it discards is the path the client delivered at `StepInitSelecting` phase 2. When
    /// Rust reaches this step with NO square popped yet (`has_moved == false`) that publish throws
    /// nothing away, because Rust asks for the path later, at `StepInitMoving` — so the discard is
    /// owed to the first path that arrives. bb2025 seed 3 i=37 and bb2020 seed 8 i=26 are both this
    /// shape; without the debt the vampire walked a path Java had thrown away.
    #[test]
    fn a_failure_before_any_square_owes_the_move_stack_discard() {
        let seed = seed_for_d6(1);
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Move));
        game.acting_player.has_moved = false;
        let out = StepBloodLust::new("fail").start(&mut game, &mut GameRng::new(seed));
        assert_ne!(out.action, StepAction::Continue);
        assert!(game.acting_player.suffering_blood_lust);
        assert!(game.acting_player.blood_lust_discards_move_stack,
            "no square has been popped, so Java's MOVE_STACK=null discarded nothing here");
    }

    /// The other half of the pair, and the one that keeps the debt from double-counting: when a
    /// square HAS been popped (`StepInitMoving.executeStep`'s `actingPlayer.setHasMoved(true)`) the
    /// publish above throws away a real stack, exactly as Java does, and nothing is owed. bb2020
    /// seed 7 separates them — i=29 fails with `moved=false` and the vampire never moves, i=30
    /// fails with `moved=true` and walks four more squares.
    #[test]
    fn a_failure_after_a_square_owes_nothing() {
        let seed = seed_for_d6(1);
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Move));
        game.acting_player.has_moved = true;
        let out = StepBloodLust::new("fail").start(&mut game, &mut GameRng::new(seed));
        assert_ne!(out.action, StepAction::Continue);
        assert!(game.acting_player.suffering_blood_lust);
        assert!(!game.acting_player.blood_lust_discards_move_stack,
            "the MOVE_STACK=null publish already discarded the delivered path");
    }

    #[test]
    fn bloodlust_dialog_yes_publishes_alternate_action() {
        let seed = seed_for_d6(1);
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Pass));
        let mut step = StepBloodLust::new("fail");
        step.start(&mut game, &mut GameRng::new(seed));
        let out = step.handle_command(&Action::BloodlustAction { change: true }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        let ba = out.published.iter().find_map(|p| {
            if let StepParameter::BloodLustAction(v) = p { *v } else { None }
        });
        assert_eq!(ba, Some(PlayerAction::PassMove));
    }

    #[test]
    fn bloodlust_dialog_no_proceeds_to_action() {
        // Java WAIT_FOR_ACTION_CHANGE: keeping a BLOCK (bloodlustAction == null, not passing) routes to
        // NEXT_STEP so the declared action still executes while suffering blood lust — NOT the failure label.
        let seed = seed_for_d6(1);
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Block));
        let mut step = StepBloodLust::new("fail");
        step.start(&mut game, &mut GameRng::new(seed));
        let out = step.handle_command(&Action::BloodlustAction { change: false }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.acting_player.suffering_blood_lust);
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::BloodLustAction(Some(_)))));
    }

    #[test]
    fn failed_roll_with_trr_offers_reroll_before_dialog() {
        let seed = seed_for_d6(1);
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Block));
        game.turn_data_home.rerolls = 1;
        let mut step = StepBloodLust::new("fail");
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::Continue);
        assert!(matches!(out.prompt, Some(AgentPrompt::ReRollOffer { .. })));
    }

    #[test]
    fn goto_label_on_failure_used() {
        let seed = seed_for_d6(1);
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Move));
        let out = StepBloodLust::new("feed_label").start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("feed_label"));
    }

    #[test]
    fn set_parameter_goto_label_on_failure() {
        let mut step = StepBloodLust::default();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("x".to_string())));
        assert_eq!(step.goto_label_on_failure.as_deref(), Some("x"));
    }

    #[test]
    fn set_parameter_blood_lust_action() {
        let mut step = StepBloodLust::default();
        assert!(step.set_parameter(&StepParameter::BloodLustAction(Some(PlayerAction::Move))));
        assert_eq!(step.bloodlust_action, Some(PlayerAction::Move));
    }

    #[test]
    fn get_alternate_action_pass_becomes_pass_move() {
        assert_eq!(StepBloodLust::get_alternate_action(PlayerAction::Pass), PlayerAction::PassMove);
    }

    #[test]
    fn get_alternate_action_foul_becomes_foul_move() {
        assert_eq!(StepBloodLust::get_alternate_action(PlayerAction::Foul), PlayerAction::FoulMove);
    }

    #[test]
    fn get_alternate_action_default_becomes_move() {
        assert_eq!(StepBloodLust::get_alternate_action(PlayerAction::Block), PlayerAction::Move);
    }

    /// `failBloodLustForAction` — the dialog that offers to convert the declared action into its
    /// Blood Lust alternate — is a BB2020/BB2025 addition. `bb2016/BloodLustBehaviour` has no such
    /// branch: a failed roll goes straight to the failure path (`setSufferingBloodLust` +
    /// `MOVE_STACK null` + `GOTO_LABEL goToLabelOnFailure`), so the declared Block is CANCELLED.
    /// Running the bb2025 dialog for bb2016 let a Vampire that failed Blood Lust still throw its
    /// block: Java drew ONE die and passed the turn over, Rust drew thirteen (vampire bb2016
    /// seed 1 i=100 — Java 77 rng calls vs Rust 89).
    #[test]
    fn bb2016_failure_cancels_the_declared_block_without_the_conversion_dialog() {
        let run = |rules: Rules| {
            let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Block));
            game.rules = rules;
            let mut step = StepBloodLust::new(String::new());
            step.goto_label_on_failure = Some("endBlocking".into());
            let out = step.fail_blood_lust_for_action(&mut game, "vamp");
            out.action
        };

        assert_eq!(run(Rules::Bb2016), StepAction::GotoLabel,
            "bb2016 jumps to the failure label, cancelling the declared Block");
        assert_eq!(run(Rules::Bb2025), StepAction::Continue,
            "bb2025 stops for the action-conversion dialog instead");
    }

    /// Java (BOTH editions): `doRoll = UtilCards.hasUnusedSkill(actingPlayer, skill)` and
    /// `actingPlayer.markSkillUsed(skill)` — the used-set belongs to the ACTING PLAYER, which
    /// `changeActingPlayer` clears at every activation, so a Vampire rolls Blood Lust ONCE PER
    /// ACTIVATION. Reading/writing `Player.used_skills` (whole GAME) let a Vampire roll Blood Lust
    /// exactly once ever. Same per-activation shape as the Bone-head / Really Stupid fix.
    #[test]
    fn blood_lust_is_rolled_once_per_activation_not_once_per_game() {
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Move));
        game.rules = Rules::Bb2016;

        let mut rng = GameRng::new(seed_for_d6(6));
        let before = rng.call_count;
        let _ = StepBloodLust::new(String::new()).start(&mut game, &mut rng);
        assert_eq!(rng.call_count - before, 1, "the first activation rolls");
        assert!(game.acting_player.used_skills.contains(&SkillId::BloodLust),
            "the ACTING player's used-set is marked");
        assert!(!game.player("vamp").unwrap().used_skills.contains(&SkillId::BloodLust),
            "the Player's whole-game used-set must NOT be marked");

        // Same activation → no second roll.
        let before = rng.call_count;
        let _ = StepBloodLust::new(String::new()).start(&mut game, &mut rng);
        assert_eq!(rng.call_count - before, 0, "no second roll within the same activation");

        // New activation (changeActingPlayer clears the set) → rolls again.
        game.acting_player.used_skills.clear();
        let before = rng.call_count;
        let _ = StepBloodLust::new(String::new()).start(&mut game, &mut rng);
        assert_eq!(rng.call_count - before, 1, "a fresh activation rolls Blood Lust again");
    }

    /// The bb2025 membership list, copied off
    /// `ffb-server/.../skillbehaviour/bb2025/BloodLustBehaviour.java`:
    ///     {VICIOUS_VINES, BLOCK, PASS, HAND_OVER, THROW_BOMB, THROW_TEAM_MATE, KICK_TEAM_MATE,
    ///      FOUL, STAND_UP, STAND_UP_BLITZ, MULTIPLE_BLOCK, SECURE_THE_BALL, PUNT}
    /// BLITZ_MOVE and GAZE_MOVE are deliberately NOT in it.
    #[test]
    fn bb2025_bloodlust_dialog_matches_the_java_action_list() {
        use PlayerAction::*;
        let shown = [ViciousVines, Block, Pass, HandOver, ThrowBomb, ThrowTeamMate, KickTeamMate,
                     Foul, StandUp, StandUpBlitz, MultipleBlock, SecureTheBall, Punt];
        for a in shown {
            assert!(shows_bloodlust_action_dialog(Rules::Bb2025, a),
                "bb2025 Java list contains {a:?}");
        }
        // Everything the Java list omits — BLITZ_MOVE is the one that cost the campaign a red.
        for a in [Move, Blitz, BlitzMove, BlitzSelect, GazeMove, Gaze, PassMove, HandOverMove,
                  FoulMove, ThrowTeamMateMove, KickTeamMateMove, HailMaryPass, DumpOff, PuntMove] {
            assert!(!shows_bloodlust_action_dialog(Rules::Bb2025, a),
                "bb2025 Java list does NOT contain {a:?}");
        }
    }

    /// bb2020's list swaps SECURE_THE_BALL/PUNT for BLITZ_MOVE/GAZE_MOVE.
    #[test]
    fn bb2020_bloodlust_dialog_matches_the_java_action_list() {
        use PlayerAction::*;
        for a in [ViciousVines, Block, Pass, HandOver, ThrowBomb, ThrowTeamMate, KickTeamMate,
                  Foul, StandUp, StandUpBlitz, BlitzMove, GazeMove, MultipleBlock] {
            assert!(shows_bloodlust_action_dialog(Rules::Bb2020, a),
                "bb2020 Java list contains {a:?}");
        }
        for a in [Move, Blitz, SecureTheBall, Punt, PassMove, HandOverMove, FoulMove] {
            assert!(!shows_bloodlust_action_dialog(Rules::Bb2020, a),
                "bb2020 Java list does NOT contain {a:?}");
        }
    }

    /// bb2016/BloodLustBehaviour has no action-change branch whatsoever.
    #[test]
    fn bb2016_never_shows_the_bloodlust_dialog() {
        use PlayerAction::*;
        for a in [Move, Block, Blitz, BlitzMove, Pass, HandOver, Foul, StandUp, StandUpBlitz,
                  MultipleBlock, GazeMove, ThrowTeamMate] {
            assert!(!shows_bloodlust_action_dialog(Rules::Bb2016, a),
                "bb2016 shows no dialog for {a:?}");
        }
    }

    /// End-to-end on the live step: a BB2025 vampire who fails Blood Lust on a BLITZ_MOVE must NOT
    /// be prompted. Before the fix this returned `Continue` + `BloodlustAction`, which published
    /// MOVE_STACK/BLOOD_LUST_ACTION and cut the blitz short.
    #[test]
    fn bb2025_failed_bloodlust_on_blitz_move_asks_nothing() {
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::BlitzMove));
        game.rules = Rules::Bb2025;
        // No team re-roll available, so the failure is final and reaches the failure branch.
        game.turn_data_home.rerolls = 0;
        game.turn_data_away.rerolls = 0;
        let mut rng = GameRng::new(seed_for_d6(1));
        let out = StepBloodLust::new(String::new()).start(&mut game, &mut rng);
        assert!(out.prompt.is_none(),
            "bb2025 BLITZ_MOVE is not in the Java dialog list, so no prompt: {:?}", out.prompt);
        assert!(game.acting_player.suffering_blood_lust);
    }

    /// The same failure on a BLOCK — which IS in the bb2025 list — still prompts, so the fix did
    /// not simply delete the dialog.
    #[test]
    fn bb2025_failed_bloodlust_on_block_still_prompts() {
        let mut game = make_game(vec![SkillId::BloodLust], Some(PlayerAction::Block));
        game.rules = Rules::Bb2025;
        game.turn_data_home.rerolls = 0;
        game.turn_data_away.rerolls = 0;
        let mut rng = GameRng::new(seed_for_d6(1));
        let out = StepBloodLust::new(String::new()).start(&mut game, &mut rng);
        assert!(matches!(out.prompt, Some(AgentPrompt::BloodlustAction { .. })),
            "BLOCK is in the bb2025 list: {:?}", out.prompt);
    }
}
