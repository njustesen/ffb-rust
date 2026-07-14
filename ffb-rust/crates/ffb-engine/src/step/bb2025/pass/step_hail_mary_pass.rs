use ffb_model::enums::{PassResult, SkillId};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use ffb_model::report::mixed::report_pass_roll::ReportPassRoll;
use crate::action::Action;
use crate::model::step_modifier::RerollHookState;
use crate::skill_behaviour::dispatch;
use crate::step::framework::{Step, StepCommandStatus, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

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
    /// Java: state.result (PassResult)
    pub result: Option<PassResult>,
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
    /// True when the fumble was saved by Safe Pass (SAVED_FUMBLE in Java mechanics PassResult).
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
                // modifier presets reRolledAction=PASS/reRollSource before the step re-executes.
                // Known gap (documented, not silently dropped): unlike StepThrowTeamMate, this
                // step does not yet implement a full re-roll-retry cycle (it never resets `roll`
                // or offers a re-roll prompt), so presetting these fields alone does not yet
                // trigger an actual second roll — see SESSION.md.
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
        // Hail Mary Pass rule: minimum roll is always 4 (no agility modifier applies).
        // Java line 149: raw ACCURATE result -> INACCURATE (Hail Mary always deviates).
        // Routing: FUMBLE / SAVED_FUMBLE -> GOTO_LABEL; INACCURATE -> NEXT_STEP.
        if self.minimum_roll == 0 {
            self.minimum_roll = 4;
        }
        if self.roll == 0 {
            self.roll = rng.d6();
        }

        // headless: showUseModifyingSkillDialog (canAddStrengthToPass) -- Phase ZT
        // headless: Safe Pass dialog (dontDropFumbles) -- Phase ZT
        // When wired, using_safe_pass == Some(true) marks FUMBLE as SAVED_FUMBLE.

        let is_fumble = self.roll == 1;
        self.saved_fumble = is_fumble && self.using_safe_pass == Some(true);

        // Java line 149: result = (raw == ACCURATE) ? INACCURATE : raw
        // Both ACCURATE (4+) and raw INACCURATE (2-3) become INACCURATE in state.
        // FUMBLE stays FUMBLE; SAVED_FUMBLE stays SAVED_FUMBLE.
        self.result = Some(if is_fumble {
            PassResult::Fumble  // Java: FUMBLE or SAVED_FUMBLE; model has no SAVED_FUMBLE variant
        } else {
            PassResult::Inaccurate  // Java: INACCURATE (includes converted ACCURATE)
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

        let label = self.goto_label_on_failure.clone();
        if is_fumble {
            // FUMBLE or SAVED_FUMBLE -> GOTO_LABEL
            StepOutcome::goto(&label)
                .publish(StepParameter::PassFumble(!self.saved_fumble))
        } else {
            // INACCURATE (roll 2-3) or ACCURATE converted to INACCURATE (roll 4+) -> NEXT_STEP
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

    #[test]
    fn roll_4_or_higher_routes_to_next_step() {
        for roll in [4, 5, 6] {
            let mut game = make_game();
            let mut step = StepHailMaryPass::new("fail".into());
            step.roll = roll;
            let out = step.start(&mut game, &mut GameRng::new(0));
            assert_eq!(out.action, StepAction::NextStep, "roll {} should route to NextStep", roll);
            assert!(out.published.iter().any(|p| matches!(p, StepParameter::PassFumble(false))));
        }
    }

    #[test]
    fn roll_2_or_3_inaccurate_routes_to_next_step() {
        // Java routing: INACCURATE -> NEXT_STEP (not GOTO_LABEL)
        for roll in [2, 3] {
            let mut game = make_game();
            let mut step = StepHailMaryPass::new("fail".into());
            step.roll = roll;
            let out = step.start(&mut game, &mut GameRng::new(0));
            assert_eq!(out.action, StepAction::NextStep, "roll {} (INACCURATE) should route to NextStep", roll);
        }
    }

    #[test]
    fn roll_1_fumble_goto_failure_publishes_pass_fumble_true() {
        let mut game = make_game();
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
        let mut game = make_game();
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
        let mut game = make_game();
        let mut step = StepHailMaryPass::new("fail".into());
        step.roll = 5;
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.result, Some(PassResult::Inaccurate),
            "ACCURATE roll should be stored as Inaccurate per Java line 149");
    }

    #[test]
    fn set_parameter_goto_label_accepted() {
        let mut step = StepHailMaryPass::new("old".into());
        step.set_parameter(&StepParameter::GotoLabelOnFailure("new".into()));
        assert_eq!(step.goto_label_on_failure.as_str(), "new");
    }

    #[test]
    fn roll_cached_not_re_rolled() {
        let mut game = make_game();
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
    fn accurate_roll_emits_pass_roll_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        let mut step = StepHailMaryPass::new("fail".into());
        step.roll = 5;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::PASS_ROLL));
    }

    #[test]
    fn fumble_roll_emits_pass_roll_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        let mut step = StepHailMaryPass::new("fail".into());
        step.roll = 1;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::PASS_ROLL));
    }

    #[test]
    fn pass_fumble_false_for_inaccurate_roll() {
        let mut game = make_game();
        let mut step = StepHailMaryPass::new("fail".into());
        step.roll = 3;
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
