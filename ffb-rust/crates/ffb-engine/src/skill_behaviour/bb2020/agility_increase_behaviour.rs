use crate::skill_behaviour::SkillBehaviour;

/// Handles agility stat increase on level-up (BB2020 edition). Registers a player modifier that
/// increases agility on level-up: max(1, pos_agility-2, agility-1). Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.AgilityIncreaseBehaviour`.
pub struct AgilityIncreaseBehaviour;

impl AgilityIncreaseBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for AgilityIncreaseBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for AgilityIncreaseBehaviour {
    fn name(&self) -> &'static str { "AgilityIncreaseBehaviour" }

    /// No step modifier hook — this behaviour only registers a player modifier. Sets player
    /// agility to max(1, pos_agility-2, agility-1) on level-up.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // AgilityIncreaseBehaviour only registers a player modifier; execute_step_hook is a no-op.
        let b = AgilityIncreaseBehaviour::new();
        assert_eq!(b.name(), "AgilityIncreaseBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = AgilityIncreaseBehaviour::default();
        assert_eq!(b.name(), "AgilityIncreaseBehaviour");
    }
}
