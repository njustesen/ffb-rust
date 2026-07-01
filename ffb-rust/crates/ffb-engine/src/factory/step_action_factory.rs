/// 1:1 translation of `com.fumbbl.ffb.server.factory.StepActionFactory`.
///
/// Maps Java camelCase step-action names to the Rust `StepAction` enum.
/// Java iterates `StepAction.values()` and matches by `getName()`.
/// Rust uses an explicit match for O(1) lookup.
use crate::step::framework::StepAction;

pub struct StepActionFactory;

impl StepActionFactory {
    pub fn new() -> Self { Self }

    /// Java: `forName(String pName)` — case-insensitive lookup by Java step-action name.
    /// Returns `None` for unknown names (Java returns `null`).
    pub fn for_name(name: &str) -> Option<StepAction> {
        match name.to_ascii_lowercase().as_str() {
            "continue"              => Some(StepAction::Continue),
            "nextstep"              => Some(StepAction::NextStep),
            "repeat"                => Some(StepAction::Repeat),
            "gotolabel"             => Some(StepAction::GotoLabel),
            "nextstepandrepeat"     => Some(StepAction::NextStepAndRepeat),
            "gotolabelandrepeat"    => Some(StepAction::GotoLabelAndRepeat),
            _                       => None,
        }
    }

    /// Reverse mapping: Java step-action name for a given `StepAction`.
    pub fn name_for(action: StepAction) -> &'static str {
        match action {
            StepAction::Continue           => "continue",
            StepAction::NextStep           => "nextStep",
            StepAction::Repeat             => "repeat",
            StepAction::GotoLabel          => "gotoLabel",
            StepAction::NextStepAndRepeat  => "nextStepAndRepeat",
            StepAction::GotoLabelAndRepeat => "gotoLabelAndRepeat",
        }
    }
}

impl Default for StepActionFactory {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_continue() {
        assert_eq!(StepActionFactory::for_name("continue"), Some(StepAction::Continue));
    }

    #[test]
    fn for_name_next_step() {
        assert_eq!(StepActionFactory::for_name("nextStep"), Some(StepAction::NextStep));
    }

    #[test]
    fn for_name_case_insensitive() {
        assert_eq!(StepActionFactory::for_name("NEXTSTEP"), Some(StepAction::NextStep));
    }

    #[test]
    fn for_name_goto_label() {
        assert_eq!(StepActionFactory::for_name("gotoLabel"), Some(StepAction::GotoLabel));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(StepActionFactory::for_name("notAnAction"), None);
    }

    #[test]
    fn name_for_roundtrip_continue() {
        let name = StepActionFactory::name_for(StepAction::Continue);
        assert_eq!(StepActionFactory::for_name(name), Some(StepAction::Continue));
    }

    #[test]
    fn name_for_roundtrip_goto_label() {
        let name = StepActionFactory::name_for(StepAction::GotoLabel);
        assert_eq!(StepActionFactory::for_name(name), Some(StepAction::GotoLabel));
    }
}
