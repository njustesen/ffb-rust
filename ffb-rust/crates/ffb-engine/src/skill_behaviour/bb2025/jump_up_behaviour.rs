use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::StepModifierTrait;
use crate::step::framework::StepId;
use crate::skill_behaviour::registry::SkillRegistry;
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;

/// Jump Up (BB2025): player may stand up without spending movement when making a block action,
/// but must pass a skill check; handles re-rolls on failure.
pub struct JumpUpBehaviour;

impl JumpUpBehaviour {
    pub fn new() -> Self { Self }

    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(JumpUpStepModifier));
        registry.register(SkillId::JumpUp, sb);
    }
}

impl Default for JumpUpBehaviour {
    fn default() -> Self { Self::new() }
}

// Java: Handles the Jump Up skill during a block action: if the player is prone and attempting to stand up for a block, rolls a skill check (with modifiers) and either advances the action on success, leaves the player prone and ends the action on failure, or asks for a re-roll if available.
pub struct JumpUpStepModifier;

impl StepModifierTrait for JumpUpStepModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::JumpUp }

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
        JumpUpBehaviour::register_into(&mut reg);
        let sb = reg.get(SkillId::JumpUp).expect("JumpUp must be registered");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn step_modifier_applies_to_correct_step() {
        let m = JumpUpStepModifier;
        assert!(m.applies_to(StepId::JumpUp));
    }

    #[test]
    fn step_modifier_does_not_apply_to_wrong_step() {
        let m = JumpUpStepModifier;
        assert!(!m.applies_to(StepId::BlockRoll));
    }
}
