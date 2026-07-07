use crate::skill_behaviour::SkillBehaviour;

/// Bombardier: player may throw bombs instead of the ball.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.BombardierBehaviour`.
pub struct BombardierBehaviour;

impl BombardierBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for BombardierBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for BombardierBehaviour {
    fn name(&self) -> &'static str { "BombardierBehaviour" }

    /// Java `StepBombardier.handleExecuteStepHook` logic (condensed):
    ///
    /// 1. Check if the current action is `THROW_BOMB` or `HAIL_MARY_BOMB`.
    /// 2. Check if the game is already in a bomb-turn mode (BOMB_HOME, BOMB_AWAY,
    ///    BOMB_HOME_BLITZ, BOMB_AWAY_BLITZ); if so, skip — already set.
    /// 3. Otherwise, derive the appropriate bomb turn mode from the active team:
    ///    - Home team + THROW_BOMB   → BOMB_HOME
    ///    - Away team + THROW_BOMB   → BOMB_AWAY
    ///    - Home team + HAIL_MARY    → BOMB_HOME_BLITZ  (blitz slot used)
    ///    - Away team + HAIL_MARY    → BOMB_AWAY_BLITZ
    /// 4. Publish the new `TurnMode` and call `setNextAction(NEXT_STEP)`.
    ///
    /// TODO(hook-infra): step-local action check and turn-mode setting require
    ///                   access to the active step's action type and game.getTurnMode().
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = BombardierBehaviour::new();
        assert_eq!(b.name(), "BombardierBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = BombardierBehaviour::default();
        assert_eq!(b.name(), "BombardierBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = BombardierBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2016,
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
}
