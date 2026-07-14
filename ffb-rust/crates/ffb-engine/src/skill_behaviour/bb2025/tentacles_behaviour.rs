use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::StepModifierTrait;
use crate::step::framework::StepId;
use crate::skill_behaviour::registry::SkillRegistry;
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;

/// Tentacles: player may make a strength contest to stop a dodging opponent from escaping.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2025.TentaclesBehaviour`.
pub struct TentaclesBehaviour;

impl TentaclesBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for TentaclesBehaviour {
    fn default() -> Self { Self::new() }
}

// ── TentaclesStepModifier ─────────────────────────────────────────────────────

// Java: When a dodging or jumping player leaves a square adjacent to a Tentacles player,
// prompts the opposing team to choose which Tentacles player to attempt the roll, then rolls
// a strength-based dice check (minimum 2, up to 6 minus ST difference) to hold the moving
// player in place and cancel the move.
pub struct TentaclesStepModifier;

impl StepModifierTrait for TentaclesStepModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::Tentacles }

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

impl TentaclesBehaviour {
    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(TentaclesStepModifier));
        registry.register(SkillId::Tentacles, sb);
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
        TentaclesBehaviour::register_into(&mut reg);
        let sb = reg.get(SkillId::Tentacles).expect("Tentacles must be registered");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn step_modifier_applies_to_correct_step() {
        let m = TentaclesStepModifier;
        assert!(m.applies_to(StepId::Tentacles));
    }

    #[test]
    fn step_modifier_does_not_apply_to_wrong_step() {
        let m = TentaclesStepModifier;
        assert!(!m.applies_to(StepId::BlockRoll));
    }
}
