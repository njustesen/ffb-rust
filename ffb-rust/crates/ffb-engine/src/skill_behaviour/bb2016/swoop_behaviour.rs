use crate::skill_behaviour::SkillBehaviour;

/// Swoop: redirects a thrown player landing square for players with ttmScattersInSingleDirection.
/// Rolls throw-in direction, scatters player 1 square. Handles out-of-bounds (crowd push + throw-in)
/// and landing on a player (InjuryTypeTTMHitPlayer). If within movement range: continues swooping.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.SwoopBehaviour`.
pub struct SwoopBehaviour;

impl SwoopBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for SwoopBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for SwoopBehaviour {
    fn name(&self) -> &'static str { "SwoopBehaviour" }

    /// Java logic (handleExecuteStepHook):
    ///   1. Check if the thrown player has ttmScattersInSingleDirection; if not, skip.
    ///   2. Roll throw-in direction (D8).
    ///   3. Scatter the thrown player 1 square in that direction.
    ///   4. If landing square is out of bounds:
    ///      a. Apply InjuryTypeCrowdPush to the player.
    ///      b. Execute throw-in sequence from the boundary square.
    ///   5. If landing square is occupied by another player:
    ///      a. Apply InjuryTypeTTMHitPlayer to both players.
    ///   6. If player is still within their remaining movement range: continue swooping
    ///      (push another ScatterPlayer step / repeat).
    ///   7. Reads/writes: StepState.ttmScattersInSingleDirection, StepState.scatterDirection,
    ///      StepState.remainingMovement, StepState.thrownPlayerCoordinate.
    ///
    // TODO(hook-infra): step-specific state (StepState.ttmScattersInSingleDirection)
    // TODO(hook-infra): step-specific state (StepState.scatterDirection)
    // TODO(hook-infra): step-specific state (StepState.remainingMovement)
    // TODO(hook-infra): step-specific state (StepState.thrownPlayerCoordinate)
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = SwoopBehaviour::new();
        assert_eq!(b.name(), "SwoopBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = SwoopBehaviour::default();
        assert_eq!(b.name(), "SwoopBehaviour");
    }
}
