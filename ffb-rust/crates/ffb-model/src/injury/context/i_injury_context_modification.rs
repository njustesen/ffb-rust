/// 1:1 translation of `com.fumbbl.ffb.injury.context.IInjuryContextModification`.
///
/// A marker trait for types that can modify an injury context.
pub trait IInjuryContextModification {
    /// Java: `IInjuryContextModification.requiresConditionalReRollSkill()`
    fn requires_conditional_re_roll_skill(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockModification {
        requires: bool,
    }

    impl IInjuryContextModification for MockModification {
        fn requires_conditional_re_roll_skill(&self) -> bool {
            self.requires
        }
    }

    #[test]
    fn returns_true_when_set() {
        let m = MockModification { requires: true };
        assert!(m.requires_conditional_re_roll_skill());
    }

    #[test]
    fn returns_false_when_not_set() {
        let m = MockModification { requires: false };
        assert!(!m.requires_conditional_re_roll_skill());
    }
}
