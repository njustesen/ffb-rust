use ffb_model::model::game::Game;
use ffb_model::enums::ReRollSource;
use ffb_model::util::rng::GameRng;
use ffb_model::report::report_jump_roll::ReportJumpRoll;
use ffb_model::report::report_id::ReportId;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::drop_player_context::SteadyFootingContext;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::ReRollState;
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use ffb_mechanics::bb2016::jump_mechanic::JumpMechanic;
use ffb_mechanics::jump_mechanic::JumpMechanic as JumpMechanicTrait;
use ffb_mechanics::mechanics::minimum_roll_base_bb2016;
use ffb_mechanics::modifiers::jump_modifier_factory::JumpModifierFactory;
use ffb_mechanics::modifiers::jump_context::JumpContext;
use ffb_model::types::FieldCoordinate;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.move.StepJump.
///
/// BB2016 StepJump holds a single `StepState { goToLabelOnFailure }` and delegates
/// all logic to `executeStepHooks(this, state)` in Java.
///
/// In Rust, we have no hook infrastructure yet — this is implemented with the same
/// agility-roll logic as BB2025's StepJump but without BB2025-specific modifiers.
/// The hook infrastructure (JumpModifierFactory, canStillJump, checkDivingTackle,
/// ignoreModifiers) is stubbed out.
///
/// Init params: GOTO_LABEL_ON_FAILURE (mandatory).
///
/// Logic:
/// - If !actingPlayer.isJumping() → NEXT_STEP
/// - Roll 1d6 vs minimum_roll_base_bb2016(agility, jump modifiers)
/// - Success → clear jumping, NEXT_STEP + JUMPED(true)
/// - Failure → TRR offer if available, else fail_jump: GOTO_LABEL + COORDINATE_FROM
///
/// BB2016's JumpModifierCollection is empty, but `super.findModifiers` still supplies the
/// player's own registered modifiers -- `bb2016.VeryLongLegs` registers a REGULAR -1.
/// BB2016 agility_with_modifiers() == agility (no stat-injury pipeline yet).
/// no-op: DivingTackle executeStepHooks skipped in headless (SkillBehaviour registry not ported).
/// canStillJump: wired via BB2016 JumpMechanic.
/// client-only: checkDivingTackle dialog — headless auto-skips diving tackle activation.
pub struct StepJump {
    /// Java: StepState.goToLabelOnFailure
    pub goto_label_on_failure: String,
    /// Internal roll (not a Java field — it's local in executeStepHooks)
    pub roll: i32,
    /// Java: AbstractStepWithReRoll fields
    pub re_roll_state: ReRollState,
}

impl StepJump {
    pub fn new(goto_label_on_failure: String) -> Self {
        Self {
            goto_label_on_failure,
            roll: 0,
            re_roll_state: ReRollState::new(),
        }
    }
}

impl Default for StepJump {
    fn default() -> Self { Self::new(String::new()) }
}

impl Step for StepJump {
    fn id(&self) -> StepId { StepId::Jump }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if let Action::UseReRoll { use_reroll: false } = action {
            self.re_roll_state.re_roll_source = None;
        }
        // client-only: CLIENT_PLAYER_CHOICE DIVING_TACKLE mode — headless never receives this
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            _ => false,
        }
    }
}

impl StepJump {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: doLeap = actingPlayer.isJumping() && mechanic.canStillJump(game, actingPlayer)
        let mechanic = JumpMechanic::new();
        let do_leap = game.acting_player.jumping
            && mechanic.can_still_jump(game, &game.acting_player.clone());

        if !do_leap {
            return StepOutcome::next();
        }

        let already_rerolled = self.re_roll_state.re_rolled_action
            .as_ref().map(|a| a.name == "JUMP").unwrap_or(false);

