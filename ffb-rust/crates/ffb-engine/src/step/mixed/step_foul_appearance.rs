/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.StepFoulAppearance` (BB2020 + BB2025).
///
/// Handles the Foul Appearance skill in the block/attack sequence.
/// Needs `GOTO_LABEL_ON_FAILURE` initialisation parameter.
///
/// Java flow (via FoulAppearanceBehaviour.handleExecuteStepHook, BB2020 edition):
/// 1. Resolve defender from TargetSelectionState (if set) or game.defender_id.
/// 2. If defender has FoulAppearance AND attacker lacks forceRollBeforeBeingBlocked cancel:
///    a. If re-rolling: consume re-roll or → handleFailure (hasBlocked=true, turnStarted=true → goto)
///    b. Roll 1d6 (rollSkill); success = roll >= minimumRollResistingFoulAppearance (= 2)
///    c. On success: commitTargetSelection + NEXT_STEP
///    d. On failure: ask for re-roll, or → handleFailure
/// 3. Else: NEXT_STEP immediately
///
/// Java: `StepFoulAppearance extends AbstractStepWithReRoll` (mixed, BB2020 + BB2025).
use ffb_model::model::game::Game;
use ffb_model::enums::{SkillId, ReRollSource, PlayerAction, PS_PRONE};
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use ffb_model::events::GameEvent;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::{ReRollState, find_skill_reroll_source};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

/// Java: `StepFoulAppearance` (mixed, BB2020 + BB2025).
pub struct StepFoulAppearance {
    /// Java: `state.goToLabelOnFailure` (mandatory init param GOTO_LABEL_ON_FAILURE).
    pub goto_label_on_failure: String,
    /// Java: AbstractStepWithReRoll fields
    pub re_roll_state: ReRollState,
    /// Roll value (0 = not yet rolled)
    pub roll: i32,
    /// True for the copy of this step that sits in the SELECT sequence (`StepParameter::InSelect`).
    ///
    /// `bb2020/SelectBlitzTarget.java:35` rolls Foul Appearance in the blitz's SELECT phase —
    /// after BLOOD_LUST, BEFORE JUMP_UP/STAND_UP and before the blitz's GO_FOR_IT — while
    /// `bb2025/BlitzBlock.java:37` rolls it inside the block sequence, after GO_FOR_IT. Rust runs
    /// the BB2025 step-set for BB2020 games, so the select copy exists in every edition and this
    /// flag makes it a no-op except for the one case Java puts there: a BB2020 blitz.
    pub in_select: bool,
}

impl StepFoulAppearance {
    pub fn new(goto_label_on_failure: impl Into<String>) -> Self {
        Self {
            goto_label_on_failure: goto_label_on_failure.into(),
            re_roll_state: ReRollState::new(),
            roll: 0,
            in_select: false,
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // The SELECT-sequence copy only stands in for `bb2020/SelectBlitzTarget.java:35`; every
        // other edition/action rolls Foul Appearance from its own sequence, so skip silently here
        // (a second roll would consume an extra die and desync the whole stream).
        if self.in_select
            && !(game.rules == ffb_model::enums::Rules::Bb2020
                && game.acting_player.player_action.map(|pa| pa.is_blitzing()).unwrap_or(false))
        {
            return StepOutcome::next();
        }

        // Java: resolve defender from TargetSelectionState (if selected+committed) or game.defender_id
        let defender_id = {
            let from_ts = game.field_model.target_selection_state.as_ref()
                .filter(|ts| ts.is_selected() && ts.is_committed())
                .and_then(|ts| ts.get_selected_player_id().cloned());
            from_ts.or_else(|| game.defender_id.clone())
        };
        let defender_has_fa = defender_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill(SkillId::FoulAppearance))
            .unwrap_or(false);
        // Java: `!UtilCards.hasSkillToCancelProperty(actingPlayer.getPlayer(),
        // NamedProperties.forceRollBeforeBeingBlocked)` — the attacker skips the Foul Appearance roll
        // only if it has a skill that CANCELS the property (e.g. Nerves of Steel), NOT if it merely
        // HAS the property itself. The old code tested `has_skill_property(FORCE_ROLL_BEFORE_BEING_
        // BLOCKED)` — so a Foul-Appearance attacker (e.g. any Nurgle player) wrongly cancelled its own
        // Foul Appearance roll against a Foul-Appearance defender, skipping the d6 Java always rolls
        // and desyncing the dice stream (nurgle seed 1 i=1: away_03 blitz).
        let attacker_cancels = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| ffb_model::util::util_cards::UtilCards::has_skill_to_cancel_property(p, NamedProperties::FORCE_ROLL_BEFORE_BEING_BLOCKED))
            .unwrap_or(false);

        if !defender_has_fa || attacker_cancels {
            return StepOutcome::next();
        }

        // Java runs the BB2020 blitz's Foul Appearance exactly once, in SelectBlitzTarget. Rust has
        // to carry TWO copies of that one roll: the SELECT sequence covers a PRONE blitzer (whose
        // select body runs, standing it up), the EndSelecting blitz-dispatch prepend covers a
        // STANDING one (whose select body is force-gotoed straight to END_SELECTING). Only one of
        // them may roll — mark the activation on the first, and bail out on the second. Cleared with
        // the rest of `used_skills` when the acting player changes, and NOT consulted by the
        // blitz-block copy, so a Frenzy re-block still rolls again as Java does.
        if self.in_select {
            let first_entry = self.re_roll_state.re_rolled_action.is_none();
            if first_entry {
                if game.acting_player.used_skills.contains(&SkillId::FoulAppearance) {
                    return StepOutcome::next();
                }
                game.acting_player.used_skills.insert(SkillId::FoulAppearance);
            }
        }

        // Java: if (FOUL_APPEARANCE == reRolledAction)
        //         if (source == null || !useReRoll) → handleFailure
        let already_rerolled = self.re_roll_state.re_rolled_action
            .as_ref().map(|a| a.name == "FOUL_APPEARANCE").unwrap_or(false);

        if already_rerolled {
            let pid = game.acting_player.player_id.clone();
            let source_opt = self.re_roll_state.re_roll_source.clone();
            let consumed = source_opt
                .as_ref()
                .map(|s| use_reroll(game, s, pid.as_deref().unwrap_or(""), rng))
                .unwrap_or(false);
            if !consumed {
                return self.fail_fa(game);
            }
            self.roll = 0;
        }

        // Java: roll = diceRoller.rollSkill() (1d6); minimumRoll = 2
        if self.roll == 0 {
            self.roll = rng.d6();
        }
        let minimum_roll = DiceInterpreter::minimum_roll_resisting_foul_appearance();
        let may_block = DiceInterpreter::is_skill_roll_successful(self.roll, minimum_roll);

        // Coverage: `GameEvent::FoulAppearanceRoll` had no construction site anywhere in the engine,
        // so the counter read 0 across 8,700 games for a roll that demonstrably fires — this file's
        // own BB2020 ordering bug was found by watching it in a dice trace. Report-only.
        let roll_event = GameEvent::FoulAppearanceRoll {
            player_id: game.acting_player.player_id.clone().unwrap_or_default(),
            roll: self.roll,
            failed: !may_block,
        };

        if may_block {
            // Java: step.commitTargetSelection() — calls targetSelectionState.commit() if not null
            if let Some(ref mut ts) = game.field_model.target_selection_state {
                ts.commit();
            }
            return StepOutcome::next().with_event(roll_event);
        }

        // Failure — try re-roll if first failure
        if !already_rerolled {
            use ffb_model::model::re_rolled_action::ReRolledAction;
            self.re_roll_state.re_rolled_action = Some(ReRolledAction::new("FOUL_APPEARANCE"));

            let pid = game.acting_player.player_id.clone();
            let skill_source = find_skill_reroll_source(game, "FOUL_APPEARANCE");
            if let Some(source) = skill_source {
                use_reroll(game, &source, pid.as_deref().unwrap_or(""), rng);
                self.re_roll_state.re_roll_source = Some(source);
                self.roll = 0;
                return self.execute_step(game, rng);
            }

            // Java: if (reRolled || !askForReRollIfAvailable) → handleFailure
            // Note: BB2020 version only asks if NOT already reRolled
            if let Some(prompt) = ask_for_reroll_if_available(game, "FOUL_APPEARANCE", minimum_roll, false) {
                self.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
                self.roll = 0;
                return StepOutcome::cont().with_prompt(prompt).with_event(roll_event);
            }
        }

        self.fail_fa(game).with_event(roll_event)
    }

    fn fail_fa(&mut self, game: &mut Game) -> StepOutcome {
        let player_action = game.acting_player.player_action;

        // Java: if (actingPlayer.isStandingUp() && (BLITZ_MOVE || blockAction || GAZE_MOVE || kickingDowned))
        //         setPlayerState(player, state.changeBase(PRONE).changeActive(false))
        if game.acting_player.standing_up {
            let set_prone = player_action.map(|pa|
                pa == PlayerAction::BlitzMove
                || pa.is_block_action()
                || pa == PlayerAction::GazeMove
                || pa.is_kicking_downed()
                // BB2020 BRIDGE. Java's list has no BLITZ because it never needs one: BB2020
                // resolves Foul Appearance inside `SelectBlitzTarget`, BEFORE JUMP_UP/STAND_UP, so a
                // failure gotos END_BLITZING and the blitzer simply never stands
                // (`bb2020/SelectBlitzTarget.java:35-36`). Rust's agent commits to blitz+target in a
                // single command, so `StepEndSelecting` dispatches an inline activation and the
                // stand-up happens outside it — there is no abort point before the stand-up to jump
                // to. Relocating the step cannot fix that (ITER103-110: every variant measured
                // 0/100, 15/100 or 25/100 against an 86/100 baseline). Reverting the stand-up here
                // reaches the same END STATE Java reaches, on the same dice.
                // ...but ONLY before the blitzer has actually blocked. Frenzy forces a second block,
                // which re-runs this step; by then the first Foul Appearance roll has already
                // succeeded and Java has committed the stand-up, so a failure on the SECOND roll
                // leaves the player Standing (Java falls through to INIT_MOVING and the activation
                // is deselected). Without the has_blocked guard Rust retroactively undid a stand-up
                // Java keeps — necromantic seed 37 step 182: FA 4 (pass), block 4 (push), Frenzy FA
                // 1 (fail), Java h01 Standing vs Rust h01 Prone.
                || (game.rules == ffb_model::enums::Rules::Bb2020
                    && pa.is_blitzing()
                    && !game.acting_player.has_blocked)
            ).unwrap_or(false);
            if set_prone {
                if let Some(pid) = game.acting_player.player_id.clone() {
                    if let Some(state) = game.field_model.player_state(&pid) {
                        game.field_model.set_player_state(&pid, state.change_base(PS_PRONE).change_active(false));
                    }
                }
            }
        }

        game.acting_player.has_blocked = true;
        game.turn_data_mut().turn_started = true;

        // Java: targetSelectionState.failed(); if blitzing → blitzUsed = true
        let is_blitzing = player_action.map(|pa| pa.is_blitzing()).unwrap_or(false);
        if let Some(ref mut ts) = game.field_model.target_selection_state {
            ts.failed();
            if is_blitzing {
                game.turn_data_mut().blitz_used = true;
            }
        } else if self.in_select && is_blitzing {
            // The select-phase copy runs BEFORE StepInitBlocking builds the TargetSelectionState, so
            // Java's `targetSelectionState != null` guard cannot be read literally here — in Java the
            // state always exists by this point, because StepSelectBlitzTarget created it at the top
            // of the same sequence. Mark the blitz used anyway: this failure skips the EndSelecting
            // blitz dispatch (the only other place Rust sets the flag), and leaving it false let the
            // team blitz a second time in the same turn — nurgle mirror seed 6 i=39, where away_02
            // was offered a Block that Java had already filtered out.
            game.turn_data_mut().blitz_used = true;
        }

        // Java handleFailure END_PLAYER_ACTION publish — a genuine EDITION SPLIT:
        //   bb2025 FoulAppearanceBehaviour: GAZE || isBlockAction() || isBlitzing()
        //   bb2020 FoulAppearanceBehaviour: GAZE || isBlockAction()      (NO isBlitzing)
        // This shared step ran the bb2025 shape for every edition, so a bb2020 Witch Elf whose
        // FRENZY second-block Foul Appearance failed had her whole activation ended where Java
        // cancels only the second block and CONTINUES THE BLITZ MOVE (dark_elf bb2020 @1e6
        // seed 48 i=67: Java resumes into a Tentacles check, Rust went to the next activation).
        let end_action = player_action.map(|pa|
            pa.is_gaze() || pa.is_block_action()
                || (game.rules == ffb_model::enums::Rules::Bb2025 && pa.is_blitzing())
        ).unwrap_or(false);

        // Java: game.setDefenderId(null)
        game.defender_id = None;

        let label = self.goto_label_on_failure.clone();
        let out = StepOutcome::goto(&label);
        if end_action {
            out.publish(StepParameter::EndPlayerAction(true))
        } else {
            out
        }
    }
}

