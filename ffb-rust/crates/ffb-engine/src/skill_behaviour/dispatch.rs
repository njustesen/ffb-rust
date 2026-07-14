/// 1:1 translation of GameState.executeStepHooks(IStep step, Object state).
///
/// Java: iterates all registered skills, collects step modifiers that `appliesTo(step)`,
/// sorts by priority (ascending), runs each — returns true on the first modifier that
/// returns true (stops processing).
///
/// Rust: reads the edition-specific SkillRegistry (lazily initialised per-edition),
/// performs the same collect→sort→dispatch loop.
use std::any::Any;
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;
use crate::step::framework::StepId;
use super::registry::{SkillRegistry, registry_for};

/// 1:1 translation of GameState.executeStepHooks(IStep step, Object state).
///
/// `step_state` is a step-specific struct passed from the calling step (mirrors Java's
/// `Object state` argument). Concrete modifiers downcast it with `downcast_mut::<T>()`.
///
/// Returns `true` if a modifier consumed the step (stop processing); `false` otherwise.
pub fn execute_step_hooks(
    game: &mut Game,
    rng: &mut ffb_model::util::rng::GameRng,
    step_id: StepId,
    step_state: &mut dyn Any,
) -> bool {
    let registry = registry_for(game.rules);
    execute_step_hooks_with_registry(&registry, game, rng, step_id, step_state)
}

/// Variant that accepts an explicit registry — used in tests where we want to inject a
/// custom registry without touching the global statics.
pub fn execute_step_hooks_with_registry(
    registry: &SkillRegistry,
    game: &mut Game,
    rng: &mut ffb_model::util::rng::GameRng,
    step_id: StepId,
    step_state: &mut dyn Any,
) -> bool {
    // Phase 1: collect (skill_id, modifier_index, priority) for applicable modifiers.
    // We store indices rather than references to avoid lifetime conflicts when we
    // later pass `game` to handle_execute_step while still reading from the registry.
    let mut applicable: Vec<(SkillId, usize, i32)> = registry
        .behaviours_iter()
        .flat_map(|(skill_id, sb)| {
            sb.get_step_modifiers()
                .iter()
                .enumerate()
                .filter(|(_, m)| m.applies_to(step_id))
                .map(|(idx, m)| (*skill_id, idx, m.priority()))
                .collect::<Vec<_>>()
        })
        .collect();

    // Phase 2: sort by priority ascending (Java: StepModifier.Comparator).
    applicable.sort_by_key(|(_, _, priority)| *priority);

    // Phase 3: dispatch — call each modifier in priority order.
    for (skill_id, idx, _) in applicable {
        // Re-fetch the modifier from the registry. The borrow from `registry.get(...)` is
        // released after the `if let` block, so `game` can be passed mutably inside.
        let stop_processing = {
            let modifier = registry
                .get(skill_id)
                .and_then(|sb| sb.get_step_modifiers().get(idx))
                .map(|m| m.as_ref());

            if let Some(modifier) = modifier {
                modifier.handle_execute_step(game, rng, step_state)
            } else {
                false
            }
        };

        if stop_processing {
            return true;
        }
    }

    false
}

/// 1:1 translation of `AbstractStep.handleSkillCommand(ClientCommandUseSkill, Object state)`.
///
/// Unlike `execute_step_hooks` (which scans every registered skill), Java only looks at the
/// modifiers registered under the *specific* skill named by the command, filtered to those
/// `appliesTo` this step, and calls `handleCommand` on each in registration order — the last
/// non-null status wins.
pub fn handle_skill_command(
    game: &mut Game,
    step_id: StepId,
    step_state: &mut dyn Any,
    skill_id: SkillId,
    skill_used: bool,
) -> crate::step::framework::StepCommandStatus {
    let registry = registry_for(game.rules);
    handle_skill_command_with_registry(&registry, game, step_id, step_state, skill_id, skill_used)
}