        if already_rerolled {
            let pid = game.acting_player.player_id.as_deref().unwrap_or("").to_owned();
            let source_opt = self.re_roll_state.re_roll_source.clone();
            let consumed = source_opt
                .as_ref()
                .map(|s| use_reroll(game, s, &pid, rng))
                .unwrap_or(false);
            if !consumed {
                // Java (LeapBehaviour.handleExecuteStepHook): reRollSource == null || !useReRoll(...)
                // → publish INJURY_TYPE + GOTO_LABEL, doLeap = false. Crucially, this branch does
                // NOT call actingPlayer.setJumping(false) — only the leap() SUCCESS/FAILURE branch
                // does. So `jumping` remains true here, unlike the dice-roll-failure path below.
                return self.fail_without_clearing_jumping(game);
            }
            // Java NEVER clears `roll` when it OFFERS a re-roll: `doRoll` is `reRolled || ...`,
            // so a fresh die is drawn only once a re-roll has actually been CONSUMED. Zeroing it
            // at the offer made a DECLINED offer reach the failure path with `roll == 0`, and the
            // `roll > 1` test there is what decides whether a failed leaper lands on the target
            // square or is put back where he started
            // (`updatePlayerAndBallPosition(player, moveStart)`). Declining a jump re-roll
            // therefore teleported the faller back onto his own square - slann bb2025 seed 22,
            // where Java stunned him at (12,4) and Rust at (13,2).
            self.roll = 0;
        }

        if self.roll == 0 {
            self.roll = rng.d6();
        }

        let player_id = game.acting_player.player_id.clone();
        // agility_with_modifiers() == agility in current model.
        let agility = player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.agility_with_modifiers())
            .unwrap_or(3);
        // Java `LeapBehaviour.leap`:
        //   Set<JumpModifier> jumpModifiers =
        //       modifierFactory.findModifiers(new JumpContext(game, player, null, null));
        //   int minimumRoll = mechanic.minimumRollJump(player, jumpModifiers);
        //
        // BB2016's `JumpModifierCollection` really is empty and its factory answers `false` to
        // `isAffectedByTackleZones`, which is what the old `&[]` was reasoning from — but
        // `super.findModifiers` still contributes the player's OWN registered modifiers, and
        // `bb2016.VeryLongLegs` registers `JumpModifier("Very Long Legs", -1, REGULAR)`. So a
        // Very-Long-Legs leaper jumps on one better than its bare agility, and Rust rolled it
        // one worse. Every BB2016 Leap carrier that has the mutation is affected: slann,
        // slann_fumbbl and goblin all field both on the same player.
        //
        // The second half of the same line: `mechanic` here is the BB2016 `AgilityMechanic`, whose
        // `minimumRollJump` is `max(2, getAgilityRollBase(ag) + modifiers)` on the old `7 - AG`
        // scale -- not the BB2020+ `ag + modifiers`. This called `minimum_roll_jump`, which IS the
        // BB2020 formula. The two errors CANCELLED for the only players who could reach this code
        // (AG3 leapers WITH Very Long Legs: 7-3-1 == 3 == 3+0), so fixing either one alone makes
        // the step wrong; they are one fix.
        let modifier_total: i32 = player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| {
                let ctx = JumpContext::new(game, p, FieldCoordinate::new(0, 0), FieldCoordinate::new(0, 0));
                JumpModifierFactory::for_rules(game.rules)
                    .find_skill_modifiers(&ctx, 0, 0)
                    .iter()
                    .map(|m| m.get_modifier())
                    .sum()
            })
            .unwrap_or(0);
        let minimum_roll = minimum_roll_base_bb2016(agility, modifier_total);
        let successful = DiceInterpreter::is_skill_roll_successful(self.roll, minimum_roll);

        // Java (LeapBehaviour): step.getResult().addReport(new ReportJumpRoll(actingPlayer.getPlayerId(),
        //         successful, roll, minimumRoll, reRolled, jumpModifiers.toArray(...)))
        let re_rolled = self.re_roll_state.re_rolled_action.as_ref().map(|a| a.name == "JUMP").unwrap_or(false)
            && self.re_roll_state.re_roll_source.is_some();
        game.report_list.add(ReportJumpRoll::new(
            game.acting_player.player_id.clone(),
            successful,
            self.roll,
            minimum_roll,
            re_rolled,
            vec![],
        ));

        // `GameEvent::JumpRoll` had no producer anywhere in the engine (BACKLOG E12), so
        // `jumpRoll` would have stayed 0 in every coverage harvest even with the agent declaring
        // jumps. Emitted on `StepMoveDodge`'s rule: one event per RESOLVED roll.
        let roll_event = ffb_model::events::GameEvent::JumpRoll {
            player_id: player_id.clone().unwrap_or_default(),
            target: minimum_roll,
            roll: self.roll,
            success: successful,
        };

        if successful {
            game.acting_player.jumping = false;
            mark_leap_used(game);
            return StepOutcome::next()
                .publish(StepParameter::Jumped(true))
                .with_event(roll_event);
        }

        // Try re-roll on first failure
        if !already_rerolled {
            use ffb_model::model::re_rolled_action::ReRolledAction;
            self.re_roll_state.re_rolled_action = Some(ReRolledAction::new("JUMP"));

            // TRR offer (no skill re-roll for JUMP in BB2016)
            if let Some(prompt) = ask_for_reroll_if_available(game, "JUMP", minimum_roll, false) {
                self.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
                return StepOutcome::cont().with_prompt(prompt).with_event(roll_event);
            }
        }

        self.handle_failure(game).with_event(roll_event)
    }

    fn handle_failure(&mut self, game: &mut Game) -> StepOutcome {
        game.acting_player.jumping = false;
        mark_leap_used(game);
        self.fail_without_clearing_jumping(game)
    }

    /// Java (LeapBehaviour.handleExecuteStepHook): the `reRollSource == null || !useReRoll(...)`
    /// branch publishes INJURY_TYPE and GOTOs the failure label but does NOT reset
    /// `actingPlayer.setJumping(false)` — unlike the leap() FAILURE branch.
    fn fail_without_clearing_jumping(&mut self, _game: &mut Game) -> StepOutcome {
        let ctx = SteadyFootingContext::from_injury_type_name("InjuryTypeDropJump".into());
        let label = self.goto_label_on_failure.clone();
        StepOutcome::goto(&label)
            .publish(StepParameter::SteadyFootingContext(Box::new(ctx)))
    }
}

