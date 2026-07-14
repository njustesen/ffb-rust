use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::StepModifierTrait;
use crate::step::framework::StepId;
use crate::skill_behaviour::registry::SkillRegistry;
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;

// Java: When the acting player has the Stab skill and usingStab is true, performs an armour roll (injury) against the defender using InjuryTypeStab instead of a normal block, then publishes a DropPlayerContext and advances to the next step.
pub struct StabStepModifier;

impl StepModifierTrait for StabStepModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::Stab }

    fn priority(&self) -> i32 { 0 }

    fn handle_execute_step(
        &self,
        _game: &mut Game,
        _rng: &mut ffb_model::util::rng::GameRng,
        _step_state: &mut dyn std::any::Any,
    ) -> bool {
        false
    }
}

/// Stab: player may stab instead of blocking; injury roll on success but no block dice.
pub struct StabBehaviour;

impl StabBehaviour {
    pub fn new() -> Self { Self }

    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(StabStepModifier));
        registry.register(SkillId::Stab, sb);
    }
}

impl Default for StabBehaviour {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill_behaviour::registry::SkillRegistry;

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
        StabBehaviour::register_into(&mut reg);
        let sb = reg.get(SkillId::Stab).expect("Stab must be registered");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn step_modifier_applies_to_correct_step() {
        let m = StabStepModifier;
        assert!(m.applies_to(StepId::Stab));
    }

    #[test]
    fn step_modifier_does_not_apply_to_wrong_step() {
        let m = StabStepModifier;
        assert!(!m.applies_to(StepId::BlockRoll));
    }
}
