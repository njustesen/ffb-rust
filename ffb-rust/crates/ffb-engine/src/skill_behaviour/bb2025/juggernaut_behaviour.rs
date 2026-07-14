use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::StepModifierTrait;
use crate::step::framework::StepId;
use crate::skill_behaviour::registry::SkillRegistry;
use ffb_model::enums::SkillId;

/// Juggernaut: a Blitz block may treat Both-Down as a Push result.
///
/// **This modifier is dead/unreachable code** (Phase AAH audit): it targets `StepId::Juggernaut`,
/// which no step ever dispatches through `dispatch::execute_step_hooks`. Java's edition-identical
/// `JuggernautBehaviour.java` (bb2025 and mixed/bb2016/bb2020 copies) is ported directly into
/// `step/action/block/step_juggernaut.rs` instead (one edition-agnostic file, confirmed complete
/// during Phase AAH's investigation) — the same "direct-in-step" pattern already established for
/// Wrestle/Stab/DumpOff/Bombardier/Dauntless. Left registered rather than deleted, matching that
/// precedent.
pub struct JuggernautBehaviour;

impl JuggernautBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for JuggernautBehaviour {
    fn default() -> Self { Self::new() }
}

// ── JuggernautStepModifier ────────────────────────────────────────────────────

// Java: When the acting player uses Juggernaut on a Blitz action, forces a pushback result and initiates the pushback sequence, or declines and advances to the next step.
pub struct JuggernautStepModifier;

impl StepModifierTrait for JuggernautStepModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::Juggernaut }

    fn priority(&self) -> i32 { 0 }

    fn handle_execute_step(
        &self,
        _game: &mut ffb_model::model::game::Game,
        _rng: &mut ffb_model::util::rng::GameRng,
        _step_state: &mut dyn std::any::Any,
    ) -> bool {
        false
    }
}

impl JuggernautBehaviour {
    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(JuggernautStepModifier));
        registry.register(SkillId::Juggernaut, sb);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn register_into_adds_step_modifier() {
        let mut reg = SkillRegistry::empty();
        JuggernautBehaviour::register_into(&mut reg);
        let sb = reg.get(SkillId::Juggernaut).expect("Juggernaut must be registered");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn step_modifier_applies_to_correct_step() {
        let m = JuggernautStepModifier;
        assert!(m.applies_to(StepId::Juggernaut));
    }

    #[test]
    fn step_modifier_does_not_apply_to_wrong_step() {
        let m = JuggernautStepModifier;
        assert!(!m.applies_to(StepId::BlockRoll));
    }
}
