/// 1:1 translation of com.fumbbl.ffb.server.skillbehaviour.bb2025.PassBehaviour.
///
/// PassBehaviour extends AbstractPassBehaviour<Pass> and registers one StepModifier
/// targeting StepHailMaryPass. Java's real hook handles:
///   - Pass skill re-roll offer on a fumble
///   - canAddStrengthToPass modifying-skill dialog / strength bonus
///   - dontDropFumbles (Safe Pass) dialog
///   - Full pass roll resolution with PassMechanic + PassModifierFactory
///
/// Phase AAV: `step_hail_mary_pass.rs` (bb2020 + bb2025) now computes its minimum roll and
/// evaluates the pass via the real `PassMechanic`/`PassModifierFactory`/`PassContext` directly,
/// matching the already-live pattern in `step_pass.rs` (the regular Pass action, which was
/// already fully wired to this same infra before this phase). This `PassStepModifier` itself
/// remains a documented no-op (`handle_execute_step` always returns `false`) since
/// `StepHailMaryPass::execute_step` implements the real logic inline rather than by calling
/// `dispatch::execute_step_hooks` — the same "registered but the real logic lives directly in
/// the step file" pattern already established for most other skills in this codebase.
use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::StepModifierTrait;
use crate::step::framework::StepId;
use crate::skill_behaviour::registry::SkillRegistry;
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;

/// Pass: player may re-roll a failed pass roll once per action.
pub struct PassBehaviour;

impl PassBehaviour {
    pub fn new() -> Self { Self }

    /// Java: PassBehaviour constructor calls registerModifier(new StepModifier<StepHailMaryPass, ...>).
    /// Rust: creates a SkillBehaviourContainer with PassStepModifier and registers it.
    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(PassStepModifier));
        registry.register(SkillId::Pass, sb);
    }
}

impl Default for PassBehaviour {
    fn default() -> Self { Self::new() }
}

// ---- Hook state --------------------------------------------------------------

/// Java: StepHailMaryPass.StepState (inner class).
/// Fields used by PassBehaviour.handleExecuteStepHook:
///   - goToLabelOnFailure  (init param)
///   - result              (PassResult name)
///   - passSkillUsed       (boolean -- Pass skill re-roll already consumed)
///   - usingModifyingSkill (Boolean tristate -- canAddStrengthToPass)
///   - usingSafePass       (Boolean tristate -- dontDropFumbles)
///   - minimumRoll         (int)
///   - roll                (int)
/// AbstractStepWithReRoll fields:
///   - re_rolled_action    (ReRolledActions)
///   - re_roll_source      (ReRollSource)
#[derive(Debug, Default)]
pub struct StepHailMaryPassHookState {
    /// Java: state.goToLabelOnFailure
    pub goto_label_on_failure: String,
    /// Java: state.result (PassResult name)
    pub result: Option<String>,
    /// Java: state.passSkillUsed
    pub pass_skill_used: bool,
    /// Java: state.usingModifyingSkill (Boolean -- None = not yet asked)
    pub using_modifying_skill: Option<bool>,
    /// Java: state.usingSafePass (Boolean -- None = not yet asked)
    pub using_safe_pass: Option<bool>,
    /// Java: state.minimumRoll
    pub minimum_roll: i32,
    /// Java: state.roll
    pub roll: i32,
    /// Java: step.getReRolledAction()
    pub re_rolled_action: Option<String>,
    /// Java: step.getReRollSource()
    pub re_roll_source: Option<String>,
    /// Output: step outcome set by hook (mirrors step.getResult().setNextAction)
    pub outcome: Option<crate::step::framework::StepOutcome>,
}

// ---- PassStepModifier -------------------------------------------------------

pub struct PassStepModifier;

