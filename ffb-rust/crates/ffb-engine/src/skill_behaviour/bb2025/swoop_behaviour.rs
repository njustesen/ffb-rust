use crate::skill_behaviour::SkillBehaviour;
use ffb_model::enums::SkillId;

/// Swoop: when landing after a throw, player may move up to 3 squares in any direction.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2025.SwoopBehaviour`.
pub struct SwoopBehaviour;

impl SwoopBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for SwoopBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for SwoopBehaviour {
    fn name(&self) -> &'static str { "SwoopBehaviour" }

    fn execute_step_hook(&self, game: &mut ffb_model::model::game::Game) -> bool {
        // Java StepModifier.handleExecuteStepHook: checks thrown player has Swoop;
        // the thrown player is tracked in StepState, not acting_player.
        let _has_skill = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill(SkillId::Swoop))
            .unwrap_or(false);
        // TODO(hook-infra): step-specific state access (StepState thrown player id,
        //   swoop landing movement) not yet available -- thrown player != acting_player
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
        let behaviour = SwoopBehaviour::new();
        let mut game = test_game();
        assert!(!behaviour.execute_step_hook(&mut game));
    }

    #[test]
    fn execute_step_hook_returns_bool() {
        let behaviour = SwoopBehaviour::new();
        let mut game = test_game();
        let _result: bool = behaviour.execute_step_hook(&mut game);
    }
}
