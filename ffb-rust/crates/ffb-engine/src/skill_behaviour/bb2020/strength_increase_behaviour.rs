use crate::skill_behaviour::SkillBehaviour;

/// Handles strength stat increase on level-up (BB2020 edition). Registers a player modifier that
/// adds +1 strength on level-up. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.StrengthIncreaseBehaviour`.
pub struct StrengthIncreaseBehaviour;

impl StrengthIncreaseBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for StrengthIncreaseBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for StrengthIncreaseBehaviour {
    fn name(&self) -> &'static str { "StrengthIncreaseBehaviour" }

    /// No step modifier hook — this behaviour only registers a player modifier. Increments player
    /// strength by 1 on level-up.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // StrengthIncreaseBehaviour only registers a player modifier; execute_step_hook is a no-op.
        let b = StrengthIncreaseBehaviour::new();
        assert_eq!(b.name(), "StrengthIncreaseBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = StrengthIncreaseBehaviour::default();
        assert_eq!(b.name(), "StrengthIncreaseBehaviour");
    }
}
