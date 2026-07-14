/// 1:1 translation of com.fumbbl.ffb.server.skillbehaviour.bb2016.CatchBehaviour.
///
/// Priority 0 modifier on StepCatchScatterThrowIn — byte-identical logic to BB2020/BB2025's
/// CatchBehaviour.java (only package/imports differ per the Java diff).
use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::StepModifierTrait;
use crate::step::framework::StepId;
use crate::skill_behaviour::registry::SkillRegistry;
use crate::step::bb2025::shared::step_catch_scatter_throw_in::StepCatchHookState;
use ffb_model::enums::{ReRollSource, SkillId};
use ffb_model::model::game::Game;
use ffb_model::model::re_rolled_action::ReRolledAction;

// ── CatchStepModifier ──────────────────────────────────────────────────────────

pub struct CatchStepModifier;

impl StepModifierTrait for CatchStepModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::CatchScatterThrowIn }

    fn priority(&self) -> i32 { 0 }

    /// Java: CatchBehaviour.handleExecuteStepHook(StepCatchScatterThrowIn step, StepState state)
    fn handle_execute_step(
        &self,
        game: &mut Game,
        _rng: &mut ffb_model::util::rng::GameRng,
        step_state: &mut dyn std::any::Any,
    ) -> bool {
        let state = step_state
            .downcast_mut::<StepCatchHookState>()
            .expect("CatchStepModifier: step_state must be StepCatchHookState");

        let has_catch = game.player(&state.catcher_id)
            .map(|p| p.has_skill(SkillId::Catch))
            .unwrap_or(false);
        if !has_catch {
            return false;
        }

        state.re_rolled_action = Some(ReRolledAction::new("CATCH"));
        state.re_roll_source = Some(ReRollSource::new("Catch"));
        state.reroll_catch = true;
        true
    }
}

// ── CatchBehaviour ────────────────────────────────────────────────────────────

/// Catch: player may re-roll a failed catch roll once per action.
pub struct CatchBehaviour;

impl CatchBehaviour {
    pub fn new() -> Self { Self }

    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(CatchStepModifier));
        registry.register(SkillId::Catch, sb);
    }
}

impl Default for CatchBehaviour {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill_behaviour::registry::SkillRegistry;
    use crate::step::framework::test_team;
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::util::rng::GameRng;

    fn player_with_skills(id: &str, skills: Vec<SkillId>) -> Player {
        Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: skills.into_iter().map(|s| SkillWithValue { skill_id: s, value: None }).collect(),
            ..Default::default()
        }
    }

    #[test]
    fn register_into_adds_step_modifier() {
        let mut reg = SkillRegistry::empty();
        CatchBehaviour::register_into(&mut reg);
        let sb = reg.get(SkillId::Catch).expect("Catch must be registered");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn step_modifier_applies_to_catch_step() {
        let m = CatchStepModifier;
        assert!(m.applies_to(StepId::CatchScatterThrowIn));
        assert!(!m.applies_to(StepId::BlockRoll));
    }

    #[test]
    fn step_modifier_priority_is_zero() {
        assert_eq!(CatchStepModifier.priority(), 0);
    }

    #[test]
    fn catcher_without_catch_returns_false() {
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016);
        game.team_home.players.push(player_with_skills("catcher", vec![]));

        let m = CatchStepModifier;
        let mut hs = StepCatchHookState::new("catcher".into());
        assert!(!m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs));
        assert!(!hs.reroll_catch);
    }

    #[test]
    fn catcher_with_catch_grants_reroll() {
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016);
        game.team_home.players.push(player_with_skills("catcher", vec![SkillId::Catch]));

        let m = CatchStepModifier;
        let mut hs = StepCatchHookState::new("catcher".into());
        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        assert!(result);
        assert!(hs.reroll_catch);
    }

}