/// Variant that accepts an explicit registry — used in tests.
pub fn handle_skill_command_with_registry(
    registry: &SkillRegistry,
    game: &mut Game,
    step_id: StepId,
    step_state: &mut dyn Any,
    skill_id: SkillId,
    skill_used: bool,
) -> crate::step::framework::StepCommandStatus {
    use crate::step::framework::StepCommandStatus;

    let mut command_status = StepCommandStatus::UnhandledCommand;
    let modifier_count = registry.get(skill_id).map(|sb| sb.get_step_modifiers().len()).unwrap_or(0);

    for idx in 0..modifier_count {
        let applies = registry.get(skill_id)
            .and_then(|sb| sb.get_step_modifiers().get(idx))
            .map(|m| m.applies_to(step_id))
            .unwrap_or(false);
        if !applies {
            continue;
        }

        let new_status = {
            let modifier = registry.get(skill_id)
                .and_then(|sb| sb.get_step_modifiers().get(idx))
                .map(|m| m.as_ref());
            modifier.map(|m| m.handle_command(game, step_state, skill_id, skill_used))
        };

        if let Some(status) = new_status {
            command_status = status;
        }
    }

    command_status
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
    use crate::model::step_modifier::StepModifierTrait;
    use crate::step::framework::test_team;
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    // ── helpers ───────────────────────────────────────────────────────────────

    struct CountingModifier {
        target: StepId,
        priority: i32,
        return_val: bool,
        // We use a shared counter via `std::sync::Arc<Mutex>` so we can observe calls.
        counter: std::sync::Arc<std::sync::Mutex<u32>>,
    }

    impl StepModifierTrait for CountingModifier {
        fn applies_to(&self, id: StepId) -> bool { id == self.target }
        fn priority(&self) -> i32 { self.priority }
        fn handle_execute_step(&self, _game: &mut Game, _rng: &mut ffb_model::util::rng::GameRng, _state: &mut dyn Any) -> bool {
            *self.counter.lock().unwrap() += 1;
            self.return_val
        }
    }

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn make_registry_with(modifiers: Vec<(SkillId, Box<dyn StepModifierTrait>)>) -> SkillRegistry {
        let mut reg = SkillRegistry::empty();
        for (skill_id, modifier) in modifiers {
            let mut sb = SbContainer::new();
            sb.register_step_modifier(modifier);
            reg.register(skill_id, sb);
        }
        reg
    }

    // ── tests ─────────────────────────────────────────────────────────────────

    #[test]
    fn no_modifiers_returns_false() {
        let reg = SkillRegistry::empty();
        let mut game = make_game();
        let mut state: () = ();
        assert!(!execute_step_hooks_with_registry(&reg, &mut game, &mut GameRng::new(0), StepId::Horns, &mut state));
    }

    #[test]
    fn modifier_not_applicable_returns_false() {
        let counter = std::sync::Arc::new(std::sync::Mutex::new(0u32));
        let reg = make_registry_with(vec![(SkillId::Horns, Box::new(CountingModifier {
            target: StepId::BlockRoll,
            priority: 0,
            return_val: false,
            counter: counter.clone(),
        }))]);
        let mut game = make_game();
        let mut state: () = ();
        // Dispatch for StepId::Horns — modifier targets BlockRoll so should not fire
        assert!(!execute_step_hooks_with_registry(&reg, &mut game, &mut GameRng::new(0), StepId::Horns, &mut state));
        assert_eq!(*counter.lock().unwrap(), 0);
    }

    #[test]
    fn applicable_modifier_is_called() {
        let counter = std::sync::Arc::new(std::sync::Mutex::new(0u32));
        let reg = make_registry_with(vec![(SkillId::Horns, Box::new(CountingModifier {
            target: StepId::Horns,
            priority: 0,
            return_val: false,
            counter: counter.clone(),
        }))]);
        let mut game = make_game();
        let mut state: () = ();
        execute_step_hooks_with_registry(&reg, &mut game, &mut GameRng::new(0), StepId::Horns, &mut state);
        assert_eq!(*counter.lock().unwrap(), 1);
    }

    #[test]
    fn modifier_returning_true_stops_processing() {
        let c1 = std::sync::Arc::new(std::sync::Mutex::new(0u32));
        let c2 = std::sync::Arc::new(std::sync::Mutex::new(0u32));
        // Two modifiers, first returns true
        let mut reg = SkillRegistry::empty();
        let mut sb1 = SbContainer::new();
        sb1.register_step_modifier(Box::new(CountingModifier {
            target: StepId::Horns, priority: 1, return_val: true, counter: c1.clone(),
        }));
        reg.register(SkillId::Horns, sb1);
        let mut sb2 = SbContainer::new();
        sb2.register_step_modifier(Box::new(CountingModifier {
            target: StepId::Horns, priority: 2, return_val: false, counter: c2.clone(),
        }));
        reg.register(SkillId::Wrestle, sb2);

        let mut game = make_game();
        let mut state: () = ();
        let result = execute_step_hooks_with_registry(&reg, &mut game, &mut GameRng::new(0), StepId::Horns, &mut state);
        assert!(result, "should return true when first modifier returns true");
        assert_eq!(*c1.lock().unwrap(), 1, "first modifier should be called");
        assert_eq!(*c2.lock().unwrap(), 0, "second modifier should NOT be called after stop");
    }

    #[test]
    fn modifiers_run_in_priority_order() {
        let order = std::sync::Arc::new(std::sync::Mutex::new(Vec::<i32>::new()));
        struct OrderedModifier {
            target: StepId,
            priority: i32,
            order: std::sync::Arc<std::sync::Mutex<Vec<i32>>>,
        }
        impl StepModifierTrait for OrderedModifier {
            fn applies_to(&self, id: StepId) -> bool { id == self.target }
            fn priority(&self) -> i32 { self.priority }
            fn handle_execute_step(&self, _: &mut Game, _: &mut ffb_model::util::rng::GameRng, _: &mut dyn Any) -> bool {
                self.order.lock().unwrap().push(self.priority);
                false
            }
        }

        let mut reg = SkillRegistry::empty();
        for (skill_id, prio) in [(SkillId::Wrestle, 3i32), (SkillId::Horns, 1), (SkillId::Dodge, 2)] {
            let mut sb = SbContainer::new();
            sb.register_step_modifier(Box::new(OrderedModifier {
                target: StepId::Horns,
                priority: prio,
                order: order.clone(),
            }));
            reg.register(skill_id, sb);
        }

        let mut game = make_game();
        let mut state: () = ();
        execute_step_hooks_with_registry(&reg, &mut game, &mut GameRng::new(0), StepId::Horns, &mut state);
        let recorded = order.lock().unwrap().clone();
        assert_eq!(recorded, vec![1, 2, 3], "modifiers must run lowest-priority-first");
    }

    #[test]
    fn step_state_is_passed_to_modifier() {
        struct StateMutatingModifier;
        impl StepModifierTrait for StateMutatingModifier {
            fn applies_to(&self, id: StepId) -> bool { id == StepId::Horns }
            fn handle_execute_step(&self, _: &mut Game, _: &mut ffb_model::util::rng::GameRng, state: &mut dyn Any) -> bool {
                if let Some(val) = state.downcast_mut::<u32>() {
                    *val = 42;
                }
                false
            }
        }

        let mut reg = SkillRegistry::empty();
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(StateMutatingModifier));
        reg.register(SkillId::Horns, sb);

        let mut game = make_game();
        let mut state: u32 = 0;
        execute_step_hooks_with_registry(&reg, &mut game, &mut GameRng::new(0), StepId::Horns, &mut state);
        assert_eq!(state, 42, "modifier must be able to mutate step_state via downcast");
    }

    // ── handle_skill_command tests ──────────────────────────────────────────────

    struct CommandRecordingModifier {
        target: StepId,
        status: crate::step::framework::StepCommandStatus,
        calls: std::sync::Arc<std::sync::Mutex<Vec<(SkillId, bool)>>>,
    }

    impl StepModifierTrait for CommandRecordingModifier {
        fn applies_to(&self, id: StepId) -> bool { id == self.target }
        fn handle_command(
            &self, _game: &mut Game, _state: &mut dyn Any, skill_id: SkillId, skill_used: bool,
        ) -> crate::step::framework::StepCommandStatus {
            self.calls.lock().unwrap().push((skill_id, skill_used));
            self.status
        }
    }

    #[test]
    fn handle_skill_command_no_modifiers_is_unhandled() {
        let reg = SkillRegistry::empty();
        let mut game = make_game();
        let mut state: () = ();
        let status = handle_skill_command_with_registry(
            &reg, &mut game, StepId::ThrowTeamMate, &mut state, SkillId::TheBallista, true,
        );
        assert_eq!(status, crate::step::framework::StepCommandStatus::UnhandledCommand);
    }

    #[test]
    fn handle_skill_command_only_looks_at_the_named_skill() {
        let calls = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let mut reg = SkillRegistry::empty();
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(CommandRecordingModifier {
            target: StepId::ThrowTeamMate,
            status: crate::step::framework::StepCommandStatus::ExecuteStep,
            calls: calls.clone(),
        }));
        reg.register(SkillId::TheBallista, sb);
        // A modifier registered under a different skill must never be consulted.
        let mut sb2 = SbContainer::new();
        sb2.register_step_modifier(Box::new(CommandRecordingModifier {
            target: StepId::ThrowTeamMate,
            status: crate::step::framework::StepCommandStatus::SkipStep,
            calls: calls.clone(),
        }));
        reg.register(SkillId::Horns, sb2);

        let mut game = make_game();
        let mut state: () = ();
        let status = handle_skill_command_with_registry(
            &reg, &mut game, StepId::ThrowTeamMate, &mut state, SkillId::TheBallista, true,
        );
        assert_eq!(status, crate::step::framework::StepCommandStatus::ExecuteStep);
        assert_eq!(*calls.lock().unwrap(), vec![(SkillId::TheBallista, true)]);
    }

    #[test]
    fn handle_skill_command_skips_modifiers_not_applicable_to_step() {
        let calls = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let mut reg = SkillRegistry::empty();
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(CommandRecordingModifier {
            target: StepId::HailMaryPass,
            status: crate::step::framework::StepCommandStatus::ExecuteStep,
            calls: calls.clone(),
        }));
        reg.register(SkillId::TheBallista, sb);

        let mut game = make_game();
        let mut state: () = ();
        let status = handle_skill_command_with_registry(
            &reg, &mut game, StepId::ThrowTeamMate, &mut state, SkillId::TheBallista, true,
        );
        assert_eq!(status, crate::step::framework::StepCommandStatus::UnhandledCommand);
        assert!(calls.lock().unwrap().is_empty());
    }

    #[test]
    fn handle_skill_command_passes_skill_used_flag_through() {
        let calls = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let mut reg = SkillRegistry::empty();
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(CommandRecordingModifier {
            target: StepId::ThrowTeamMate,
            status: crate::step::framework::StepCommandStatus::ExecuteStep,
            calls: calls.clone(),
        }));
        reg.register(SkillId::TheBallista, sb);

        let mut game = make_game();
        let mut state: () = ();
        handle_skill_command_with_registry(
            &reg, &mut game, StepId::ThrowTeamMate, &mut state, SkillId::TheBallista, false,
        );
        assert_eq!(*calls.lock().unwrap(), vec![(SkillId::TheBallista, false)]);
    }
}
