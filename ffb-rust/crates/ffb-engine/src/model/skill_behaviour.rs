/// 1:1 translation of com.fumbbl.ffb.server.model.SkillBehaviour<T>.
///
/// Java: abstract class with registration pattern for step modifiers, player modifiers,
/// injury context modifications, and step overrides. Uses reflection and generics.
///
/// Rust: struct with owned collections + trait methods. Concrete skill behaviours
/// implement this struct via composition rather than inheritance.
use std::collections::HashMap;
use ffb_model::model::PlayerModifier;
use crate::step::framework::StepId;
use crate::model::step_modifier::StepModifierTrait;
use crate::injury::modification::InjuryContextModification;

/// Registration container for all server-side behaviours of a skill.
pub struct SkillBehaviour {
    player_modifiers: Vec<Box<dyn PlayerModifier>>,
    step_modifiers: Vec<Box<dyn StepModifierTrait>>,
    /// Java: Map<StepId, Class<? extends IStep>> steps — step overrides keyed by id.
    steps: HashMap<StepId, StepId>,
    /// Java: fInjuryContextModification — InjuryContextModification<? extends ModificationParams>
    injury_context_modification: Option<Box<dyn InjuryContextModification>>,
}

impl SkillBehaviour {
    pub fn new() -> Self {
        Self {
            player_modifiers: Vec::new(),
            step_modifiers: Vec::new(),
            steps: HashMap::new(),
            injury_context_modification: None,
        }
    }

    /// Java: registerModifier(StepModifier)
    pub fn register_step_modifier(&mut self, modifier: Box<dyn StepModifierTrait>) {
        self.step_modifiers.push(modifier);
    }

    /// Java: registerModifier(PlayerModifier)
    pub fn register_player_modifier(&mut self, modifier: Box<dyn PlayerModifier>) {
        self.player_modifiers.push(modifier);
    }

    /// Java: registerStep(StepId, Class<? extends IStep>)
    pub fn register_step(&mut self, step_id: StepId, replacement: StepId) {
        self.steps.insert(step_id, replacement);
    }

    /// Java: getStepModifiers()
    pub fn get_step_modifiers(&self) -> &[Box<dyn StepModifierTrait>] {
        &self.step_modifiers
    }

    /// Java: getPlayerModifiers()
    pub fn get_player_modifiers(&self) -> &[Box<dyn PlayerModifier>] {
        &self.player_modifiers
    }

    /// Java: getSteps()
    pub fn get_steps(&self) -> &HashMap<StepId, StepId> {
        &self.steps
    }

    /// Java: registerInjuryContextModification(InjuryContextModification)
    pub fn register_injury_context_modification(&mut self, m: Box<dyn InjuryContextModification>) {
        self.injury_context_modification = Some(m);
    }

    /// Java: getInjuryContextModification()
    pub fn get_injury_context_modification(&self) -> Option<&dyn InjuryContextModification> {
        self.injury_context_modification.as_deref()
    }

    /// Java: hasInjuryModifier(String injuryTypeName)
    pub fn has_injury_modifier(&self, injury_type_name: &str) -> bool {
        self.injury_context_modification
            .as_ref()
            .map(|m| m.is_valid_type(injury_type_name))
            .unwrap_or(false)
    }
}

impl Default for SkillBehaviour {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoopStepModifier;
    impl StepModifierTrait for NoopStepModifier {
        fn applies_to(&self, _: StepId) -> bool { false }
    }

    struct NoopPlayerModifier;
    impl PlayerModifier for NoopPlayerModifier {
        fn apply(&self, _: &mut ffb_model::model::player::Player) {}
    }

    #[test]
    fn new_has_empty_collections() {
        let sb = SkillBehaviour::new();
        assert!(sb.get_step_modifiers().is_empty());
        assert!(sb.get_player_modifiers().is_empty());
        assert!(sb.get_steps().is_empty());
        assert!(!sb.has_injury_modifier("Block"));
    }

    #[test]
    fn register_step_modifier() {
        let mut sb = SkillBehaviour::new();
        sb.register_step_modifier(Box::new(NoopStepModifier));
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn register_player_modifier() {
        let mut sb = SkillBehaviour::new();
        sb.register_player_modifier(Box::new(NoopPlayerModifier));
        assert_eq!(sb.get_player_modifiers().len(), 1);
    }

    #[test]
    fn register_step_override() {
        let mut sb = SkillBehaviour::new();
        sb.register_step(StepId::BlockRoll, StepId::GoForIt);
        assert_eq!(sb.get_steps().len(), 1);
        assert_eq!(sb.get_steps().get(&StepId::BlockRoll), Some(&StepId::GoForIt));
    }

    #[test]
    fn register_multiple_step_modifiers() {
        let mut sb = SkillBehaviour::new();
        sb.register_step_modifier(Box::new(NoopStepModifier));
        sb.register_step_modifier(Box::new(NoopStepModifier));
        assert_eq!(sb.get_step_modifiers().len(), 2);
    }

    #[test]
    fn register_injury_context_modification_wires_has_injury_modifier() {
        use crate::injury::modification::{InjuryContextModification, ModificationParams};
        use crate::injury::InjuryContext;
        use ffb_model::enums::ApothecaryMode;
        use ffb_model::model::game::Game;
        use ffb_model::util::rng::GameRng;
        use crate::step::framework::test_team;

        struct BlockModification;
        impl InjuryContextModification for BlockModification {
            fn skill_use(&self) -> ffb_model::model::SkillUse { ffb_model::model::SkillUse::ADD_ARMOUR_MODIFIER }
            fn valid_types(&self) -> &'static [&'static str] { &["Block"] }
            fn skill_id(&self) -> Option<u16> { None }
            fn set_skill_id(&mut self, _id: u16) {}
        }

        let mut sb = SkillBehaviour::new();
        assert!(!sb.has_injury_modifier("Block"));
        sb.register_injury_context_modification(Box::new(BlockModification));
        assert!(sb.has_injury_modifier("Block"));
        assert!(!sb.has_injury_modifier("Stab"));
    }

    #[test]
    fn get_injury_context_modification_returns_none_by_default() {
        let sb = SkillBehaviour::new();
        assert!(sb.get_injury_context_modification().is_none());
    }
}
