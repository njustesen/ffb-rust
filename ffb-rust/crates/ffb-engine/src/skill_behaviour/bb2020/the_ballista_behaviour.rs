use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::{RerollHookState, StepModifierTrait};
use crate::step::framework::{StepCommandStatus, StepId};
use crate::skill_behaviour::registry::SkillRegistry;
use ffb_model::enums::SkillId;

/// Java: `ReRollSources.THE_BALLISTA` — matches `SkillId::TheBallista`'s `Debug` name so
/// `util_server_re_roll::use_reroll`'s skill-based fallback can mark it used.
const THE_BALLISTA_RE_ROLL_SOURCE: &str = "TheBallista";

/// BB2020 TheBallista skill behaviour.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2020.TheBallistaBehaviour`.
///
/// **BB2020 vs BB2025 difference:**
///
/// BB2020 always sets the re-rolled action to `THROW_TEAM_MATE`:
/// ```java
/// step.setReRolledAction(ReRolledActions.THROW_TEAM_MATE);
/// ```
///
/// BB2025 selects between `KICK_TEAM_MATE` and `THROW_TEAM_MATE` based on `state.kicked`:
/// ```java
/// ReRolledAction action = state.kicked ? ReRolledActions.KICK_TEAM_MATE : ReRolledActions.THROW_TEAM_MATE;
/// step.setReRolledAction(action);
/// ```
///
/// This means BB2020 does not support the kick-team-mate (KTM) re-roll distinction that BB2025
/// added for the Treeman/Halfling KTM mechanic.
pub struct TheBallistaBehaviour;

impl TheBallistaBehaviour {
    pub fn new() -> Self { Self }

    /// Returns the re-rolled action kind for TheBallista in **BB2020** (always ThrowTeamMate).
    ///
    /// In BB2025 this depends on whether the action is a kick (`state.kicked`).
    pub fn rerolled_action_bb2020(_kicked: bool) -> RerolledActionKind {
        // BB2020: always ThrowTeamMate, never KickTeamMate.
        RerolledActionKind::ThrowTeamMate
    }

    /// Register the ThrowTeamMate + HailMaryPass step modifiers into the skill registry.
    /// Java: `TheBallistaBehaviour()` constructor's two `registerModifier(...)` calls.
    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(TheBallistaThrowTeamMateModifier));
        sb.register_step_modifier(Box::new(TheBallistaHailMaryPassModifier));
        registry.register(SkillId::TheBallista, sb);
    }
}

/// Re-rolled action kinds relevant to TheBallista.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RerolledActionKind {
    ThrowTeamMate,
    KickTeamMate,
}

impl Default for TheBallistaBehaviour {
    fn default() -> Self { Self::new() }
}

// ── TheBallistaThrowTeamMateModifier ─────────────────────────────────────────
// Java: StepModifier<StepThrowTeamMate, StepThrowTeamMate.StepState>(priority=1)

pub struct TheBallistaThrowTeamMateModifier;

impl StepModifierTrait for TheBallistaThrowTeamMateModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::ThrowTeamMate }

    fn priority(&self) -> i32 { 1 }

    /// Java: handleExecuteStepHook(StepThrowTeamMate step, StepState state) { return false; }
    fn handle_execute_step(
        &self,
        _game: &mut ffb_model::model::game::Game,
        _rng: &mut ffb_model::util::rng::GameRng,
        _step_state: &mut dyn std::any::Any,
    ) -> bool {
        false
    }

    /// Java:
    /// ```java
    /// step.setReRolledAction(ReRolledActions.THROW_TEAM_MATE);
    /// step.setReRollSource(useSkillCommand.isSkillUsed() ? getReRollSource() : null);
    /// return StepCommandStatus.EXECUTE_STEP;
    /// ```
    /// BB2020 always uses THROW_TEAM_MATE — no kicked/KTM distinction (see module docs).
    fn handle_command(
        &self,
        _game: &mut ffb_model::model::game::Game,
        step_state: &mut dyn std::any::Any,
        _skill_id: SkillId,
        skill_used: bool,
    ) -> StepCommandStatus {
        if let Some(state) = step_state.downcast_mut::<RerollHookState>() {
            state.re_rolled_action = Some("THROW_TEAM_MATE".to_string());
            state.re_roll_source = skill_used.then(|| THE_BALLISTA_RE_ROLL_SOURCE.to_string());
        }
        StepCommandStatus::ExecuteStep
    }
}

// ── TheBallistaHailMaryPassModifier ──────────────────────────────────────────
// Java: StepModifier<StepHailMaryPass, StepHailMaryPass.StepState>(priority=0/default)

pub struct TheBallistaHailMaryPassModifier;