impl StepModifierTrait for PassStepModifier {
    /// Java: StepModifier<StepHailMaryPass, StepHailMaryPass.StepState>
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::HailMaryPass }

    fn priority(&self) -> i32 { 0 }

    /// Java: PassBehaviour.handleExecuteStepHook(StepHailMaryPass step, StepState state)
    ///
    /// The Java body resolves a Hail Mary pass roll end-to-end:
    ///   1. Sets thrower coordinate, marks ball/bomb moving.
    ///   2. Fetches PassMechanic + PassModifierFactory, builds PassContext.
    ///   3. On re-roll path: uses or declines the re-roll.
    ///   4. Rolls d6, evaluates result via PassMechanic.evaluatePass.
    ///   5. On fumble: offers pass-skill re-roll dialog OR askForReRollIfAvailable.
    ///   6. Resolves Safe Pass (dontDropFumbles) dialog.
    ///   7. Publishes PASS_FUMBLE; routes to NEXT_STEP or goToLabelOnFailure.
    ///
    /// All of steps 2-7 depend on infra not yet ported (PassMechanic, PassModifierFactory,
    /// PassState, UtilServerDialog, UtilServerReRoll.askForReRollIfAvailable with modificationSkill).
    /// They are marked // headless: and skipped.  The hook returns false (no outcome set),
    /// which matches Java's return value of `false` throughout.
    fn handle_execute_step(
        &self,
        _game: &mut Game,
        _rng: &mut GameRng,
        step_state: &mut dyn std::any::Any,
    ) -> bool {
        let _state = step_state
            .downcast_mut::<StepHailMaryPassHookState>()
            .expect("PassStepModifier: step_state must be StepHailMaryPassHookState");

        // headless: game.getThrower() null check
        // headless: getGameState().getPassState() / setPassState(new PassState())
        // headless: passState.setThrowerCoordinate(...)
        // headless: PlayerAction.HAIL_MARY_BOMB -> setBombMoving / setBallMoving
        // headless: PassMechanic + PassModifierFactory + PassContext + findModifiers
        // headless: re-roll path (ReRolledActions.PASS == reRolledAction)
        //   headless: UtilServerReRoll.useReRoll -> doRoll = true/false
        //   headless: showUseModifyingSkillDialog
        // headless: doRoll block:
        //   headless: usingModifyingSkill path (canAddStrengthToPass + statBasedModifier)
        //   headless: mechanic.evaluatePass(...)
        //   headless: state.minimumRoll calculation
        //   headless: passState.setResult(state.result)
        //   headless: addReport(new ReportPassRoll(...))
        //   headless: fumble re-roll offer:
        //     headless: UtilCards.getRerollSource(thrower, PASS) + Dialog passSkillUsed
        //     headless: UtilServerReRoll.askForReRollIfAvailable(... modificationSkill)
        //   headless: state.usingModifyingSkill == null -> showUseModifyingSkillDialog
        // headless: doNextStep block:
        //   headless: handleSafePass (dontDropFumbles dialog)
        //   headless: publishParameter(PASS_FUMBLE, ...)
        //   headless: SAVED_FUMBLE / FUMBLE / ACCURATE routing
        //   headless: setNextAction(GOTO_LABEL / NEXT_STEP)

        // Java return value is always `false`.
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use crate::step::framework::{StepId, test_team};
    use ffb_model::util::rng::GameRng;

    fn test_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    // ---- PassBehaviour -------------------------------------------------------

    // ---- register_into ------------------------------------------------------

    #[test]
    fn register_into_adds_pass_skill() {
        let mut reg = SkillRegistry::empty();
        PassBehaviour::register_into(&mut reg);
        assert!(reg.get(SkillId::Pass).is_some(), "Pass must be registered");
    }

    #[test]
    fn register_into_registers_one_step_modifier() {
        let mut reg = SkillRegistry::empty();
        PassBehaviour::register_into(&mut reg);
        let sb = reg.get(SkillId::Pass).unwrap();
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    // ---- PassStepModifier ---------------------------------------------------

    #[test]
    fn step_modifier_applies_to_hail_mary_pass() {
        let m = PassStepModifier;
        assert!(m.applies_to(StepId::HailMaryPass));
    }

    #[test]
    fn step_modifier_does_not_apply_to_block_roll() {
        let m = PassStepModifier;
        assert!(!m.applies_to(StepId::BlockRoll));
    }

    #[test]
    fn handle_execute_step_returns_false() {
        let m = PassStepModifier;
        let mut game = test_game();
        let mut rng = GameRng::new(0);
        let mut hook = StepHailMaryPassHookState {
            goto_label_on_failure: "FAIL".into(),
            ..Default::default()
        };
        let result = m.handle_execute_step(&mut game, &mut rng, &mut hook);
        assert!(!result, "handle_execute_step must return false (Java always returns false)");
    }

    #[test]
    fn handle_execute_step_leaves_outcome_unset_when_headless() {
        let m = PassStepModifier;
        let mut game = test_game();
        let mut rng = GameRng::new(0);
        let mut hook = StepHailMaryPassHookState::default();
        m.handle_execute_step(&mut game, &mut rng, &mut hook);
        // headless: all infra is skipped, no outcome is set
        assert!(hook.outcome.is_none());
    }

    #[test]
    fn handle_execute_step_wrong_state_type_panics() {
        let m = PassStepModifier;
        let mut game = test_game();
        let mut rng = GameRng::new(0);
        let mut bad_state: u32 = 42;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            m.handle_execute_step(&mut game, &mut rng, &mut bad_state);
        }));
        assert!(result.is_err(), "wrong state type must panic");
    }

    // ---- StepHailMaryPassHookState ------------------------------------------

    #[test]
    fn hook_state_default_values() {
        let s = StepHailMaryPassHookState::default();
        assert!(!s.pass_skill_used);
        assert!(s.using_modifying_skill.is_none());
        assert!(s.using_safe_pass.is_none());
        assert_eq!(s.roll, 0);
        assert_eq!(s.minimum_roll, 0);
        assert!(s.outcome.is_none());
    }
}
