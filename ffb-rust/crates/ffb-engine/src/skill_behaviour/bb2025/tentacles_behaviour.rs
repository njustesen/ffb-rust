use crate::skill_behaviour::SkillBehaviour;
use ffb_model::enums::SkillId;

/// Tentacles: player may make a strength contest to stop a dodging opponent from escaping.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2025.TentaclesBehaviour`.
pub struct TentaclesBehaviour;

impl TentaclesBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for TentaclesBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for TentaclesBehaviour {
    fn name(&self) -> &'static str { "TentaclesBehaviour" }

    fn execute_step_hook(&self, game: &mut ffb_model::model::game::Game) -> bool {
        // Java StepModifier.handleExecuteStepHook: complex -- iterates adjacent opponents
        // checking if they have Tentacles; if so, contests the dodge with a strength roll.
        // The Tentacles holder is an adjacent opponent, not necessarily acting_player.
        let _has_skill = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill(SkillId::Tentacles))
            .unwrap_or(false);
        // TODO(hook-infra): step-specific state access (adjacent opponent iteration,
        //   Tentacles holder identification, strength contest roll) not yet available
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
        };
        let away = home.clone();
        ffb_model::model::game::Game::new(home, away, ffb_model::enums::Rules::Bb2025)
    }

    #[test]
    fn hook_is_noop_returns_false() {
        let behaviour = TentaclesBehaviour::new();
        let mut game = test_game();
        assert!(!behaviour.execute_step_hook(&mut game));
    }

    #[test]
    fn execute_step_hook_returns_bool() {
        let behaviour = TentaclesBehaviour::new();
        let mut game = test_game();
        let _result: bool = behaviour.execute_step_hook(&mut game);
    }
}