impl StepModifierTrait for TheBallistaHailMaryPassModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::HailMaryPass }

    fn priority(&self) -> i32 { 0 }

    /// Java: handleExecuteStepHook(StepHailMaryPass step, StepState state) { return false; }
    fn handle_execute_step(
        &self,
        _game: &mut ffb_model::model::game::Game,
        _rng: &mut ffb_model::util::rng::GameRng,
        _step_state: &mut dyn std::any::Any,
    ) -> bool {
        false
    }

    /// Java:
    /// ```java
    /// step.setReRolledAction(ReRolledActions.PASS);
    /// step.setReRollSource(useSkillCommand.isSkillUsed() ? getReRollSource() : null);
    /// return StepCommandStatus.EXECUTE_STEP;
    /// ```
    fn handle_command(
        &self,
        _game: &mut ffb_model::model::game::Game,
        step_state: &mut dyn std::any::Any,
        _skill_id: SkillId,
        skill_used: bool,
    ) -> StepCommandStatus {
        if let Some(state) = step_state.downcast_mut::<RerollHookState>() {
            state.re_rolled_action = Some("PASS".to_string());
            state.re_roll_source = skill_used.then(|| THE_BALLISTA_RE_ROLL_SOURCE.to_string());
        }
        StepCommandStatus::ExecuteStep
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BB2020: rerolled action is always ThrowTeamMate, even when kicked=true.
    #[test]
    fn bb2020_always_uses_throw_team_mate_when_kicked() {
        assert_eq!(
            TheBallistaBehaviour::rerolled_action_bb2020(true),
            RerolledActionKind::ThrowTeamMate
        );
    }

    /// BB2020: rerolled action is ThrowTeamMate when kicked=false.
    #[test]
    fn bb2020_always_uses_throw_team_mate_when_not_kicked() {
        assert_eq!(
            TheBallistaBehaviour::rerolled_action_bb2020(false),
            RerolledActionKind::ThrowTeamMate
        );
    }

    /// BB2020 never returns KickTeamMate.
    #[test]
    fn bb2020_never_uses_kick_team_mate() {
        for kicked in [true, false] {
            assert_ne!(
                TheBallistaBehaviour::rerolled_action_bb2020(kicked),
                RerolledActionKind::KickTeamMate,
                "BB2020 must never select KickTeamMate re-roll action"
            );
        }
    }

    fn test_game() -> ffb_model::model::game::Game {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        ffb_model::model::game::Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn register_into_adds_two_step_modifiers() {
        let mut reg = SkillRegistry::empty();
        TheBallistaBehaviour::register_into(&mut reg);
        let sb = reg.get(SkillId::TheBallista).expect("TheBallista must be registered");
        assert_eq!(sb.get_step_modifiers().len(), 2);
    }

    #[test]
    fn throw_team_mate_modifier_applies_to_correct_step() {
        assert!(TheBallistaThrowTeamMateModifier.applies_to(StepId::ThrowTeamMate));
        assert!(!TheBallistaThrowTeamMateModifier.applies_to(StepId::HailMaryPass));
    }

    #[test]
    fn throw_team_mate_modifier_priority_is_one() {
        assert_eq!(TheBallistaThrowTeamMateModifier.priority(), 1);
    }

    #[test]
    fn throw_team_mate_handle_command_always_sets_throw_team_mate() {
        let m = TheBallistaThrowTeamMateModifier;
        let mut game = test_game();
        // BB2020 ignores `kicked` — always THROW_TEAM_MATE, never KICK_TEAM_MATE.
        let mut state = RerollHookState { kicked: true, ..Default::default() };
        let status = m.handle_command(&mut game, &mut state, SkillId::TheBallista, true);
        assert_eq!(status, StepCommandStatus::ExecuteStep);
        assert_eq!(state.re_rolled_action.as_deref(), Some("THROW_TEAM_MATE"));
        assert_eq!(state.re_roll_source.as_deref(), Some("TheBallista"));
    }

    #[test]
    fn throw_team_mate_handle_command_clears_source_when_declined() {
        let m = TheBallistaThrowTeamMateModifier;
        let mut game = test_game();
        let mut state = RerollHookState::default();
        m.handle_command(&mut game, &mut state, SkillId::TheBallista, false);
        assert_eq!(state.re_rolled_action.as_deref(), Some("THROW_TEAM_MATE"));
        assert!(state.re_roll_source.is_none());
    }

    #[test]
    fn hail_mary_pass_modifier_applies_to_correct_step() {
        assert!(TheBallistaHailMaryPassModifier.applies_to(StepId::HailMaryPass));
        assert!(!TheBallistaHailMaryPassModifier.applies_to(StepId::ThrowTeamMate));
    }

    #[test]
    fn hail_mary_pass_modifier_priority_is_zero() {
        assert_eq!(TheBallistaHailMaryPassModifier.priority(), 0);
    }

    #[test]
    fn hail_mary_pass_handle_command_sets_pass_action() {
        let m = TheBallistaHailMaryPassModifier;
        let mut game = test_game();
        let mut state = RerollHookState::default();
        let status = m.handle_command(&mut game, &mut state, SkillId::TheBallista, true);
        assert_eq!(status, StepCommandStatus::ExecuteStep);
        assert_eq!(state.re_rolled_action.as_deref(), Some("PASS"));
        assert_eq!(state.re_roll_source.as_deref(), Some("TheBallista"));
    }

    #[test]
    fn hail_mary_pass_handle_command_clears_source_when_declined() {
        let m = TheBallistaHailMaryPassModifier;
        let mut game = test_game();
        let mut state = RerollHookState::default();
        m.handle_command(&mut game, &mut state, SkillId::TheBallista, false);
        assert!(state.re_roll_source.is_none());
    }
}
