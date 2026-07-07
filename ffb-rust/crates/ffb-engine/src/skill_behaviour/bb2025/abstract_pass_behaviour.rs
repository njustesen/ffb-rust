use crate::skill_behaviour::SkillBehaviour;

/// Abstract base for pass-related skill behaviours (Pass, Hail Mary Pass).
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2025.AbstractPassBehaviour`.
pub struct AbstractPassBehaviour;

impl AbstractPassBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for AbstractPassBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for AbstractPassBehaviour {
    fn name(&self) -> &'static str { "AbstractPassBehaviour" }

    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        // Java StepModifier.handleExecuteStepHook: no-op in AbstractPassBehaviour base class;
        // real logic is delegated to concrete subclasses (PassBehaviour).
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
        let behaviour = AbstractPassBehaviour::new();
        let mut game = test_game();
        assert!(!behaviour.execute_step_hook(&mut game));
    }

    #[test]
    fn execute_step_hook_returns_bool() {
        let behaviour = AbstractPassBehaviour::new();
        let mut game = test_game();
        let _result: bool = behaviour.execute_step_hook(&mut game);
    }
}
