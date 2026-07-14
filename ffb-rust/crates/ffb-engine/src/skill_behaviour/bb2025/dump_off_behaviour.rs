use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::StepModifierTrait;
use crate::step::framework::StepId;
use crate::skill_behaviour::registry::SkillRegistry;
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;

// ── DumpOffStepModifier ───────────────────────────────────────────────────────

pub struct DumpOffStepModifier;

impl StepModifierTrait for DumpOffStepModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::DumpOff }

    fn priority(&self) -> i32 { 0 }

    // Java: Handles the Dump Off skill by prompting the defending ball-carrier to optionally make a pass before the block resolves, then pushing a Pass sequence onto the step stack if accepted.
    // Real logic already lives directly in step/action/block/step_dump_off.rs (checks
    // SkillId::DumpOff directly, dialog-gated, TurnMode switching) - this modifier body stays a
    // no-op to avoid dead duplicate logic; registration is harmless now that applies_to targets
    // the correct StepId.
    fn handle_execute_step(
        &self,
        _game: &mut Game,
        _rng: &mut ffb_model::util::rng::GameRng,
        _step_state: &mut dyn std::any::Any,
    ) -> bool {
        false
    }
}

// ── DumpOffBehaviour (marker + registration) ──────────────────────────────────

/// Dump-Off: player may make a quick pass when targeted by a block.
pub struct DumpOffBehaviour;

impl DumpOffBehaviour {
    pub fn new() -> Self { Self }

    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(DumpOffStepModifier));
        registry.register(SkillId::DumpOff, sb);
    }
}

impl Default for DumpOffBehaviour {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill_behaviour::registry::SkillRegistry;

    #[test]
    fn step_modifier_applies_to_dump_off_step() {
        assert!(DumpOffStepModifier.applies_to(StepId::DumpOff));
        assert!(!DumpOffStepModifier.applies_to(StepId::BlockRoll));
    }

    #[test]
    fn register_into_adds_step_modifier() {
        let mut reg = SkillRegistry::empty();
        DumpOffBehaviour::register_into(&mut reg);
        let sb = reg.get(SkillId::DumpOff).expect("DumpOff must be registered");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    fn test_game() -> ffb_model::model::game::Game {
        let home = ffb_model::model::team::Team {
            id: "home".into(), name: "Home".into(), race: "human".into(),
            roster_id: "human".into(), coach: "Coach".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0,
            team_value: 0, treasury: 0, special_rules: vec![], players: vec![],
            vampire_lord: false,
            necromancer: false,
        };
        let away = home.clone();
        ffb_model::model::game::Game::new(home, away, ffb_model::enums::Rules::Bb2025)
    }

}
