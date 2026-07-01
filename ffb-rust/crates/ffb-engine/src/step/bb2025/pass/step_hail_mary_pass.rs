use ffb_model::enums::PassResult;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.pass.StepHailMaryPass.
///
/// Resolves a Hail Mary Pass skill roll.  Flow:
///  1. Roll d6 (or re-use cached `roll` if re-entering after re-roll).
///  2. Apply modifiers (PassMechanic — not yet translated).
///  3. Offer "use modifying skill" dialog (canAddStrengthToPass — not yet translated).
///  4. Offer Safe Pass dialog (dontDropFumbles — not yet translated).
///  5. Publish PassFumble.
///  6. ACCURATE/SAVED_FUMBLE → NEXT_STEP; FUMBLE/INACCURATE → `goto_label_on_failure`.
///
/// In Java, `executeStep()` delegates entirely to `getGameState().executeStepHooks(this, state)`
/// which calls the HailMaryPassHandler factory — that infrastructure is not yet translated.
/// The stub below performs the minimal d6 roll (threshold 4+) matching the Hail Mary Pass
/// skill rule, and routes accordingly.  The pass_skill_used / Safe Pass / re-roll paths
/// remain as TODO comments referencing the Java call sites.
///
/// Needs init param: `GotoLabelOnFailure`.
/// Publishes: `PassFumble`.
pub struct StepHailMaryPass {
    /// Java: state.goToLabelOnFailure (init param, mandatory)
    pub goto_label_on_failure: String,
    /// Java: state.result (PassResult)
    pub result: Option<PassResult>,
    /// Java: state.passSkillUsed — whether the pass skill re-roll was already consumed
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
        }
    }
}

impl Step for StepHailMaryPass {
    fn id(&self) -> StepId { StepId::HailMaryPass }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_USE_SKILL → canAddStrengthToPass → usingModifyingSkill = isSkillUsed()
        // Java: CLIENT_USE_SKILL → dontDropFumbles      → usingSafePass = isSkillUsed()
        // Java: otherwise → handleSkillCommand(cmd, state)  [pass skill re-roll]
        match action {
            Action::UseSkill { skill_id, use_skill } => {
                // Java: route by skill property: canAddStrengthToPass → usingModifyingSkill
                //                                dontDropFumbles       → usingSafePass
                if skill_id.properties().contains(&NamedProperties::CAN_ADD_STRENGTH_TO_PASS) {
                    self.using_modifying_skill = Some(*use_skill);
                } else if skill_id.properties().contains(&NamedProperties::DONT_DROP_FUMBLES) {
                    self.using_safe_pass = Some(*use_skill);
                } else {
                    // fallback: treat as modifying skill
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
    fn execute_step(&mut self, _game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: getGameState().executeStepHooks(this, state)
        //   → HailMaryPassHandler (not yet translated)
        //
        // Hail Mary Pass rule: roll 1d6; on 4+ the pass is "accurate" (no catcher-side roll).
        // On 1-3 the pass is inaccurate and scatters.
        // On a natural 1 before Safe Pass it is a fumble.
        // minimumRoll is always 4 for a Hail Mary Pass (no agility modifier applies).

        // Java: if (roll == 0) roll = diceRoller.rollSkill()
        // Hail Mary Pass fixed threshold is always 4 regardless of modifiers.
        if self.minimum_roll == 0 {
            self.minimum_roll = 4;
        }
        if self.roll == 0 {
            self.roll = rng.d6();
        }

        // DEFERRED: if usingModifyingSkill == null && modifyingSkill exists → showDialog → CONTINUE
        // DEFERRED: PassMechanic.evaluatePass with statBasedModifier when usingModifyingSkill==true

        let is_fumble = self.roll == 1;
        let is_accurate = self.roll >= self.minimum_roll;

        // DEFERRED: Safe Pass dialog: if result == SAVED_FUMBLE && usingSafePass == null → showDialog
        // DEFERRED: if !usingSafePass → result = FUMBLE

        // Java: publishParameter(PASS_FUMBLE, PassResult.FUMBLE == state.result)
        // Java: if ACCURATE/SAVED_FUMBLE → NEXT_STEP
        // Java: else (FUMBLE / INACCURATE) → GOTO_LABEL(goToLabelOnFailure)

        let pass_fumble = is_fumble && self.using_safe_pass != Some(true);
        let label = self.goto_label_on_failure.clone();

        if is_accurate {
            // ACCURATE
            StepOutcome::next()
                .publish(StepParameter::PassFumble(false))
        } else if pass_fumble {
            // FUMBLE
            StepOutcome::goto(&label)
                .publish(StepParameter::PassFumble(true))
        } else {
            // INACCURATE (or SAVED_FUMBLE treated as inaccurate path)
            StepOutcome::goto(&label)
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
    fn roll_4_accurate_next_step() {
        let mut game = make_game();
        let mut step = StepHailMaryPass::new("fail".into());
        step.roll = 4;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        let fumble = out.published.iter().find(|p| matches!(p, StepParameter::PassFumble(false)));
        assert!(fumble.is_some());
    }

    #[test]
    fn roll_3_inaccurate_goto_failure() {
        let mut game = make_game();
        let mut step = StepHailMaryPass::new("fail".into());
        step.roll = 3;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn roll_1_fumble_goto_failure_publishes_pass_fumble_true() {
        let mut game = make_game();
        let mut step = StepHailMaryPass::new("fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        let fumble = out.published.iter().find(|p| matches!(p, StepParameter::PassFumble(true)));
        assert!(fumble.is_some(), "expected PassFumble(true) for natural 1");
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
        step.roll = 6; // cached successful roll
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.roll, 6); // unchanged
        assert_eq!(out.action, StepAction::NextStep);
    }
}
