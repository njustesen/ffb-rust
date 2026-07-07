use crate::skill_behaviour::SkillBehaviour;

/// Pass: +1 modifier on passing rolls; also handles Throw Team-Mate and Hail Mary Pass variants.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.PassBehaviour`.
///
/// Player modifier: +1 on all pass rolls (applied elsewhere via player-modifier hooks).
pub struct PassBehaviour;

impl PassBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for PassBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for PassBehaviour {
    fn name(&self) -> &'static str { "PassBehaviour" }

    /// Java `StepModifier.handleExecuteStepHook` logic — three distinct step contexts:
    ///
    /// **StepPass context:**
    /// 1. If the coach selects to use the Pass skill re-roll via dialog:
    ///    set `StepState.reRollSource` from the Pass skill entry.
    ///
    /// **StepThrowTeamMate context:**
    /// 1. Set `StepState.reRollSource` to `ReRollSource::THROW_TEAM_MATE`.
    ///
    /// **StepHailMaryPass context (full roll logic):**
    /// 1. Roll a D6 for the Hail Mary Pass.
    /// 2. On a result of 1: set pass result to `FUMBLE`.
    /// 3. On any other result: set pass result to `INACCURATE`.
    /// 4. If result is FUMBLE: ask the Pass skill for a re-roll dialog.
    ///    - On re-roll confirmation: repeat the roll.
    ///
    /// All step-local state fields are unavailable in the current Rust signature:
    // TODO(hook-infra): step-specific state (StepState.reRollSource)
    // TODO(hook-infra): step-specific pass roll fields (pass result, fumble flag)
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = PassBehaviour::new();
        assert_eq!(b.name(), "PassBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = PassBehaviour::default();
        assert_eq!(b.name(), "PassBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = PassBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2016,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = PassBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
#[test]    fn name_is_not_empty() {        assert!(!PassBehaviour::new().name().is_empty());    }    #[test]    fn execute_step_hook_false_with_bb2020() {        use ffb_model::enums::Rules;        use crate::step::framework::test_team;        let b = PassBehaviour::new();        let mut game = ffb_model::model::game::Game::new(            test_team("home", 0), test_team("away", 0), Rules::Bb2020,        );        assert!(!b.execute_step_hook(&mut game));    }
}
