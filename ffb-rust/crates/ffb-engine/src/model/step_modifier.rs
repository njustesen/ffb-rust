/// 1:1 translation of `com.fumbbl.ffb.server.model.StepModifier<T, V>`.
///
/// Java: abstract generic class that intercepts step execution. Concrete subclasses
/// handle `handleCommandHook(step, state, useSkillCommand)` and
/// `handleExecuteStepHook(step, state)`.
///
/// Rust: a trait that any step modifier implements. `priority()` determines
/// execution order (lower = earlier). `applies_to()` checks whether this modifier
/// is relevant to a given step ID.
use crate::step::framework::StepId;

/// Java: `StepCommandStatus` — what happens after a modifier handles a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepCommandStatus {
    /// Command fully handled — no further processing.
    Handled,
    /// Command not handled — pass through to the next modifier or step.
    NotHandled,
}

/// Rust analogue of Java's abstract `StepModifier<T, V>`.
pub trait StepModifierTrait {
    /// Java: `appliesTo(IStep step)` — true if this modifier targets the given step type.
    fn applies_to(&self, step_id: StepId) -> bool;

    /// Java: `getPriority()` — lower values are applied first.
    fn priority(&self) -> i32 { 0 }

    /// Java: `handleExecuteStepHook(step, state)` — called before step execution.
    /// Returns true if the step should be skipped (modifier handles it entirely).
    fn handle_execute_step(&mut self) -> bool { false }
}

/// Comparator for sorting modifiers by priority (ascending).
/// Java: `StepModifier.Comparator`
pub fn sort_by_priority(modifiers: &mut Vec<Box<dyn StepModifierTrait>>) {
    modifiers.sort_by_key(|m| m.priority());
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestModifier {
        target: StepId,
        priority: i32,
    }

    impl StepModifierTrait for TestModifier {
        fn applies_to(&self, step_id: StepId) -> bool { step_id == self.target }
        fn priority(&self) -> i32 { self.priority }
    }

    #[test]
    fn applies_to_matching_step() {
        let m = TestModifier { target: StepId::BlockRoll, priority: 0 };
        assert!(m.applies_to(StepId::BlockRoll));
    }

    #[test]
    fn does_not_apply_to_other_step() {
        let m = TestModifier { target: StepId::BlockRoll, priority: 0 };
        assert!(!m.applies_to(StepId::GoForIt));
    }

    #[test]
    fn handle_execute_step_default_false() {
        let mut m = TestModifier { target: StepId::BlockRoll, priority: 0 };
        assert!(!m.handle_execute_step());
    }

    #[test]
    fn sort_by_priority_orders_ascending() {
        let mut mods: Vec<Box<dyn StepModifierTrait>> = vec![
            Box::new(TestModifier { target: StepId::BlockRoll, priority: 5 }),
            Box::new(TestModifier { target: StepId::BlockRoll, priority: 1 }),
            Box::new(TestModifier { target: StepId::BlockRoll, priority: 3 }),
        ];
        sort_by_priority(&mut mods);
        let priorities: Vec<i32> = mods.iter().map(|m| m.priority()).collect();
        assert_eq!(priorities, vec![1, 3, 5]);
    }

    #[test]
    fn sort_by_priority_empty_vec_does_not_panic() {
        let mut mods: Vec<Box<dyn StepModifierTrait>> = vec![];
        sort_by_priority(&mut mods);
        assert!(mods.is_empty());
    }

    #[test]
    fn sort_by_priority_single_item_unchanged() {
        let mut mods: Vec<Box<dyn StepModifierTrait>> = vec![
            Box::new(TestModifier { target: StepId::BlockRoll, priority: 7 }),
        ];
        sort_by_priority(&mut mods);
        assert_eq!(mods[0].priority(), 7);
    }

    #[test]
    fn step_command_status_eq() {
        assert_eq!(StepCommandStatus::Handled, StepCommandStatus::Handled);
        assert_ne!(StepCommandStatus::Handled, StepCommandStatus::NotHandled);
    }
}
