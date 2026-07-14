/// 1:1 translation of com.fumbbl.ffb.server.skillbehaviour.bb2020.PilingOnBehaviour.
///
/// PilingOn also exists in BB2020 (inherited from BB2016). The step modifier hooks into
/// StepDropFallingPlayers. Full logic is deferred until StepDropFallingPlayersHookState is ported.
use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::skill_behaviour::registry::SkillRegistry;
use ffb_model::enums::SkillId;

pub struct PilingOnBehaviour;

impl PilingOnBehaviour {
    pub fn new() -> Self { Self }

    /// Register PilingOn into the BB2020 skill registry.
    /// No step modifier yet — full logic deferred until StepDropFallingPlayersHookState is ported.
    pub fn register_into(registry: &mut SkillRegistry) {
        let sb = SbContainer::new();
        registry.register(SkillId::PilingOn, sb);
    }
}

impl Default for PilingOnBehaviour {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use crate::step::framework::test_team;
    use ffb_model::model::game::Game;

    fn test_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn register_into_registers_piling_on_skill() {
        let mut reg = SkillRegistry::empty();
        PilingOnBehaviour::register_into(&mut reg);
        let sb = reg.get(SkillId::PilingOn).expect("PilingOn must be registered");
        // No step modifier yet — deferred until StepDropFallingPlayersHookState is ported
        assert_eq!(sb.get_step_modifiers().len(), 0,
            "PilingOn step modifier stub: no modifiers until hook infra is ported");
    }
}
