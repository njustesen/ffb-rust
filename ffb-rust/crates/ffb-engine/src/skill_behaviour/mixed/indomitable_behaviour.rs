use crate::skill_behaviour::SkillBehaviour;

/// Indomitable: player may use this skill after a successful Dauntless roll (multi-edition),
/// doubling the target's effective strength for the block.
///
/// **This modifier is dead/unreachable code** (Phase AAH audit) — it has no `register_into` at
/// all and is never inserted into `SkillRegistry`. In Java, `IndomitableBehaviour` registers a
/// priority-3 `StepModifier<StepDauntless, StepState>` chained onto the *same* step as Dauntless's
/// own priority-2 modifier (single-block path only — the multi-block equivalent is
/// `StepDoubleStrength`/`step/mixed/multiblock/step_double_strength.rs`, a distinct mechanism).
/// This single-block chain (headless-simplified: auto-declines when undecided rather than waiting
/// on a live dialog) is now ported directly into
/// `step/action/block/step_dauntless.rs::resolve_indomitable`, matching the "direct-in-step"
/// pattern already established for Wrestle/Stab/DumpOff/Bombardier/Dauntless itself.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.mixed.IndomitableBehaviour`.
pub struct IndomitableBehaviour;

impl IndomitableBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for IndomitableBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for IndomitableBehaviour {
    fn name(&self) -> &'static str { "IndomitableBehaviour" }

    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = IndomitableBehaviour::new();
        assert_eq!(b.name(), "IndomitableBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = IndomitableBehaviour::default();
        assert_eq!(b.name(), "IndomitableBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = IndomitableBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2025,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = IndomitableBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
    #[test]
    fn default_creates_instance_same_as_new() {
        let _a = IndomitableBehaviour::new();
        let _b = IndomitableBehaviour::default();
    }
}