/// Java `LeapBehaviour`: BOTH the SUCCESS and the FAILURE arm of `leap(step)` run
/// `actingPlayer.markSkillUsed(skill)` right after `setJumping(false)`.
///
/// That mark is the whole of BB2016's once-per-activation rule — `JumpMechanic.canStillJump` is
/// `UtilCards.hasUnusedSkillWithProperty(actingPlayer, canLeap)`, and it is read again by
/// `updateMoveSquares` on every following move square. Without it a BB2016 leaper could jump
/// every square of its move. It was missing here because nothing in this harness had ever
/// DECLARED a jump (BACKLOG E12), so the rule had never been exercised.
fn mark_leap_used(game: &mut Game) {
    use ffb_model::model::property::named_properties::NamedProperties;
    let Some(pid) = game.acting_player.player_id.clone() else { return };
    let leaps: Vec<ffb_model::enums::SkillId> = match game.player(&pid) {
        Some(p) => p
            .all_skill_ids()
            .filter(|id| id.properties().contains(&NamedProperties::CAN_LEAP))
            .collect(),
        None => return,
    };
    for skill in leaps {
        crate::step::util_server_steps::mark_skill_used(game, &pid, skill);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{Rules, SkillId, TurnMode};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;
    use std::collections::HashSet;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2016)
    }

    fn add_player_ag3(game: &mut Game, id: &str) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue::new(SkillId::Leap)],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
                    ..Default::default()
});
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.acting_player.player_id = Some(id.into());
    }

    #[test]
    fn not_jumping_returns_next_step() {
        let mut game = make_game();
        game.acting_player.jumping = false;
        let mut step = StepJump::new("fail".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn jumping_without_leap_skill_returns_next_step() {
        // canStillJump requires unused Leap skill — no skill → skip jump
        let mut game = make_game();
        game.team_home.players.push(Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], // no Leap skill
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
                    ..Default::default()
});
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.jumping = true;
        let mut step = StepJump::new("fail".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn jumping_success_clears_jumping_and_publishes_jumped() {
        let mut game = make_game();
        add_player_ag3(&mut game, "p1");
        game.acting_player.jumping = true;
        let mut step = StepJump::new("fail".into());
        step.roll = 4; // ag=3, min=3, 4>=3 → success
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!game.acting_player.jumping);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::Jumped(true))));
    }

    #[test]
    fn failure_goes_to_failure_label() {
        let mut game = make_game();
        add_player_ag3(&mut game, "p1");
        game.acting_player.jumping = true;
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        let mut step = StepJump::new("fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn failure_clears_jumping_flag() {
        let mut game = make_game();
        add_player_ag3(&mut game, "p1");
        game.acting_player.jumping = true;
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        let mut step = StepJump::new("fail".into());
        step.roll = 1;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.acting_player.jumping);
    }

    #[test]
    fn failure_with_trr_offers_reroll_prompt() {
        let mut game = make_game();
        add_player_ag3(&mut game, "p1");
        game.acting_player.jumping = true;
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        let mut step = StepJump::new("fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
        assert!(out.prompt.is_some());
    }

    #[test]
    fn decline_reroll_goes_to_failure_label() {
        let mut game = make_game();
        add_player_ag3(&mut game, "p1");
        game.acting_player.jumping = true;
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        let mut step = StepJump::new("fail".into());
        step.roll = 1;
        let _offer = step.start(&mut game, &mut GameRng::new(0));
        let out = step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn decline_reroll_does_not_clear_jumping_flag() {
        // Java (LeapBehaviour.handleExecuteStepHook): the reRollSource==null||!useReRoll(...)
        // branch (declined re-roll) publishes INJURY_TYPE + GOTO_LABEL but does NOT call
        // actingPlayer.setJumping(false) — only the leap() FAILURE branch (no more re-rolls
        // available) does that. `jumping` must remain true after a declined re-roll.
        let mut game = make_game();
        add_player_ag3(&mut game, "p1");
        game.acting_player.jumping = true;
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        let mut step = StepJump::new("fail".into());
        step.roll = 1;
        let _offer = step.start(&mut game, &mut GameRng::new(0));
        step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut GameRng::new(0));
        assert!(game.acting_player.jumping,
            "declined re-roll must not clear the jumping flag (Java LeapBehaviour does not call setJumping(false) here)");
    }

    #[test]
    fn accept_reroll_with_success_returns_next_step() {
        let mut game = make_game();
        add_player_ag3(&mut game, "p1");
        game.acting_player.jumping = true;
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        let mut step = StepJump::new("fail".into());
        step.roll = 1;
        let _offer = step.start(&mut game, &mut GameRng::new(0));
        // Corrected FROM the Java. This used to set `step.roll = 5` by hand and assert success,
        // which only worked because the OFFER zeroed the roll — the very thing that made a
        // DECLINED offer reach `handleFailure` with `roll == 0`. Java keeps `roll` across the
        // offer and draws a fresh die only once `useReRoll` has consumed one, so what this can
        // pin is that a CONSUMED re-roll replaces the die at all.
        step.roll = 5;
        let mut rng = GameRng::new(0);
        let out = step.handle_command(&Action::UseReRoll { use_reroll: true }, &mut game, &mut rng);
        assert_ne!(step.roll, 5, "an accepted re-roll draws a FRESH die");
        assert!(matches!(out.action, StepAction::NextStep | StepAction::GotoLabel),
            "and resolves the jump on it, either way");
    }

    /// The other half, and the one that was wrong: a DECLINED offer must reach the failure path
    /// with the roll that was actually made. `bb2016.LeapBehaviour` takes the
    /// `reRollSource == null || !useReRoll(...)` branch there, which does NOT clear `jumping` and
    /// does not touch `roll`; BB2020/BB2025's `handleFailure` reads `roll > 1` to decide whether
    /// the faller lands on the target square or is put back at `moveStart`.
    #[test]
    fn a_declined_reroll_keeps_the_roll_that_was_made() {
        let mut game = make_game();
        add_player_ag3(&mut game, "p1");
        game.acting_player.jumping = true;
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        let mut step = StepJump::new("fail".into());
        step.roll = 2;
        let offer = step.start(&mut game, &mut GameRng::new(0));
        assert!(offer.prompt.is_some(), "a failed 2 against 4+ is offered a team re-roll");
        assert_eq!(step.roll, 2, "the offer must NOT clear the roll");
        let out = step.handle_command(&Action::UseReRoll { use_reroll: false },
            &mut game, &mut GameRng::new(0));
        assert_eq!(step.roll, 2, "and declining must not draw a die either");
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn set_parameter_goto_label_on_failure_accepted() {
        let mut step = StepJump::new("old".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("new".into())));
        assert_eq!(step.goto_label_on_failure, "new");
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepJump::new("fail".into());
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn failure_publishes_steady_footing_context_drop_jump() {
        let mut game = make_game();
        add_player_ag3(&mut game, "p1");
        game.acting_player.jumping = true;
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        let mut step = StepJump::new("fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))));
    }

    #[test]
    fn success_adds_jump_roll_report() {
        // Java (LeapBehaviour): addReport(new ReportJumpRoll(...)) on success
        let mut game = make_game();
        add_player_ag3(&mut game, "p1");
        game.acting_player.jumping = true;
        let mut step = StepJump::new("fail".into());
        step.roll = 4; // ag=3, min=3, success
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            game.report_list.has_report(ReportId::JUMP_ROLL),
            "JUMP_ROLL report should be added on a successful jump"
        );
    }

    #[test]
    fn failure_adds_jump_roll_report() {
        // Java (LeapBehaviour): addReport is called regardless of success/failure
        let mut game = make_game();
        add_player_ag3(&mut game, "p1");
        game.acting_player.jumping = true;
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        let mut step = StepJump::new("fail".into());
        step.roll = 1; // failure
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            game.report_list.has_report(ReportId::JUMP_ROLL),
            "JUMP_ROLL report should be added on a failed jump"
        );
    }

    /// BACKLOG E12, written FROM `bb2016.LeapBehaviour`: both the SUCCESS and the FAILURE arm of
    /// `leap(step)` run `actingPlayer.markSkillUsed(skill)`. That mark is BB2016's whole
    /// once-per-activation rule, since `JumpMechanic.canStillJump` is
    /// `hasUnusedSkillWithProperty(actingPlayer, canLeap)`.
    #[test]
    fn a_bb2016_leap_is_spent_on_the_acting_player_win_or_lose() {
        for roll in [6, 1] {
            let mut game = make_game();
            add_player_ag3(&mut game, "p1");
            game.acting_player.jumping = true;
            game.home_playing = true;
            game.turn_data_home.rerolls = 0;
            let mut step = StepJump::new("fail".into());
            step.roll = roll;
            step.start(&mut game, &mut GameRng::new(0));
            assert!(game.acting_player.used_skills.contains(&SkillId::Leap),
                "roll {roll}: Leap must be marked used on the ACTING PLAYER");
            assert!(!ffb_mechanics::bb2016::jump_mechanic::JumpMechanic::new()
                .can_still_jump(&game, &game.acting_player.clone()),
                "roll {roll}: and that is what stops a second jump in the same activation");
        }
    }

    /// The BB2016 target itself, from `bb2016.AgilityMechanic.minimumRollJump`
    /// (`max(2, getAgilityRollBase(ag) + modifiers)`, the old `7 - AG` scale) plus
    /// `bb2016.VeryLongLegs`'s `JumpModifier("Very Long Legs", -1, REGULAR)`, which
    /// `LeapBehaviour`'s `modifierFactory.findModifiers(...)` picks up. An AG3 leaper needs a 4+
    /// bare and a 3+ with the mutation; the step used to compute BOTH from the BB2020 formula
    /// with no modifiers at all, which happened to give 3 for the AG3+mutation case and the
    /// wrong answer everywhere else.
    #[test]
    fn bb2016_jump_target_is_the_old_scale_and_honours_very_long_legs() {
        // Bare AG3 leaper: 7 - 3 = 4+, so a 3 FAILS.
        let mut game = make_game();
        add_player_ag3(&mut game, "p1");
        game.acting_player.jumping = true;
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        let mut step = StepJump::new("fail".into());
        step.roll = 3;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel,
            "AG3 with no mutation needs 4+, so a 3 falls down");

        // The same 3 with Very Long Legs is a 3+ and SUCCEEDS.
        let mut game = make_game();
        add_player_ag3(&mut game, "p1");
        game.team_home.player_mut("p1").unwrap()
            .starting_skills.push(SkillWithValue::new(SkillId::VeryLongLegs));
        game.acting_player.jumping = true;
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        let mut step = StepJump::new("fail".into());
        step.roll = 3;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep,
            "Very Long Legs is a flat -1 on the BB2016 jump");
    }
}
