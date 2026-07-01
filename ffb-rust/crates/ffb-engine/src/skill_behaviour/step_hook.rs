/// 1:1 translation of `com.fumbbl.ffb.server.skillbehaviour.StepHook`.
///
/// Java: a runtime annotation that marks a method as a step-hook for a given `HookPoint`.
/// Rust: an enum of hook points, plus a trait for types that respond to them.
///
/// The only defined hook point is `PASS_INTERCEPT` — used to inject intercept-eligibility
/// checks into the pass resolution sequence.

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
