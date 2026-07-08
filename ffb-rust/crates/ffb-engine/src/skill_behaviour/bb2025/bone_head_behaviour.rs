use crate::skill_behaviour::SkillBehaviour;
use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::StepModifierTrait;
use crate::step::framework::StepId;
use crate::skill_behaviour::registry::SkillRegistry;
use ffb_model::enums::SkillId;

/// Bone Head: player must roll 2+ each activation or stand still this turn.
pub struct BoneHeadBehaviour;

impl BoneHeadBehaviour {
    pub fn new() -> Self { Self }

    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(BoneHeadStepModifier));
        registry.register(SkillId::BoneHead, sb);
    }
}

impl Default for BoneHeadBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for BoneHeadBehaviour {
    fn name(&self) -> &'static str { "BoneHeadBehaviour" }

    fn execute_step_hook(&self, game: &mut ffb_model::model::game::Game) -> bool {
        // Java StepModifier.handleExecuteStepHook: checks UtilCards.hasSkill(actingPlayer, skill), rolls die, on fail sets confused state.
        let has_skill = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill(SkillId::BoneHead))
            .unwrap_or(false);
        if !has_skill {
            return false;
        }
        // TODO(hook-infra): step-specific state access (die roll result, confused state mutation, re-roll dialog) not yet available
        false
    }
}

// ── BoneHeadStepModifier ──────────────────────────────────────────────────────

pub struct BoneHeadStepModifier;

impl StepModifierTrait for BoneHeadStepModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::BoneHead }

    fn priority(&self) -> i32 { 0 }

    // Java: Rolls a confusion check for a player with the Bone Head skill; on failure marks the player confused and inactive, cancels the current player action, and jumps to the failure label, with optional re-roll support.
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
    fn hook_is_noop_returns_false() {
        let behaviour = BoneHeadBehaviour::new();
        let mut game = test_game();
        assert!(!behaviour.execute_step_hook(&mut game));
    }

    #[test]
    fn execute_step_hook_returns_bool() {
        let behaviour = BoneHeadBehaviour::new();
        let mut game = test_game();
        let _result: bool = behaviour.execute_step_hook(&mut game);
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = BoneHeadBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2025,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = BoneHeadBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
    #[test]    fn name_is_not_empty() {        assert!(!BoneHeadBehaviour::new().name().is_empty());    }    #[test]    fn execute_step_hook_false_with_bb2025() {        use ffb_model::enums::Rules;        use crate::step::framework::test_team;        let b = BoneHeadBehaviour::new();        let mut game = ffb_model::model::game::Game::new(            test_team("home", 0), test_team("away", 0), Rules::Bb2025,        );        assert!(!b.execute_step_hook(&mut game));    }

    #[test]
    fn register_into_adds_step_modifier() {
        let mut reg = SkillRegistry::empty();
        BoneHeadBehaviour::register_into(&mut reg);
        let sb = reg.get(SkillId::BoneHead).expect("BoneHead must be registered");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn step_modifier_applies_to_correct_step() {
        let m = BoneHeadStepModifier;
        assert!(m.applies_to(StepId::BoneHead));
    }

    #[test]
    fn step_modifier_does_not_apply_to_wrong_step() {
        let m = BoneHeadStepModifier;
        assert!(!m.applies_to(StepId::BlockRoll));
    }
}
