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
///
/// `Send + Sync` required so the modifier can live in `Arc<SkillRegistry>`.
pub trait StepModifierTrait: Send + Sync {
    /// Java: `appliesTo(IStep step)` — true if this modifier targets the given step type.
    fn applies_to(&self, step_id: StepId) -> bool;

    /// Java: `getPriority()` — lower values are applied first.
    fn priority(&self) -> i32 { 0 }

    /// Java: `handleExecuteStepHook(step, state)` — called before step execution.
    ///
    /// `game` gives access to the full game state. `step_state` is a step-specific
    /// struct passed by the calling step; concrete modifiers downcast it to the expected
    /// type. Java passes an `Object state` for the same reason.
    ///
    /// Returns `true` if the step should stop further hook processing (modifier consumed it).
    fn handle_execute_step(
        &self,
        _game: &mut ffb_model::model::game::Game,
        _rng: &mut ffb_model::util::rng::GameRng,
        _step_state: &mut dyn std::any::Any,
    ) -> bool {
        false
    }
}

/// Comparator for sorting modifiers by priority (ascending).
/// Java: `StepModifier.Comparator`
pub fn sort_by_priority(modifiers: &mut [Box<dyn StepModifierTrait>]) {
    modifiers.sort_by_key(|m| m.priority());
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::util::rng::GameRng;

    struct TestModifier {
        target: StepId,
        priority: i32,
    }

    impl StepModifierTrait for TestModifier {
        fn applies_to(&self, step_id: StepId) -> bool { step_id == self.target }
        fn priority(&self) -> i32 { self.priority }
    }

    struct ReturningModifier {
        target: StepId,
        return_val: bool,
        call_count: std::sync::Arc<std::sync::Mutex<u32>>,
    }

    impl StepModifierTrait for ReturningModifier {
        fn applies_to(&self, step_id: StepId) -> bool { step_id == self.target }
        fn handle_execute_step(&self, _game: &mut ffb_model::model::game::Game, _rng: &mut ffb_model::util::rng::GameRng, _step_state: &mut dyn std::any::Any) -> bool {
            *self.call_count.lock().unwrap() += 1;
            self.return_val
        }
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
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let m = TestModifier { target: StepId::BlockRoll, priority: 0 };
        let mut game = ffb_model::model::game::Game::new(test_team("h", 0), test_team("a", 0), Rules::Bb2025);
        let mut state: () = ();
        assert!(!m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut state));
    }

    #[test]
    fn handle_execute_step_concrete_can_return_true() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let counter = std::sync::Arc::new(std::sync::Mutex::new(0u32));
        let m = ReturningModifier { target: StepId::BlockRoll, return_val: true, call_count: counter.clone() };
        let mut game = ffb_model::model::game::Game::new(test_team("h", 0), test_team("a", 0), Rules::Bb2025);
        let mut state: () = ();
        assert!(m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut state));
        assert_eq!(*counter.lock().unwrap(), 1);
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
