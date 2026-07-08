use crate::skill_behaviour::SkillBehaviour;
use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::StepModifierTrait;
use crate::step::framework::StepId;
use crate::skill_behaviour::registry::SkillRegistry;
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;

/// Shadowing: player may follow a dodging opponent one square, forcing continued dodging.
pub struct ShadowingBehaviour;

impl ShadowingBehaviour {
    pub fn new() -> Self { Self }

    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(ShadowingStepModifier));
        registry.register(SkillId::Shadowing, sb);
    }
}

impl Default for ShadowingBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for ShadowingBehaviour {
    fn name(&self) -> &'static str { "ShadowingBehaviour" }

    fn execute_step_hook(&self, game: &mut ffb_model::model::game::Game) -> bool {
        // Java StepModifier.handleExecuteStepHook: checks acting player has Shadowing; complex shadow logic finds eligible shadowers and resolves contest.
        let has_skill = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill(SkillId::Shadowing))
            .unwrap_or(false);
        if !has_skill {
            return false;
        }
        // TODO(hook-infra): step-specific state access (dodging player position, shadow contest roll, player movement) not yet available
        false
    }
}

// ── ShadowingStepModifier ─────────────────────────────────────────────────────

// Java: Finds adjacent opposing players with the Shadowing skill, prompts the defending team to choose a shadower, rolls a skill check (4+) with re-roll support, and if successful moves the shadowing player to follow the acting player's previous position.
pub struct ShadowingStepModifier;

impl StepModifierTrait for ShadowingStepModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::Shadowing }

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
    use crate::step::framework::test_team;
    use ffb_model::enums::Rules;

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
    fn hook_is_noop_returns_false() {
        let behaviour = ShadowingBehaviour::new();
        let mut game = test_game();
        assert!(!behaviour.execute_step_hook(&mut game));
    }

    #[test]
    fn execute_step_hook_returns_bool() {
        let behaviour = ShadowingBehaviour::new();
        let mut game = test_game();
        let _result: bool = behaviour.execute_step_hook(&mut game);
    }

    #[test]
    fn execute_step_hook_returns_false() {
        let b = ShadowingBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2025,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = ShadowingBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
    #[test]    fn name_is_not_empty() {        assert!(!ShadowingBehaviour::new().name().is_empty());    }    #[test]    fn execute_step_hook_false_with_bb2025() {        let b = ShadowingBehaviour::new();        let mut game = ffb_model::model::game::Game::new(            test_team("home", 0), test_team("away", 0), Rules::Bb2025,        );        assert!(!b.execute_step_hook(&mut game));    }

    #[test]
    fn register_into_adds_step_modifier() {
        let mut reg = SkillRegistry::empty();
        ShadowingBehaviour::register_into(&mut reg);
        let sb = reg.get(SkillId::Shadowing).expect("Shadowing must be registered");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn step_modifier_applies_to_correct_step() {
        let m = ShadowingStepModifier;
        assert!(m.applies_to(StepId::Shadowing));
    }

    #[test]
    fn step_modifier_does_not_apply_to_wrong_step() {
        let m = ShadowingStepModifier;
        assert!(!m.applies_to(StepId::BlockRoll));
    }
}
