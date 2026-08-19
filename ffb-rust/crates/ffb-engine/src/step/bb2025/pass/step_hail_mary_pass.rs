use ffb_model::enums::{PassOutcome, PassingDistance, ReRollSource, SkillId};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use ffb_model::report::mixed::report_pass_roll::ReportPassRoll;
use ffb_mechanics::bb2025::pass_mechanic::PassMechanic as Bb2025PassMechanic;
use ffb_mechanics::modifiers::pass_context::PassContext;
use ffb_mechanics::modifiers::pass_modifier_factory::PassModifierFactory;
use ffb_mechanics::pass_mechanic::PassMechanic as PassMechanicTrait;
use crate::action::Action;
use crate::model::step_modifier::RerollHookState;
use crate::skill_behaviour::dispatch;
use crate::step::framework::{Step, StepCommandStatus, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

/// Java: `ReRolledActions.PASS` — the re-rolled-action key this step registers itself under.
const REROLLED_ACTION_PASS: &str = "PASS";

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.pass.StepHailMaryPass.
///
/// Resolves a Hail Mary Pass skill roll.  Flow:
///  1. Roll d6 (or re-use cached `roll` if re-entering after re-roll).
///  2. Apply modifiers (PassMechanic -- headless, Phase ZT).
///  3. Offer "use modifying skill" dialog (canAddStrengthToPass -- headless, Phase ZT).
///  4. Offer Safe Pass dialog (dontDropFumbles -- headless, Phase ZT).
///  5. Java line 149: convert ACCURATE -> INACCURATE (Hail Mary always deviates).
///  6. Publish PassFumble.
///  7. FUMBLE/SAVED_FUMBLE -> GOTO_LABEL; INACCURATE -> NEXT_STEP.
///
/// Needs init param: `GotoLabelOnFailure`.
/// Publishes: `PassFumble`.
pub struct StepHailMaryPass {
    /// Java: state.goToLabelOnFailure (init param, mandatory)
    pub goto_label_on_failure: String,
    /// Java: state.result (PassOutcome)
    pub result: Option<PassOutcome>,
    /// Java: state.passSkillUsed -- whether the pass skill re-roll was already consumed
    pub pass_skill_used: bool,
    /// Java: state.usingModifyingSkill (Boolean tristate)
    pub using_modifying_skill: Option<bool>,
    /// Java: state.usingSafePass (Boolean tristate)
    pub using_safe_pass: Option<bool>,
    /// Java: state.minimumRoll
    pub minimum_roll: i32,
    /// Java: state.roll
    pub roll: i32,
    // AbstractStepWithReRoll fields
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
    /// True when the fumble was saved by Safe Pass (SAVED_FUMBLE in Java mechanics PassOutcome).
    pub saved_fumble: bool,
}

impl StepHailMaryPass {
    pub fn new(goto_label_on_failure: String) -> Self {
        Self {
            goto_label_on_failure,
            result: None,
            pass_skill_used: false,
            using_modifying_skill: None,
            using_safe_pass: None,
            minimum_roll: 0,
            roll: 0,
            re_rolled_action: None,
            re_roll_source: None,
            saved_fumble: false,
        }
    }
}

impl Step for StepHailMaryPass {
    fn id(&self) -> StepId { StepId::HailMaryPass }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_USE_SKILL -> canAddStrengthToPass -> usingModifyingSkill = isSkillUsed()
        // Java: CLIENT_USE_SKILL -> dontDropFumbles      -> usingSafePass = isSkillUsed()
        // Java: otherwise -> handleSkillCommand(cmd, state)  [pass skill re-roll, e.g. TheBallista]
        match action {
            Action::UseSkill { skill_id, use_skill } if *skill_id == SkillId::TheBallista => {
                // Java: AbstractStep.handleSkillCommand -> TheBallistaBehaviour's StepHailMaryPass
                // modifier presets reRolledAction=PASS/reRollSource before the step re-executes;
                // execute_step's retry-consumption branch resets `roll` and re-rolls (Phase AAZ).
                let mut hook_state = RerollHookState {
                    re_rolled_action: self.re_rolled_action.clone(),
                    re_roll_source: self.re_roll_source.clone(),
                    kicked: false,
                };
                let status = dispatch::handle_skill_command(
                    game, StepId::HailMaryPass, &mut hook_state, *skill_id, *use_skill,
                );
                if status == StepCommandStatus::ExecuteStep {
                    self.re_rolled_action = hook_state.re_rolled_action;
                    self.re_roll_source = hook_state.re_roll_source;
                }
            }
            Action::UseSkill { skill_id, use_skill } => {
                if skill_id.properties().contains(&NamedProperties::CAN_ADD_STRENGTH_TO_PASS) {
                    self.using_modifying_skill = Some(*use_skill);
                } else if skill_id.properties().contains(&NamedProperties::DONT_DROP_FUMBLES) {
                    self.using_safe_pass = Some(*use_skill);
                } else {
                    self.using_modifying_skill = Some(*use_skill);
                }
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            StepParameter::UsingModifyingSkill(v) => { self.using_modifying_skill = Some(*v); true }
            StepParameter::PassResultParam(v) => { self.result = Some(*v); true }
            _ => false,
        }
    }
}

impl StepHailMaryPass {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: PassBehaviour.handleExecuteStepHook -- StepHailMaryPass variant.
        //
        // Hail Mary Pass rule (Phase AAV): a real Passing Ability Test treated as a Long Bomb,
        // using the thrower's real Passing stat + modifiers (not a fixed "always 4" stub) — see
        // step_hail_mary_pass.rs (bb2020) for the same fix. Note this only affects the *reported*
        // minimum roll here: routing below is fumble (roll==1) vs not, independent of
        // minimum_roll, and ACCURATE/INACCURATE both route identically (line 149's conversion),
        // so this doesn't change gameplay outcomes in bb2025 — only report/log fidelity.
        // Java line 149: raw ACCURATE result -> INACCURATE (Hail Mary always deviates).
        // Routing: FUMBLE / SAVED_FUMBLE -> GOTO_LABEL; INACCURATE -> NEXT_STEP.
        if self.minimum_roll == 0 {
            let thrower = game.thrower_id.clone().and_then(|id| game.player(&id)).cloned();
            self.minimum_roll = thrower.as_ref().map(|t| {
                let factory = PassModifierFactory::for_rules(game.rules);
                let ctx = PassContext::new(game, t, PassingDistance::LongBomb, false);
                let mut modifiers: Vec<ffb_mechanics::modifiers::PassModifier> = factory.find_modifiers(&ctx)
                    .into_iter()
                    .map(|m| ffb_mechanics::modifiers::PassModifier::with_report(
                        m.get_name(), m.get_report_string(), m.get_modifier(), m.get_type(),
                    ))
                    .collect();
                modifiers.extend(factory.find_card_modifiers(&ctx));
                Bb2025PassMechanic.minimum_roll_simple(t, PassingDistance::LongBomb, &modifiers)
            }).flatten().unwrap_or(4);
        }
        // Java: AbstractStepWithReRoll retry contract — if a reroll was granted for this step's
        // own action key (e.g. TheBallista's handleCommandHook), consuming it successfully forces
        // a genuine second roll instead of reusing the cached one.
        if self.re_rolled_action.as_deref() == Some(REROLLED_ACTION_PASS) {
            if let Some(source_name) = self.re_roll_source.clone() {
                let thrower_id_for_reroll = game.thrower_id.clone().unwrap_or_default();
                if use_reroll(game, &ReRollSource::new(source_name), &thrower_id_for_reroll) {
                    self.roll = 0;
                }
            }
        }

        if self.roll == 0 {
            self.roll = rng.d6();
        }

        // headless: showUseModifyingSkillDialog (canAddStrengthToPass) -- Phase ZT
        // headless: Safe Pass dialog (dontDropFumbles) -- Phase ZT
        // When wired, using_safe_pass == Some(true) marks FUMBLE as SAVED_FUMBLE.

        // Java PassBehaviour(bb2025) :146-149: the roll is evaluated through the REAL pass
        // mechanic against LONG_BOMB + the live modifiers — a roll whose modified result is <= 1
        // is a FUMBLE even when the die is not a 1 (elf bb2025 seed 8 i=243: roll 5 under Very
        // Sunny fumbled in Java; the old `roll == 1` shortcut called it INACCURATE and
        // triple-scattered instead of bouncing at the thrower). ACCURATE converts to INACCURATE
        // (a Hail Mary always deviates); the minimum reported/offered to the reroll dialog is
        // Java :148's explicit `max(2, 2 + distance.mod2020 + sum(mods))`.
        let (raw_result, java_min) = if game.rules == ffb_model::enums::Rules::Bb2016 {
            // Java bb2016 PassBehaviour's StepHailMaryPass hook bypasses the pass mechanic
            // entirely: `state.result = (roll == 1) ? FUMBLE : INACCURATE` — no distance or
            // tacklezone modifiers, no SAVED_FUMBLE, and the reroll dialogs are offered with a
            // fixed minimum of 2 (DialogSkillUseParameter(..., 2) / askForReRollIfAvailable(...,
            // 2, false)). Routing bb2016 through the bb2025-style mechanic evaluate made a
            // natural 3 a "modified fumble" (3 + LongBomb -2 = 1) and burned a team reroll Java
            // never rolled (dwarf bb2016 seed 43 i=2: one extra d6 shifted all three scatter d8s).
            let raw = if self.roll == 1 {
                ffb_mechanics::pass_result::PassResult::FUMBLE
            } else {
                ffb_mechanics::pass_result::PassResult::INACCURATE
            };
            (raw, 2)
        } else {
            let thrower = game.thrower_id.clone().and_then(|id| game.player(&id)).cloned();
            match thrower {
                Some(t) => {
                    let factory = PassModifierFactory::for_rules(game.rules);
                    let ctx = PassContext::new(game, &t, PassingDistance::LongBomb, false);
                    let mut modifiers: Vec<ffb_mechanics::modifiers::PassModifier> = factory.find_modifiers(&ctx)
                        .into_iter()
                        .map(|m| ffb_mechanics::modifiers::PassModifier::with_report(
                            m.get_name(), m.get_report_string(), m.get_modifier(), m.get_type(),
                        ))
                        .collect();
                    modifiers.extend(factory.find_card_modifiers(&ctx));
                    let bomb_action = game.thrower_action
                        == Some(ffb_model::enums::PlayerAction::HailMaryBomb);
                    let mechanic = crate::mechanic::pass_mechanic_for(game.rules);
                    let raw = mechanic.evaluate_pass_simple(
                        &t, self.roll, PassingDistance::LongBomb, &modifiers, bomb_action);
                    let min = std::cmp::max(2, 2 + PassingDistance::LongBomb.modifier_2020()
                        + modifiers.iter().map(|m| m.get_modifier()).sum::<i32>());
                    (raw, min)
                }
                None => (ffb_mechanics::pass_result::PassResult::FUMBLE, self.minimum_roll),
            }
        };
        self.minimum_roll = java_min;
        let is_fumble = matches!(raw_result,
            ffb_mechanics::pass_result::PassResult::FUMBLE
            | ffb_mechanics::pass_result::PassResult::SAVED_FUMBLE);
        self.saved_fumble = raw_result == ffb_mechanics::pass_result::PassResult::SAVED_FUMBLE
            || (is_fumble && self.using_safe_pass == Some(true));

        // Java PassBehaviour :159-176 — fumble reroll cascade: a skill reroll source for PASS
        // (the Pass skill dialog, auto-used by both harnesses) FIRST, then the team reroll offer.
        if is_fumble && !self.saved_fumble && self.re_rolled_action.is_none() {
            if let Some(source) = crate::step::abstract_step_with_re_roll::find_skill_reroll_source(game, REROLLED_ACTION_PASS) {
                self.re_rolled_action = Some(REROLLED_ACTION_PASS.into());
                self.re_roll_source = Some(source.name.clone());
                return self.execute_step(game, rng);
            }
            if let Some(prompt) = ask_for_reroll_if_available(game, REROLLED_ACTION_PASS, self.minimum_roll, true) {
                self.re_rolled_action = Some(REROLLED_ACTION_PASS.into());
                self.re_roll_source = Some("TRR".into());
                return StepOutcome::cont().with_prompt(prompt);
            }
        }

        // Java line 149: result = (raw == ACCURATE) ? INACCURATE : raw
        // Both ACCURATE (4+) and raw INACCURATE (2-3) become INACCURATE in state.
        // FUMBLE stays FUMBLE; SAVED_FUMBLE stays SAVED_FUMBLE.
        self.result = Some(if is_fumble {
            PassOutcome::Fumble  // Java: FUMBLE or SAVED_FUMBLE; model has no SAVED_FUMBLE variant
        } else {
            PassOutcome::Inaccurate  // Java: INACCURATE (includes converted ACCURATE)
        });

        let re_rolled = self.re_rolled_action.is_some() && self.re_roll_source.is_some();
        game.report_list.add(ReportPassRoll::new(
            game.thrower_id.clone(),
            self.roll >= self.minimum_roll,
            self.roll,
            self.minimum_roll,
            re_rolled,
            vec![],
            None,      // passing_distance: N/A for hail mary
            false,     // bomb
            None,      // result name
            true,      // hail_mary_pass
            None,      // stat_based_roll_modifier
        ));

        // Java PassBehaviour :188-217 — the ball placement follows the result:
        //   SAVED_FUMBLE → ball at the THROWER, not moving, GOTO;
        //   FUMBLE       → ball at the THROWER (moving), publish SCATTER_BALL mode, GOTO;
        //   INACCURATE   → ball at the PASS COORDINATE (MissedPass scatters from there), NEXT.
        // Note the ball is relocated to the thrower even when the thrower did not carry it —
        // the turn-start snapshot can offer a Hail Mary to a player who has since lost the ball,
        // and Java still drops THE ball at the thrower (elf bb2025 seed 83 i=75).
        let thrower_coord = game.thrower_id.as_deref()
            .and_then(|id| game.field_model.player_coordinate(id));
        let is_bomb = game.thrower_action == Some(ffb_model::enums::PlayerAction::HailMaryBomb);
        let label = self.goto_label_on_failure.clone();
        if is_fumble && self.saved_fumble {
            if is_bomb {
                game.field_model.bomb_coordinate = None;
                game.field_model.bomb_moving = false;
            } else {
                game.field_model.ball_coordinate = thrower_coord;
                game.field_model.ball_moving = false;
            }
            StepOutcome::goto(&label)
                .publish(StepParameter::PassFumble(false))
        } else if is_fumble {
            if is_bomb {
                game.field_model.bomb_coordinate = thrower_coord;
                StepOutcome::goto(&label)
                    .publish(StepParameter::PassFumble(true))
            } else {
                game.field_model.ball_coordinate = thrower_coord;
                StepOutcome::goto(&label)
                    .publish(StepParameter::PassFumble(true))
                    .publish(StepParameter::CatchScatterThrowInMode(
                        ffb_model::model::catch_scatter_throw_in_mode::CatchScatterThrowInMode::ScatterBall))
            }
        } else {
            if is_bomb {
                game.field_model.bomb_coordinate = thrower_coord;
                game.field_model.bomb_moving = false;
            } else {
                game.field_model.ball_coordinate = game.pass_coordinate;
            }
            StepOutcome::next()
                .publish(StepParameter::PassFumble(false))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    fn make_game_with_thrower(passing: i32) -> Game {
        use ffb_model::enums::{PlayerGender, PlayerType};
        use ffb_model::model::player::Player;
        let mut game = make_game();
        game.team_home.players.push(Player {
            id: "thrower".into(), name: "thrower".into(), nr: 1, position_id: "thrower".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.thrower_id = Some("thrower".into());
        game
    }

    #[test]
    fn bb2016_hmp_fumbles_only_on_a_natural_one() {
        // Java bb2016 PassBehaviour hook: `result = (roll == 1) ? FUMBLE : INACCURATE` — no
        // mechanic evaluation. The mechanic path called a natural 3 a modified fumble
        // (3 + LongBomb -2 = 1) and burned a reroll Java never rolled (dwarf bb2016 seed 43).
        let mut game = make_game_with_thrower(4);
        game.rules = Rules::Bb2016;
        game.home_playing = true;
        let mut step = StepHailMaryPass::new(String::new());
        // Pre-set the roll to the dwarf seed-43 value (the step skips rolling when roll != 0):
        // 3 at LongBomb is a modified fumble under the mechanic but INACCURATE per Java bb2016.
        step.roll = 3;
        let mut rng = GameRng::new(7);
        step.execute_step(&mut game, &mut rng);
        assert_eq!(step.result, Some(PassOutcome::Inaccurate),
            "bb2016 HMP roll {} must be INACCURATE, not a modified fumble", step.roll);
        assert!(step.re_rolled_action.is_none(),
            "no reroll cascade may trigger for a non-1 bb2016 HMP roll");
        assert_eq!(step.minimum_roll, 2, "Java offers bb2016 HMP rerolls with minimum 2");
    }

    #[test]
    fn roll_5_or_higher_routes_to_next_step_for_pa2_thrower() {
        // Java PassBehaviour :146-149 evaluates through the mechanic: PA2+ thrower, Long Bomb
        // (modifier_2020 = 3): roll r → modified r-3; ACCURATE (→Inaccurate) needs r-3 >= 2 or
        // a natural 6. Rolls 5 and 6 route to NextStep.
        for roll in [5, 6] {
            let mut game = make_game_with_thrower(2);
            let mut step = StepHailMaryPass::new("fail".into());
            step.roll = roll;
            let out = step.start(&mut game, &mut GameRng::new(0));
            assert_eq!(out.action, StepAction::NextStep, "roll {} should route to NextStep", roll);
            assert!(out.published.iter().any(|p| matches!(p, StepParameter::PassFumble(false))));
        }
    }

    #[test]
    fn modified_result_of_one_or_less_is_a_fumble() {
        // The elf bb2025 seed 8 lesson: a die that is not a 1 still FUMBLES when the modified
        // result is <= 1 (Java bb2025 PassMechanic.evaluatePass). PA2 Long Bomb: rolls 2-4
        // modify to <= 1 → FUMBLE → GOTO_LABEL.
        for roll in [2, 3, 4] {
            let mut game = make_game_with_thrower(2);
            let mut step = StepHailMaryPass::new("fail".into());
            step.roll = roll;
            let out = step.start(&mut game, &mut GameRng::new(0));
            assert_eq!(out.action, StepAction::GotoLabel, "roll {} (modified <= 1) must FUMBLE", roll);
        }
    }

    #[test]
    fn roll_1_fumble_goto_failure_publishes_pass_fumble_true() {
        let mut game = make_game_with_thrower(2);
        let mut step = StepHailMaryPass::new("fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::PassFumble(true))),
            "expected PassFumble(true) for natural 1");
    }

    #[test]
    fn roll_1_with_safe_pass_is_saved_fumble_goto_label() {
        // SAVED_FUMBLE -> GOTO_LABEL (not NEXT_STEP), PassFumble(false)
        let mut game = make_game_with_thrower(2);
        let mut step = StepHailMaryPass::new("fail".into());
        step.roll = 1;
        step.using_safe_pass = Some(true);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::PassFumble(false))),
            "SAVED_FUMBLE should publish PassFumble(false)");
        assert!(step.saved_fumble, "saved_fumble flag should be set");
    }

    #[test]
    fn accurate_roll_result_stored_as_inaccurate() {
        // Java line 149: ACCURATE -> INACCURATE conversion
        let mut game = make_game_with_thrower(2);
        let mut step = StepHailMaryPass::new("fail".into());
        step.roll = 5;
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.result, Some(PassOutcome::Inaccurate),
            "ACCURATE roll should be stored as Inaccurate per Java line 149");
    }

    /// The elf seed 83 shape: a fumbled HMP by a Pass-skill thrower must consume the SKILL
    /// reroll (Java PassBehaviour :163-172 shows the Pass dialog; both harnesses auto-use) and
    /// roll a SECOND die.
    #[test]
    fn fumbled_hmp_uses_the_pass_skill_reroll_and_rolls_again() {
        use ffb_model::model::skill_def::SkillWithValue;
        let mut game = make_game_with_thrower(2);
        if let Some(p) = game.team_home.players.iter_mut().find(|p| p.id == "thrower") {
            p.starting_skills = vec![SkillWithValue::new(SkillId::Pass)];
        }
        game.turn_mode = ffb_model::enums::TurnMode::Regular;
        game.acting_player.player_id = Some("thrower".into());
        game.acting_player.player_action = Some(ffb_model::enums::PlayerAction::HailMaryPass);
        let mut step = StepHailMaryPass::new("fail".into());
        step.roll = 2; // modified 2-3 <= 1 → FUMBLE for the PA2 thrower
        let _ = step.start(&mut game, &mut GameRng::new(7));
        assert_eq!(step.re_rolled_action.as_deref(), Some("PASS"),
            "the fumble must trigger the reroll cascade");
        assert_ne!(step.roll, 2, "the skill reroll must produce a fresh roll");
    }

    #[test]
    fn set_parameter_goto_label_accepted() {
        let mut step = StepHailMaryPass::new("old".into());
        step.set_parameter(&StepParameter::GotoLabelOnFailure("new".into()));
        assert_eq!(step.goto_label_on_failure.as_str(), "new");
    }

    #[test]
    fn roll_cached_not_re_rolled() {
        let mut game = make_game_with_thrower(2);
        let mut step = StepHailMaryPass::new("fail".into());
        step.roll = 6;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.roll, 6);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn minimum_roll_set_to_4_on_first_execute() {
        let mut game = make_game();
        let mut step = StepHailMaryPass::new("fail".into());
        step.roll = 4;
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.minimum_roll, 4);
    }

    #[test]
    fn minimum_roll_is_computed_from_thrower_passing_stat_as_a_long_bomb() {
        use ffb_model::enums::{PlayerGender, PlayerType};
        use ffb_model::model::player::Player;
        // Bb2025PassMechanic::minimum_roll = passing + LongBomb's modifier_2020 (+3), floor 2.
        let mut game = make_game();
        game.team_home.players.push(Player {
            id: "thrower".into(), name: "thrower".into(), nr: 1, position_id: "thrower".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 2, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.thrower_id = Some("thrower".into());
        let mut step = StepHailMaryPass::new("fail".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.minimum_roll, 5);
    }

    #[test]
    fn accurate_roll_emits_pass_roll_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game_with_thrower(2);
        let mut step = StepHailMaryPass::new("fail".into());
        step.roll = 5;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::PASS_ROLL));
    }

    #[test]
    fn fumble_roll_emits_pass_roll_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game_with_thrower(2);
        let mut step = StepHailMaryPass::new("fail".into());
        step.roll = 1;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::PASS_ROLL));
    }

    #[test]
    fn pass_fumble_false_for_inaccurate_roll() {
        let mut game = make_game_with_thrower(2);
        let mut step = StepHailMaryPass::new("fail".into());
        step.roll = 5;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::PassFumble(false))));
    }

    #[test]
    fn the_ballista_use_skill_true_sets_pass_rerolled_action_and_source() {
        let mut game = make_game();
        let mut step = StepHailMaryPass::new("fail".into());
        step.roll = 3; // avoid re-executing into an unrelated branch
        step.handle_command(
            &Action::UseSkill { skill_id: SkillId::TheBallista, use_skill: true },
            &mut game, &mut GameRng::new(0),
        );
        assert_eq!(step.re_rolled_action.as_deref(), Some("PASS"));
        assert_eq!(step.re_roll_source.as_deref(), Some("TheBallista"));
    }

    #[test]
    fn the_ballista_use_skill_false_clears_source() {
        let mut game = make_game();
        let mut step = StepHailMaryPass::new("fail".into());
        step.roll = 3;
        step.handle_command(
            &Action::UseSkill { skill_id: SkillId::TheBallista, use_skill: false },
            &mut game, &mut GameRng::new(0),
        );
        assert_eq!(step.re_rolled_action.as_deref(), Some("PASS"));
        assert!(step.re_roll_source.is_none());
    }

    #[test]
    fn the_ballista_reroll_actually_consumes_the_skill_and_forces_a_second_roll() {
        // Phase AAZ: previously, presetting re_rolled_action/re_roll_source alone never
        // triggered a real second roll — execute_step never checked those fields at all.
        use ffb_model::model::skill_def::SkillWithValue;
        let mut game = make_game_with_thrower(4);
        game.team_home.player_mut("thrower").unwrap().extra_skills
            .push(SkillWithValue::new(SkillId::TheBallista));

        let mut step = StepHailMaryPass::new("fail".into());
        step.roll = 1; // stale/fumble roll from before the reroll was granted
        step.re_rolled_action = Some("PASS".into());
        step.re_roll_source = Some("TheBallista".into());
        step.start(&mut game, &mut GameRng::new(0));

        assert!(
            game.player("thrower").unwrap().used_skills.contains(&SkillId::TheBallista),
            "TheBallista should be marked used once its re-roll is actually consumed"
        );
    }

    #[test]
    fn fumbled_roll_offers_a_team_reroll_when_available() {
        let mut game = make_game_with_thrower(2);
        game.home_playing = true;
        game.turn_data_mut().rerolls = 1;

        let mut step = StepHailMaryPass::new("fail".into());
        step.roll = 1; // fumble

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue, "a reroll should be offered, not an immediate miss");
        assert_eq!(step.re_rolled_action.as_deref(), Some("PASS"));
        assert_eq!(step.re_roll_source.as_deref(), Some("TRR"));
        assert_eq!(game.turn_data().rerolls, 1, "the reroll must only be offered, not yet consumed");
    }

    #[test]
    fn accepting_the_offered_reroll_consumes_it_and_forces_a_second_roll() {
        let mut game = make_game_with_thrower(2);
        game.home_playing = true;
        game.turn_data_mut().rerolls = 1;

        let mut step = StepHailMaryPass::new("fail".into());
        step.roll = 1;
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_data().rerolls, 1, "sanity: not yet consumed");

        step.handle_command(&Action::UseReRoll { use_reroll: true }, &mut game, &mut GameRng::new(1));
        assert_eq!(game.turn_data().rerolls, 0, "accepting the offer should consume the team re-roll");
        assert!(game.turn_data().reroll_used);
    }

    #[test]
    fn no_reroll_available_falls_through_to_goto_failure_unchanged() {
        // Regression guard: without an available re-roll, behavior matches the pre-AAZ stub.
        let mut game = make_game_with_thrower(2);
        game.home_playing = true;
        game.turn_data_mut().rerolls = 0;

        let mut step = StepHailMaryPass::new("fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(step.re_rolled_action.is_none());
    }

    #[test]
    fn modifying_skill_use_unaffected_by_ballista_wiring() {
        let mut game = make_game();
        let mut step = StepHailMaryPass::new("fail".into());
        step.roll = 3;
        step.handle_command(
            &Action::UseSkill { skill_id: SkillId::Dauntless, use_skill: true },
            &mut game, &mut GameRng::new(0),
        );
        assert_eq!(step.using_modifying_skill, Some(true));
        assert!(step.re_rolled_action.is_none());
    }
}
