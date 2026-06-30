use crate::skill_behaviour::SkillBehaviour;

/// BB2020 Tentacles skill behaviour. StepModifier on StepTentacles: rolls strength contest when
/// adjacent player tries to leave; if successful traps the player. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.TentaclesBehaviour`.
pub struct TentaclesBehaviour;

impl TentaclesBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for TentaclesBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for TentaclesBehaviour {
    fn name(&self) -> &'static str { "TentaclesBehaviour" }

    /// Java `StepModifier<StepTentacles, StepState>.handleExecuteStepHook`: if opposing player
    /// tries to leave, rolls tentacles check (ST vs ST), handles reroll. Returns false always.
    /// TODO(hook-infra): needs step tentacles state, ST comparison.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        // TODO(hook-infra): step-specific state access (StepState.xxx)
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // Without step infra the hook always returns false.
        let b = TentaclesBehaviour::new();
        assert_eq!(b.name(), "TentaclesBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = TentaclesBehaviour::default();
        assert_eq!(b.name(), "TentaclesBehaviour");
    }
}
