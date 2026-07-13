use crate::skill_behaviour::SkillBehaviour;
use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::StepModifierTrait;
use crate::step::framework::StepId;
use crate::skill_behaviour::registry::SkillRegistry;
use ffb_model::enums::SkillId;

/// Dauntless: can attempt to reduce the opponent ST for a block (multi-edition).
///
/// **This modifier is dead/unreachable code** (Phase AAH audit): it targets `StepId::Dauntless`,
/// which no step ever dispatches through `dispatch::execute_step_hooks`. The real logic for both
/// of Java's `DauntlessBehaviour` modifiers is ported directly into the step files instead — the
/// same "direct-in-step" pattern already established for Wrestle/Stab/DumpOff/Bombardier:
///
/// - **Modifier 1 (single-block, priority 2)** → `step/action/block/step_dauntless.rs`: the
///   die-roll/re-roll/Blind-Rage-silent-reroll logic, chained with Indomitable's own priority-3
///   modifier (`IndomitableBehaviour.java`, also ported directly into that same file — Rust has
///   no separate `IndomitableStepModifier`; see `skill_behaviour/mixed/indomitable_behaviour.rs`,
///   itself dead for the same reason).
/// - **Modifier 2 (multi-block)** → `step/mixed/multiblock/step_dauntless_multiple.rs`.
///
/// Left registered (harmless — Java's `StepModifier` shape is what registration validates against,
/// not reachability) rather than deleted, matching the precedent set for Wrestle/Stab/DumpOff.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.mixed.DauntlessBehaviour`.
pub struct DauntlessBehaviour;

impl DauntlessBehaviour {
    pub fn new() -> Self { Self }

    /// Register DauntlessStepModifier into the given SkillBehaviourContainer, then insert
    /// it into the SkillRegistry under SkillId::Dauntless.
    ///
    /// Java: DauntlessBehaviour constructor calls `registerModifier(new StepModifier<>() {...})`.
    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(DauntlessStepModifier));
        registry.register(SkillId::Dauntless, sb);
    }
}

impl Default for DauntlessBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for DauntlessBehaviour {
    fn name(&self) -> &'static str { "DauntlessBehaviour" }

    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

// ── DauntlessStepModifier ─────────────────────────────────────────────────────

pub struct DauntlessStepModifier;

impl StepModifierTrait for DauntlessStepModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::Dauntless }

    fn priority(&self) -> i32 { 2 }

    // Dead — real logic lives directly in step/action/block/step_dauntless.rs (see the module
    // doc comment above). Kept as a no-op to avoid dead duplicate logic.
    fn handle_execute_step(
        &self,
        _game: &mut ffb_model::model::game::Game,
        _rng: &mut ffb_model::util::rng::GameRng,
        _step_state: &mut dyn std::any::Any,
    ) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = DauntlessBehaviour::new();
        assert_eq!(b.name(), "DauntlessBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = DauntlessBehaviour::default();
        assert_eq!(b.name(), "DauntlessBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = DauntlessBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2025,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = DauntlessBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
    #[test]
    fn default_creates_instance_same_as_new() {
        let _a = DauntlessBehaviour::new();
        let _b = DauntlessBehaviour::default();
    }

    #[test]
    fn register_into_adds_step_modifier() {
        let mut reg = SkillRegistry::empty();
        DauntlessBehaviour::register_into(&mut reg);
        let sb = reg.get(SkillId::Dauntless).expect("Dauntless must be registered");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn step_modifier_applies_to_correct_step() {
        let m = DauntlessStepModifier;
        assert!(m.applies_to(StepId::Dauntless));
    }

    #[test]
    fn step_modifier_does_not_apply_to_wrong_step() {
        let m = DauntlessStepModifier;
        assert!(!m.applies_to(StepId::BlockRoll));
    }
}
