/// 1:1 translation of `com.fumbbl.ffb.server.skillbehaviour.StepHook`.
///
/// Java: a runtime annotation that marks a method as a step-hook for a given `HookPoint`.
/// Rust: an enum of hook points, plus a trait for types that respond to them.
///
/// The only defined hook point is `PASS_INTERCEPT` — used to inject intercept-eligibility
/// checks into the pass resolution sequence.

use ffb_model::enums::Rules;
use crate::step::framework::StepId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookPoint {
    /// Java: `StepHook.HookPoint.PASS_INTERCEPT`
    PassIntercept,
}

/// Rust analogue of the `@StepHook(HookPoint.XXX)` annotation.
/// Any type that implements this trait can declare which hook points it handles.
pub trait StepHookHandler {
    fn hook_points(&self) -> &[HookPoint];
}

/// 1:1 translation of `StepFactory.getSteps(HookPoint)`.
///
/// Java builds this table at `GameState` construction time by reflection-scanning every
/// `IStep` class for a `@StepHook` annotation, filtered by the class's `@RulesCollection` to
/// the current edition (`StepFactory.initialize()`). Rust has no reflection, so — following
/// this codebase's established substitution convention for reflection-based registries (see
/// `LogicPluginFactory`) — this is an explicit static table instead. It is exhaustive: across
/// the whole Java server, exactly two `IStep` classes carry `@StepHook`, each scoped to a
/// single edition: `StepSafeThrow` (`@RulesCollection(BB2016)`) and the nested
/// `CloudBursterBehaviour.StepCloudBurster` (`@RulesCollection(BB2020)`). BB2025 registers
/// nothing for `PASS_INTERCEPT` — not a gap, that's what the real Java `StepFactory` produces
/// for that edition too, since neither annotated class matches its `RulesCollection`.
pub fn hooked_steps(rules: Rules, hook_point: HookPoint) -> &'static [StepId] {
    match (rules, hook_point) {
        (Rules::Bb2016, HookPoint::PassIntercept) => &[StepId::SafeThrow],
        (Rules::Bb2020, HookPoint::PassIntercept) => &[StepId::CloudBurster],
        (Rules::Bb2025, HookPoint::PassIntercept) => &[],
        (Rules::Common, _) => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hooked_steps_bb2016_returns_safe_throw() {
        assert_eq!(hooked_steps(Rules::Bb2016, HookPoint::PassIntercept), &[StepId::SafeThrow]);
    }

    #[test]
    fn hooked_steps_bb2020_returns_cloud_burster() {
        assert_eq!(hooked_steps(Rules::Bb2020, HookPoint::PassIntercept), &[StepId::CloudBurster]);
    }

    #[test]
    fn hooked_steps_bb2025_returns_none() {
        assert!(hooked_steps(Rules::Bb2025, HookPoint::PassIntercept).is_empty());
    }

    #[test]
    fn hooked_steps_common_returns_none() {
        assert!(hooked_steps(Rules::Common, HookPoint::PassIntercept).is_empty());
    }

    #[test]
    fn hook_point_pass_intercept_eq() {
        assert_eq!(HookPoint::PassIntercept, HookPoint::PassIntercept);
    }

    #[test]
    fn hook_point_clone() {
        let h = HookPoint::PassIntercept;
        assert_eq!(h, h.clone());
    }

    struct MockHandler;
    impl StepHookHandler for MockHandler {
        fn hook_points(&self) -> &[HookPoint] { &[HookPoint::PassIntercept] }
    }

    #[test]
    fn step_hook_handler_reports_hook_points() {
        let h = MockHandler;
        assert_eq!(h.hook_points(), &[HookPoint::PassIntercept]);
    }

    #[test]
    fn hook_point_debug_contains_name() {
        assert!(format!("{:?}", HookPoint::PassIntercept).contains("PassIntercept"));
    }

    #[test]
    fn hook_point_is_hashable() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(HookPoint::PassIntercept);
        assert!(set.contains(&HookPoint::PassIntercept));
    }

    #[test]
    fn empty_hook_points_handler() {
        struct EmptyHandler;
        impl StepHookHandler for EmptyHandler {
            fn hook_points(&self) -> &[HookPoint] { &[] }
        }
        let h = EmptyHandler;
        assert!(h.hook_points().is_empty());
    }
}
