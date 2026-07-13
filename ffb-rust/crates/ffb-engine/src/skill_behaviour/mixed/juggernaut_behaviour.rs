use crate::skill_behaviour::SkillBehaviour;

/// Juggernaut: removes Wrestle/Sidestep/Stand Firm from block results on a Blitz action
/// (multi-edition).
///
/// **This modifier is dead/unreachable code** (Phase AAH audit), same reason as
/// `bb2025::juggernaut_behaviour` — targets `StepId::Juggernaut`, which nothing dispatches. Java's
/// BB2016/BB2020 `JuggernautBehaviour.java` (byte-identical to the bb2025 copy modulo package/
/// imports) is ported directly into `step/action/block/step_juggernaut.rs`, one shared
/// edition-agnostic file used by all 3 rulesets (confirmed complete during Phase AAH's
/// investigation). Left registered rather than deleted, matching the Wrestle/Stab/DumpOff
/// precedent — though note this particular file has no `register_into`/`SkillRegistry` entry at
/// all (only referenced from the separate, unrelated `util_skill_behaviours.rs` informational
/// list), so there's nothing to unregister here either way.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.mixed.JuggernautBehaviour`.
pub struct JuggernautBehaviour;

impl JuggernautBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for JuggernautBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for JuggernautBehaviour {
    fn name(&self) -> &'static str { "JuggernautBehaviour" }

    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = JuggernautBehaviour::new();
        assert_eq!(b.name(), "JuggernautBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = JuggernautBehaviour::default();
        assert_eq!(b.name(), "JuggernautBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = JuggernautBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2025,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = JuggernautBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
    #[test]
    fn default_creates_instance_same_as_new() {
        let _a = JuggernautBehaviour::new();
        let _b = JuggernautBehaviour::default();
    }
}