impl Default for StepFoulAppearance {
    fn default() -> Self { Self::new("") }
}

impl Step for StepFoulAppearance {
    fn id(&self) -> StepId { StepId::FoulAppearance }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::UseReRoll { use_reroll: true } => self.execute_step(game, rng),
            Action::UseReRoll { use_reroll: false } => {
                self.re_roll_state.re_roll_source = None;
                self.execute_step(game, rng)
            }
            _ => self.execute_step(game, rng),
        }
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            StepParameter::InSelect(v) => { self.in_select = *v; true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::{Rules, SkillId, PlayerType, PlayerGender, TurnMode, PlayerState};
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::util::rng::GameRng;
    use std::collections::HashSet;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, id: &str, skills: Vec<SkillId>) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: skills.iter().map(|&s| SkillWithValue { skill_id: s, value: None }).collect(),
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
});
    }

    #[test]
    fn no_defender_returns_next_step() {
        let mut step = StepFoulAppearance::new("fail");
        let mut game = make_game();
        game.defender_id = None;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn defender_without_foul_appearance_returns_next() {
        let mut game = make_game();
        add_player(&mut game, "def", vec![]);
        game.defender_id = Some("def".into());
        let mut step = StepFoulAppearance::new("fail");
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn successful_roll_returns_next_step() {
        let mut game = make_game();
        add_player(&mut game, "atk", vec![]);
        add_player(&mut game, "def", vec![SkillId::FoulAppearance]);
        game.acting_player.player_id = Some("atk".into());
        game.defender_id = Some("def".into());
        let mut step = StepFoulAppearance::new("fail");
        step.roll = 2; // success (>= 2)
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn failed_roll_no_reroll_goes_to_label() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        add_player(&mut game, "atk", vec![]);
        add_player(&mut game, "def", vec![SkillId::FoulAppearance]);
        game.acting_player.player_id = Some("atk".into());
        game.defender_id = Some("def".into());
        let mut step = StepFoulAppearance::new("fa_fail");
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fa_fail"));
        assert!(game.acting_player.has_blocked);
        assert!(game.turn_data().turn_started);
    }

    #[test]
    fn failed_roll_with_trr_offers_reroll() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        add_player(&mut game, "atk", vec![]);
        add_player(&mut game, "def", vec![SkillId::FoulAppearance]);
        game.acting_player.player_id = Some("atk".into());
        game.defender_id = Some("def".into());
        let mut step = StepFoulAppearance::new("fail");
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
        assert!(out.prompt.is_some());
    }

    /// Java edition split: bb2020's handleFailure publishes END_PLAYER_ACTION only for
    /// GAZE/block actions — a failed BLITZ Foul Appearance keeps the activation alive (the
    /// blitz move continues); bb2025 also ends blitzes.
    #[test]
    fn failed_blitz_fa_ends_action_only_in_bb2025() {
        for (rules, expect_end) in [(ffb_model::enums::Rules::Bb2020, false),
                                    (ffb_model::enums::Rules::Bb2025, true)] {
            let mut game = make_game();
            game.rules = rules;
            game.home_playing = true;
            game.acting_player.player_id = Some("atk".into());
            game.acting_player.player_action = Some(PlayerAction::Blitz);
            let out = StepFoulAppearance::new("END_BLOCKING").fail_fa(&mut game);
            let ended = out.published.iter().any(|p| matches!(
                p, crate::step::framework::StepParameter::EndPlayerAction(true)));
            assert_eq!(ended, expect_end, "{rules:?}");
        }
    }

    #[test]
    fn set_parameter_goto_label_on_failure() {
        let mut step = StepFoulAppearance::new("old_label");
        let accepted = step.set_parameter(&StepParameter::GotoLabelOnFailure("new_label".into()));
        assert!(accepted);
        assert_eq!(step.goto_label_on_failure, "new_label");
    }

    /// The SELECT-sequence copy stands in for `bb2020/SelectBlitzTarget.java:35` and must roll for
    /// a BB2020 blitz only — anything else already rolls Foul Appearance from its own sequence, so
    /// a second roll here would eat a die and desync the stream.
    #[test]
    fn in_select_copy_only_rolls_for_a_bb2020_blitz() {
        use ffb_model::enums::{PlayerAction, Rules};
        let run = |rules: Rules, action: PlayerAction| {
            let mut game = Game::new(test_team("home", 0), test_team("away", 0), rules);
            game.turn_mode = TurnMode::Regular;
            game.home_playing = true;
            game.turn_data_home.rerolls = 0;
            add_player(&mut game, "atk", vec![]);
            add_player(&mut game, "def", vec![SkillId::FoulAppearance]);
            game.acting_player.player_id = Some("atk".into());
            game.acting_player.player_action = Some(action);
            game.defender_id = Some("def".into());
            let mut step = StepFoulAppearance::new("fa_fail");
            step.in_select = true;
            step.roll = 1; // a failure, so a roll that happens is unmistakable
            step.start(&mut game, &mut GameRng::new(0))
        };
        assert_eq!(run(Rules::Bb2020, PlayerAction::Blitz).action, StepAction::GotoLabel);
        assert_eq!(run(Rules::Bb2020, PlayerAction::Block).action, StepAction::NextStep);
        assert_eq!(run(Rules::Bb2025, PlayerAction::Blitz).action, StepAction::NextStep);
    }

    /// A BB2020 blitz carries the select copy TWICE — once in the select sequence (prone blitzer),
    /// once in the EndSelecting dispatch prepend (standing blitzer) — because only one of the two
    /// ever runs. If both were to run, the second must not roll again.
    #[test]
    fn in_select_copy_rolls_only_once_per_activation() {
        use ffb_model::enums::{PlayerAction, Rules};
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020);
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        add_player(&mut game, "atk", vec![]);
        add_player(&mut game, "def", vec![SkillId::FoulAppearance]);
        game.acting_player.player_id = Some("atk".into());
        game.acting_player.player_action = Some(PlayerAction::Blitz);
        game.defender_id = Some("def".into());

        let mut first = StepFoulAppearance::new("fa_fail");
        first.in_select = true;
        let mut rng = GameRng::new(0);
        first.start(&mut game, &mut rng);
        let after_first = rng.call_count;

        let mut second = StepFoulAppearance::new("fa_fail");
        second.in_select = true;
        let out = second.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(rng.call_count, after_first, "the second copy must not roll a die");
    }

    #[test]
    fn set_parameter_rejects_unknown() {
        let mut step = StepFoulAppearance::new("fail");
        let rejected = !step.set_parameter(&StepParameter::EndTurn(true));
        assert!(rejected);
    }

    #[test]
    fn failed_with_standing_up_blitz_move_sets_player_prone() {
        use ffb_model::enums::{PlayerAction, PS_PRONE, PS_STANDING};
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        add_player(&mut game, "atk", vec![]);
        add_player(&mut game, "def", vec![SkillId::FoulAppearance]);
        game.acting_player.player_id = Some("atk".into());
        game.acting_player.player_action = Some(PlayerAction::BlitzMove);
        game.acting_player.standing_up = true;
        game.acting_player.has_blocked = false;
        game.field_model.set_player_state("atk", PlayerState::new(PS_STANDING));
        game.defender_id = Some("def".into());
        let mut step = StepFoulAppearance::new("fa_fail");
        step.roll = 1;
        step.start(&mut game, &mut GameRng::new(0));
        // Player should be set to PRONE + inactive
        let state = game.field_model.player_state("atk").unwrap();
        assert_eq!(state.base(), PS_PRONE);
        assert!(!state.is_active());
    }

    /// BB2020 bridge: a failed Foul Appearance must leave a BB2020 blitzer PRONE. Java never needs
    /// `BLITZ` in the revert list because it resolves Foul Appearance in `SelectBlitzTarget`, before
    /// the stand-up; Rust's single-command blitz has no abort point there, so it reverts instead.
    /// BB2025/BB2016 must keep Java's list exactly — a `Blitz` action does NOT revert there.
    #[test]
    fn bb2020_failed_foul_appearance_reverts_a_blitzer_to_prone() {
        use ffb_model::enums::{PlayerAction, PS_PRONE, PS_STANDING, Rules};
        let run = |rules: Rules| {
            let mut game = Game::new(test_team("home", 0), test_team("away", 0), rules);
            game.turn_mode = TurnMode::Regular;
            game.home_playing = true;
            game.turn_data_home.rerolls = 0;
            add_player(&mut game, "atk", vec![]);
            add_player(&mut game, "def", vec![SkillId::FoulAppearance]);
            game.acting_player.player_id = Some("atk".into());
            game.acting_player.player_action = Some(PlayerAction::Blitz);
            game.acting_player.standing_up = true;
            game.field_model.set_player_state("atk", PlayerState::new(PS_STANDING));
            game.defender_id = Some("def".into());
            let mut step = StepFoulAppearance::new("fa_fail");
            step.roll = 1; // guaranteed failure (minimum is 2)
            step.start(&mut game, &mut GameRng::new(0));
            game.field_model.player_state("atk").unwrap().base()
        };
        assert_eq!(run(Rules::Bb2020), PS_PRONE,
            "a BB2020 blitzer that fails Foul Appearance must end up prone, as it does in Java");
        assert_eq!(run(Rules::Bb2025), PS_STANDING,
            "BB2025 keeps Java's revert list exactly — BLITZ is not in it");
    }

    /// Frenzy forces a second block, which re-runs this step. By then the FIRST Foul Appearance
    /// roll has succeeded and Java has committed the stand-up, so a failure on the second roll must
    /// leave the blitzer Standing. necromantic seed 37 step 182: FA 4 (pass), block 4 (push),
    /// Frenzy FA 1 (fail) — Java h01 Standing, Rust was reverting it to Prone.
    #[test]
    fn bb2020_blitzer_that_has_already_blocked_is_not_reverted_to_prone() {
        use ffb_model::enums::{PlayerAction, PS_STANDING, Rules};
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020);
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        add_player(&mut game, "atk", vec![]);
        add_player(&mut game, "def", vec![SkillId::FoulAppearance]);
        game.acting_player.player_id = Some("atk".into());
        game.acting_player.player_action = Some(PlayerAction::Blitz);
        game.acting_player.standing_up = true;
        game.acting_player.has_blocked = true; // the Frenzy re-entry
        game.field_model.set_player_state("atk", PlayerState::new(PS_STANDING));
        game.defender_id = Some("def".into());
        let mut step = StepFoulAppearance::new("fa_fail");
        step.roll = 1;
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.field_model.player_state("atk").unwrap().base(), PS_STANDING);
    }

    #[test]
    fn failed_with_blitzing_action_sets_blitz_used_and_target_selection_failed() {
        use ffb_model::enums::PlayerAction;
        use ffb_model::model::target_selection_state::{TargetSelectionState, TargetSelectionStatus};
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        add_player(&mut game, "atk", vec![]);
        add_player(&mut game, "def", vec![SkillId::FoulAppearance]);
        game.acting_player.player_id = Some("atk".into());
        game.acting_player.player_action = Some(PlayerAction::Blitz);
        game.defender_id = Some("def".into());
        let ts = TargetSelectionState::new("def");
        game.field_model.target_selection_state = Some(ts);
        let mut step = StepFoulAppearance::new("fa_fail");
        step.roll = 1;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.turn_data().blitz_used);
        assert_eq!(
            game.field_model.target_selection_state.as_ref().map(|ts| ts.status),
            Some(TargetSelectionStatus::FAILED)
        );
        assert!(game.defender_id.is_none());
    }

    #[test]
    fn failed_with_blitz_action_publishes_end_player_action() {
        // bb2025 FoulAppearanceBehaviour.handleFailure adds `isBlitzing()` to the
        // END_PLAYER_ACTION condition (bb2020 does not) — this crate targets bb2025.
        use ffb_model::enums::PlayerAction;
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        add_player(&mut game, "atk", vec![]);
        add_player(&mut game, "def", vec![SkillId::FoulAppearance]);
        game.acting_player.player_id = Some("atk".into());
        game.acting_player.player_action = Some(PlayerAction::Blitz);
        game.defender_id = Some("def".into());
        let mut step = StepFoulAppearance::new("fa_fail");
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(
            out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))),
            "blitzing action should publish EndPlayerAction(true) on failure (bb2025)"
        );
    }

    #[test]
    fn failed_with_block_action_publishes_end_player_action() {
        use ffb_model::enums::PlayerAction;
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        add_player(&mut game, "atk", vec![]);
        add_player(&mut game, "def", vec![SkillId::FoulAppearance]);
        game.acting_player.player_id = Some("atk".into());
        game.acting_player.player_action = Some(PlayerAction::Block);
        game.defender_id = Some("def".into());
        let mut step = StepFoulAppearance::new("fa_fail");
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))));
    }
}
