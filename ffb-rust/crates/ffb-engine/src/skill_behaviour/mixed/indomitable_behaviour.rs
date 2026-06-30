use crate::skill_behaviour::SkillBehaviour;

/// Indomitable: player may use this skill after a failed Dauntless roll (multi-edition).
///
/// Registers on StepDauntless.
///
/// Java `handleCommandHook` sets `StepState.status` to `SKILL_CHOICE_YES` or
/// `SKILL_CHOICE_NO` based on the coach's dialog response.
///
/// Java `execute_step_hook` logic:
/// 1. If `StepState.status == SKILL_CHOICE_YES`:
///    - Mark the skill as used.
///    - Publish `ReportId::DOUBLE_TARGET_STRENGTH` with value `true`.
///    - Add a `ReportIndomitable` report entry.
/// 2. If `StepState.status == WAITING_FOR_SKILL_USE`:
///    - Set `doNextStep = false` (hold — waiting for coach input).
/// 3. Otherwise continue normally.
///
/// All step-local state fields are unavailable in the current Rust signature:
// TODO(hook-infra): step-specific state (StepState.status)
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
}
