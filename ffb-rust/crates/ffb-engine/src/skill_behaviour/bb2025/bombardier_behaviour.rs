use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::StepModifierTrait;
use crate::step::framework::StepId;
use crate::skill_behaviour::registry::SkillRegistry;
use ffb_model::enums::SkillId;

/// Bombardier: allows the player to throw a bomb instead of the ball.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2025.BombardierBehaviour`.
pub struct BombardierBehaviour;

impl BombardierBehaviour {
    pub fn new() -> Self { Self }

    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(BombardierStepModifier));
        registry.register(SkillId::Bombardier, sb);
    }
}

impl Default for BombardierBehaviour {
    fn default() -> Self { Self::new() }
}

// Java: Marks the Bombardier skill as used and switches the game's turn mode to the appropriate bomb mode (BOMB_HOME, BOMB_AWAY, BOMB_HOME_BLITZ, or BOMB_AWAY_BLITZ) when a player performs a throw-bomb or hail-mary-bomb action outside a designated bomb turn.
pub struct BombardierStepModifier;

impl StepModifierTrait for BombardierStepModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::Bombardier }

    fn handle_execute_step(
        &self,
        _game: &mut ffb_model::model::game::Game,
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
        BombardierBehaviour::register_into(&mut reg);
        let sb = reg.get(SkillId::Bombardier).expect("Bombardier must be registered");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn step_modifier_applies_to_correct_step() {
        let m = BombardierStepModifier;
        assert!(m.applies_to(StepId::Bombardier));
    }

    #[test]
    fn step_modifier_does_not_apply_to_wrong_step() {
        let m = BombardierStepModifier;
        assert!(!m.applies_to(StepId::BlockRoll));
    }
}
