/// 1:1 translation of com.fumbbl.ffb.server.skillbehaviour.bb2025.TheBallistaBehaviour.
///
/// TheBallistaBehaviour registers two step modifiers:
///   1. StepModifier<StepThrowTeamMate> (priority 1) — handleExecuteStepHook returns false;
///      handleCommandHook sets `reRolledAction` to KICK_TEAM_MATE or THROW_TEAM_MATE
///      (depending on `state.kicked`) and `reRollSource` to THE_BALLISTA iff the skill was used.
///   2. StepModifier<StepHailMaryPass>  (priority 0) — handleExecuteStepHook returns false;
///      handleCommandHook always sets `reRolledAction` to PASS.
///
/// Both execute-step hooks are no-ops in Java (return false) — the real effect is
/// `handleCommandHook` presetting the step's re-roll state before it re-executes.
use crate::skill_behaviour::SkillBehaviour;
use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::{RerollHookState, StepModifierTrait};
use crate::step::framework::{StepCommandStatus, StepId};
use crate::skill_behaviour::registry::SkillRegistry;
use ffb_model::enums::SkillId;

/// Java: `ReRollSources.THE_BALLISTA` — the re-roll source name recorded on a step when
/// TheBallista is used. Matches `SkillId::TheBallista`'s `Debug` name so
/// `util_server_re_roll::use_reroll`'s skill-based fallback can mark it used.
const THE_BALLISTA_RE_ROLL_SOURCE: &str = "TheBallista";

pub struct TheBallistaBehaviour;

impl TheBallistaBehaviour {
    pub fn new() -> Self { Self }

    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        // Java registers ThrowTeamMate modifier first (priority 1), then HailMaryPass (default).
        sb.register_step_modifier(Box::new(TheBallistaThrowTeamMateModifier));
        sb.register_step_modifier(Box::new(TheBallistaHailMaryPassModifier));
        registry.register(SkillId::TheBallista, sb);
    }
}

impl Default for TheBallistaBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for TheBallistaBehaviour {
    fn name(&self) -> &'static str { "TheBallistaBehaviour" }

    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        // Both Java step modifiers return false from handleExecuteStepHook.
        false
    }
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
    /// ReRolledAction action = state.kicked ? ReRolledActions.KICK_TEAM_MATE : ReRolledActions.THROW_TEAM_MATE;
    /// step.setReRolledAction(action);
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
            state.re_rolled_action = Some(
                if state.kicked { "KICK_TEAM_MATE" } else { "THROW_TEAM_MATE" }.to_string(),
            );
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
    use ffb_model::enums::Rules;
    use crate::step::framework::test_team;

    fn test_game() -> ffb_model::model::game::Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        ffb_model::model::game::Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn name_is_not_empty() {
        assert!(!TheBallistaBehaviour::new().name().is_empty());
    }

    #[test]
    fn execute_step_hook_returns_false() {
        let b = TheBallistaBehaviour::new();
        let mut game = test_game();
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = TheBallistaBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
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
        let m = TheBallistaThrowTeamMateModifier;
        assert!(m.applies_to(StepId::ThrowTeamMate));
    }

    #[test]
    fn throw_team_mate_modifier_does_not_apply_to_wrong_step() {
        let m = TheBallistaThrowTeamMateModifier;
        assert!(!m.applies_to(StepId::HailMaryPass));
    }

    #[test]
    fn throw_team_mate_modifier_priority_is_one() {
        let m = TheBallistaThrowTeamMateModifier;
        assert_eq!(m.priority(), 1);
    }

    #[test]
    fn throw_team_mate_modifier_execute_step_returns_false() {
        use ffb_model::util::rng::GameRng;
        let m = TheBallistaThrowTeamMateModifier;
        let mut game = test_game();
        let mut state: () = ();
        assert!(!m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut state));
    }

    #[test]
    fn hail_mary_pass_modifier_applies_to_correct_step() {
        let m = TheBallistaHailMaryPassModifier;
        assert!(m.applies_to(StepId::HailMaryPass));
    }

    #[test]
    fn hail_mary_pass_modifier_does_not_apply_to_wrong_step() {
        let m = TheBallistaHailMaryPassModifier;
        assert!(!m.applies_to(StepId::ThrowTeamMate));
    }

    #[test]
    fn hail_mary_pass_modifier_priority_is_zero() {
        let m = TheBallistaHailMaryPassModifier;
        assert_eq!(m.priority(), 0);
    }

    #[test]
    fn hail_mary_pass_modifier_execute_step_returns_false() {
        use ffb_model::util::rng::GameRng;
        let m = TheBallistaHailMaryPassModifier;
        let mut game = test_game();
        let mut state: () = ();
        assert!(!m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut state));
    }

    #[test]
    fn throw_team_mate_handle_command_sets_throw_team_mate_when_not_kicked() {
        let m = TheBallistaThrowTeamMateModifier;
        let mut game = test_game();
        let mut state = RerollHookState { kicked: false, ..Default::default() };
        let status = m.handle_command(&mut game, &mut state, SkillId::TheBallista, true);
        assert_eq!(status, StepCommandStatus::ExecuteStep);
        assert_eq!(state.re_rolled_action.as_deref(), Some("THROW_TEAM_MATE"));
        assert_eq!(state.re_roll_source.as_deref(), Some("TheBallista"));
    }

    #[test]
    fn throw_team_mate_handle_command_sets_kick_team_mate_when_kicked() {
        let m = TheBallistaThrowTeamMateModifier;
        let mut game = test_game();
        let mut state = RerollHookState { kicked: true, ..Default::default() };
        m.handle_command(&mut game, &mut state, SkillId::TheBallista, true);
        assert_eq!(state.re_rolled_action.as_deref(), Some("KICK_TEAM_MATE"));
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
