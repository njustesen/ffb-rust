use crate::skill_behaviour::SkillBehaviour;
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

impl SkillBehaviour for BombardierBehaviour {
    fn name(&self) -> &'static str { "BombardierBehaviour" }

    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        // Java StepModifier.handleExecuteStepHook: checks UtilCards.hasSkill(actingPlayer, skill)
        // for the Bombardier skill (identified via canThrowBombs property).
        // TODO(hook-infra): SkillId::Bombardier is not in the current SkillId enum;
        //   has_skill_property("canThrowBombs") is not available in this crate version.
        //   Full skill check deferred until hook infrastructure and property lookup are ported.
        false
    }
}

// Java: Marks the Bombardier skill as used and switches the game's turn mode to the appropriate bomb mode (BOMB_HOME, BOMB_AWAY, BOMB_HOME_BLITZ, or BOMB_AWAY_BLITZ) when a player performs a throw-bomb or hail-mary-bomb action outside a designated bomb turn.
pub struct BombardierStepModifier;

impl StepModifierTrait for BombardierStepModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::Bombardier }

    fn handle_execute_step(
        &self,
        _game: &mut ffb_model::model::game::Game,
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
    fn hook_is_noop_returns_false() {
        let behaviour = BombardierBehaviour::new();
        let mut game = test_game();
        assert!(!behaviour.execute_step_hook(&mut game));
    }

    #[test]
    fn execute_step_hook_returns_bool() {
        let behaviour = BombardierBehaviour::new();
        let mut game = test_game();
        let _result: bool = behaviour.execute_step_hook(&mut game);
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = BombardierBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2025,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = BombardierBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
#[test]    fn name_is_not_empty() {        assert!(!BombardierBehaviour::new().name().is_empty());    }    #[test]    fn execute_step_hook_false_with_bb2025() {        use ffb_model::enums::Rules;        use crate::step::framework::test_team;        let b = BombardierBehaviour::new();        let mut game = ffb_model::model::game::Game::new(            test_team("home", 0), test_team("away", 0), Rules::Bb2025,        );        assert!(!b.execute_step_hook(&mut game));    }

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
