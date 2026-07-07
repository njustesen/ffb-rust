use crate::skill_behaviour::SkillBehaviour;

/// Horns: +1 ST modifier when making a Blitz action (all editions).
///
/// Registers on StepHorns.
///
/// Java `execute_step_hook` logic:
/// 1. Set `StepState.usingHorns = player.hasSkill(Horns) && action == BLITZ`.
/// 2. If `usingHorns` is true:
///    - Mark the skill as used.
///    - Add a `ReportSkillUse` report entry.
/// 3. Always advance to `NEXT_STEP`. Returns `false`.
///
/// All step-local state fields are unavailable in the current Rust signature:
// TODO(hook-infra): step-specific state (StepState.usingHorns)
// TODO(hook-infra): step action check (StepState.playerAction == BLITZ)
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.common.HornsBehaviour`.
pub struct HornsBehaviour;

impl HornsBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for HornsBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for HornsBehaviour {
    fn name(&self) -> &'static str { "HornsBehaviour" }

    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = HornsBehaviour::new();
        assert_eq!(b.name(), "HornsBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = HornsBehaviour::default();
        assert_eq!(b.name(), "HornsBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = HornsBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2025,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = HornsBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
    #[test]
    fn default_creates_instance_same_as_new() {
        let _a = HornsBehaviour::new();
        let _b = HornsBehaviour::default();
    }
}
