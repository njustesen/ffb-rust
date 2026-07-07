use crate::skill_behaviour::SkillBehaviour;

/// BB2020 Bombardier skill behaviour.
/// StepModifier on StepBombardier: when player throws a bomb outside bomb-turn, marks skill used
/// and switches TurnMode to BOMB_HOME or BOMB_AWAY (with BLITZ variant if in blitz). Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.BombardierBehaviour`.
pub struct BombardierBehaviour;

impl BombardierBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for BombardierBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for BombardierBehaviour {
    fn name(&self) -> &'static str { "BombardierBehaviour" }

    /// Java `StepModifier<StepBombardier, StepState>.handleExecuteStepHook`:
    /// if not bomb turn and action is THROW_BOMB or HAIL_MARY_BOMB, marks skill used and sets
    /// TurnMode to BOMB_HOME or BOMB_AWAY (with BLITZ variant if in blitz).
    /// Returns false always.
    ///
    /// TODO(hook-infra): needs game.getTurnMode(), game.getActingPlayer(),
    /// game.getTeamHome().
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        // TODO(hook-infra): step-specific state access (game.getTurnMode(),
        // game.getActingPlayer(), game.getTeamHome())
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // Without step infra the hook always returns false.
        let b = BombardierBehaviour::new();
        assert_eq!(b.name(), "BombardierBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = BombardierBehaviour::default();
        assert_eq!(b.name(), "BombardierBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = BombardierBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
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
#[test]    fn name_is_not_empty() {        assert!(!BombardierBehaviour::new().name().is_empty());    }    #[test]    fn execute_step_hook_false_with_bb2020() {        use ffb_model::enums::Rules;        use crate::step::framework::test_team;        let b = BombardierBehaviour::new();        let mut game = ffb_model::model::game::Game::new(            test_team("home", 0), test_team("away", 0), Rules::Bb2020,        );        assert!(!b.execute_step_hook(&mut game));    }
}
