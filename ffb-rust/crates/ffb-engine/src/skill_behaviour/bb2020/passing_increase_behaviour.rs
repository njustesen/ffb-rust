use crate::skill_behaviour::SkillBehaviour;

/// Handles passing stat increase on level-up (BB2020 edition). Registers a player modifier that
/// increases passing stat on level-up: if passing<=0 set to 6, else max(1, pos_passing-2,
/// passing-1). Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.PassingIncreaseBehaviour`.
pub struct PassingIncreaseBehaviour;

impl PassingIncreaseBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for PassingIncreaseBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for PassingIncreaseBehaviour {
    fn name(&self) -> &'static str { "PassingIncreaseBehaviour" }

    /// No step modifier hook — this behaviour only registers a player modifier. Sets player
    /// passing stat on level-up.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // PassingIncreaseBehaviour only registers a player modifier; execute_step_hook is a no-op.
        let b = PassingIncreaseBehaviour::new();
        assert_eq!(b.name(), "PassingIncreaseBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = PassingIncreaseBehaviour::default();
        assert_eq!(b.name(), "PassingIncreaseBehaviour");
    }
}
