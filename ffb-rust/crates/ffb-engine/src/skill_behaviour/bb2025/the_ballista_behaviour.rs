/// 1:1 translation of com.fumbbl.ffb.server.skillbehaviour.bb2025.TheBallistaBehaviour.
///
/// TheBallistaBehaviour registers two step modifiers:
///   1. StepModifier<StepThrowTeamMate> (priority 1) — handleExecuteStepHook returns false.
///   2. StepModifier<StepHailMaryPass>  (priority 0) — handleExecuteStepHook returns false.
///
/// Both execute-step hooks are no-ops in Java (return false). The command hook
/// (handleCommandHook) sets re-roll state on the step from the UseSkill client command,
/// but that path is headless infrastructure not yet ported.
use crate::skill_behaviour::SkillBehaviour;
use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::StepModifierTrait;
use crate::step::framework::StepId;
use crate::skill_behaviour::registry::SkillRegistry;
use ffb_model::enums::SkillId;

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
        // headless: handleCommandHook sets reRolledAction/reRollSource from UseSkill command
        //           (step-specific re-roll state access not yet ported)
        false
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
        // headless: handleCommandHook sets reRolledAction/reRollSource from UseSkill command
        //           (step-specific re-roll state access not yet ported)
        false
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
}
