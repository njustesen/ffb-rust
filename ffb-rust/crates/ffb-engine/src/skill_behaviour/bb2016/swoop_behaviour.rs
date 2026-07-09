/// 1:1 translation of com.fumbbl.ffb.server.skillbehaviour.bb2016.SwoopBehaviour.
///
/// BB2016 SwoopBehaviour differs substantially from BB2025 SwoopBehaviour:
/// - BB2016: handles full TTM movement loop — scatters player 1 square per move step,
///   handles out-of-bounds (InjuryTypeCrowdPush, throw-in), landing on player
///   (InjuryTypeTTMHitPlayer), and continues until movement exhausted.
/// - BB2025: only rolls scatter direction and asks for a re-roll; actual movement delegated
///   to a different step.
///
/// **BB2016 `handleExecuteStepHook` logic:**
/// 1. Guard: `player.hasSkillProperty(ttmScattersInSingleDirection)`, else skip.
/// 2. Roll throw-in direction (D8) relative to current coordinateFrom → coordinateTo.
/// 3. Compute `coordinateTo = findScatterCoordinate(coordinateFrom, direction, 1)`.
/// 4. Publish `ReportSwoopPlayer(from, to, [direction], [roll])`.
/// 5. If `coordinateTo` out of bounds:
///    - Set player FALLING state.
///    - `handleInjury(InjuryTypeCrowdPush, thrownPlayer, from)`.
///    - If thrownPlayer has ball: publish THROW_IN + END_TURN.
///    - Publish THROWN_PLAYER_COORDINATE=null.
///    - `GOTO_LABEL(goToLabelOnFallDown)`.
/// 6. Else (in bounds):
///    - Move swooping player to `coordinateTo`, move ball if carrying.
///    - Increment `actingPlayer.currentMove`.
///    - If `currentMove < player.movementWithModifiers`:
///      → `UtilServerPlayerSwoop.updateSwoopSquares()` (still swooping).
///    - Else (landing):
///      → For each other player in square: apply InjuryTypeTTMHitPlayer, possibly END_TURN.
///      → `NEXT_STEP`.
///
/// TODO(hook-infra): `StepSwoopHookState` not yet ported.
///   When `mixed/ttm/StepSwoop` gains a typed hook state, add `SwoopStepModifier` here.
use crate::skill_behaviour::SkillBehaviour;
use ffb_model::model::game::Game;

pub struct SwoopBehaviour;

impl SwoopBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for SwoopBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for SwoopBehaviour {
    fn name(&self) -> &'static str { "SwoopBehaviour" }

    fn execute_step_hook(&self, _game: &mut Game) -> bool {
        // TODO(hook-infra): implement once StepSwoopHookState is ported
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use crate::step::framework::test_team;

    fn test_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2016)
    }

    #[test]
    fn name_returns_correct_string() {
        assert_eq!(SwoopBehaviour::new().name(), "SwoopBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        assert_eq!(SwoopBehaviour::default().name(), "SwoopBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        let b = SwoopBehaviour::new();
        let mut game = test_game();
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = SwoopBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }

    #[test]
    fn name_is_not_empty() {
        assert!(!SwoopBehaviour::new().name().is_empty());
    }
}
