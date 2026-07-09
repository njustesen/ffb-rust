/// 1:1 translation of com.fumbbl.ffb.server.skillbehaviour.bb2020.PilingOnBehaviour.
///
/// PilingOn also exists in BB2020 (inherited from BB2016). The step modifier hooks into
/// StepDropFallingPlayers. Full logic is deferred until StepDropFallingPlayersHookState is ported.
use crate::skill_behaviour::SkillBehaviour;
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

impl SkillBehaviour for PilingOnBehaviour {
    fn name(&self) -> &'static str { "PilingOnBehaviour" }

    /// Java `StepModifier<StepDropFallingPlayers, StepState>.handleExecuteStepHook`: after
    /// knockdown checks if PilingOn player can re-roll injury, shows dialog (DialogPilingOn),
    /// rolls 2+, handles Brawler reroll and WeepingDagger interaction. Returns false always.
    /// TODO(hook-infra): needs state.usingPilingOn, state.foulerId, state.victim, game option
    /// checks.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        // TODO(hook-infra): step-specific state access (StepState.xxx)
        false
    }
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
    fn name_is_correct() {
        assert_eq!(PilingOnBehaviour::new().name(), "PilingOnBehaviour");
    }

    #[test]
    fn name_is_not_empty() {
        assert!(!PilingOnBehaviour::new().name().is_empty());
    }

    #[test]
    fn execute_step_hook_returns_false() {
        let b = PilingOnBehaviour::new();
        let mut game = test_game();
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = PilingOnBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
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
