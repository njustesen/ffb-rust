use crate::skill_behaviour::SkillBehaviour;
use ffb_model::enums::SkillId;

/// Foul Appearance: opponents must roll 2+ before performing a block or foul.
pub struct FoulAppearanceBehaviour;

impl FoulAppearanceBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for FoulAppearanceBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for FoulAppearanceBehaviour {
    fn name(&self) -> &'static str { "FoulAppearanceBehaviour" }

    fn execute_step_hook(&self, game: &mut ffb_model::model::game::Game) -> bool {
        // Java StepModifier.handleExecuteStepHook: iterates adjacent opponents checking FoulAppearance; if defender has it, attacker must roll 2+.
        let has_skill = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill(SkillId::FoulAppearance))
            .unwrap_or(false);
        if !has_skill {
            return false;
        }
        // TODO(hook-infra): step-specific state access (adjacent opponent iteration, FoulAppearance roll, re-roll dialog) not yet available
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
        };
        let away = home.clone();
        ffb_model::model::game::Game::new(home, away, ffb_model::enums::Rules::Bb2025)
    }

    #[test]
    fn hook_is_noop_returns_false() {
        let behaviour = FoulAppearanceBehaviour::new();
        let mut game = test_game();
        assert!(!behaviour.execute_step_hook(&mut game));
    }

    #[test]
    fn execute_step_hook_returns_bool() {
        let behaviour = FoulAppearanceBehaviour::new();
        let mut game = test_game();
        let _result: bool = behaviour.execute_step_hook(&mut game);
    }
}
