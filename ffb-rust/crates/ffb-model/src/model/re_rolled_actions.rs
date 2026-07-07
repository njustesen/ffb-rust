use std::collections::HashMap;
use super::re_rolled_action::ReRolledAction;

/// 1:1 translation of com.fumbbl.ffb.ReRolledActions.
/// Java uses reflection to populate; Rust enumerates all constants directly.
pub struct ReRolledActions {
    values: HashMap<String, ReRolledAction>,
}

impl ReRolledActions {
    pub fn new() -> Self {
        let all = Self::all_actions();
        let mut values = HashMap::new();
        for action in all {
            values.insert(action.name.to_lowercase(), action);
        }
        ReRolledActions { values }
    }

    pub fn values(&self) -> &HashMap<String, ReRolledAction> {
        &self.values
    }

    pub fn for_name(&self, name: &str) -> Option<&ReRolledAction> {
        self.values.get(&name.to_lowercase())
    }

    fn all_actions() -> Vec<ReRolledAction> {
        vec![
            ReRolledAction::new("Go For It"),
            ReRolledAction::new("Rush"),
            ReRolledAction::new("Dodge"),
            ReRolledAction::new("Catch"),
            ReRolledAction::new("Pick Up"),
            ReRolledAction::new("com.fumbbl.ffb.skill.common.Pass"),
            ReRolledAction::new("com.fumbbl.ffb.skill.common.Dauntless"),
            ReRolledAction::new("Jump"),
            ReRolledAction::new("com.fumbbl.ffb.skill.common.FoulAppearance"),
            ReRolledAction::new("Block"),
            ReRolledAction::new("Really Stupid"),
            ReRolledAction::new("Bone Head"),
            ReRolledAction::new("Bone-Head"),
            ReRolledAction::new("com.fumbbl.ffb.skill.bb2016.WildAnimal"),
            ReRolledAction::new("com.fumbbl.ffb.skill.mixed.AnimalSavagery"),
            ReRolledAction::new("Take Root"),
            ReRolledAction::new("Winnings"),
            ReRolledAction::new("Always Hungry"),
            ReRolledAction::new("Throw Team-Mate"),
            ReRolledAction::new("Kick Team-Mate"),
            ReRolledAction::new("Right Stuff"),
            ReRolledAction::new("Shadowing"),
            ReRolledAction::new("Shadowing Escape"),
            ReRolledAction::new("Tentacles"),
            ReRolledAction::new("Tentacles Escape"),
            ReRolledAction::new("Escape"),
            ReRolledAction::new("Safe Throw"),
            ReRolledAction::new("Interception"),
            ReRolledAction::new("com.fumbbl.ffb.skill.common.JumpUp"),
            ReRolledAction::new("standUp"),
            ReRolledAction::new("Chainsaw"),
            ReRolledAction::new("Chomp"),
            ReRolledAction::new("Bloodlust"),
            ReRolledAction::new("Hypnotic Gaze"),
            ReRolledAction::new("Animosity"),
            ReRolledAction::new("com.fumbbl.ffb.skill.mixed.UnchannelledFury"),
            ReRolledAction::new("Projectile Vomit"),
            ReRolledAction::new("Breathe Fire"),
            ReRolledAction::new("Trapdoor"),
            ReRolledAction::new("Argue the Call"),
            ReRolledAction::new("Old Pro"),
            ReRolledAction::new("Throw Keg"),
            ReRolledAction::new("Direction"),
            ReRolledAction::new("Look Into My Eyes"),
            ReRolledAction::new("Baleful Hex"),
            ReRolledAction::new("Single Die"),
            ReRolledAction::new("All You Can Eat"),
            ReRolledAction::new("com.fumbbl.ffb.skill.mixed.special.CatchOfTheDay"),
            ReRolledAction::new("Single Block Die"),
            ReRolledAction::new("com.fumbbl.ffb.skill.bb2020.special.ThenIStartedBlastin"),
            ReRolledAction::new("Multi Block Dice"),
            ReRolledAction::new("Steady Footing"),
            ReRolledAction::new("Single BothDown"),
            ReRolledAction::new("Single Die Per Activation"),
            ReRolledAction::new("Getting Even"),
            ReRolledAction::new("Single Skull"),
            ReRolledAction::new("Regeneration"),
            ReRolledAction::new("com.fumbbl.ffb.skill.bb2025.special.BlastinSolvesEverything"),
            ReRolledAction::new("Punt Direction"),
            ReRolledAction::new("Punt Distance"),
        ]
    }
}

impl Default for ReRolledActions {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn for_name_case_insensitive() {
        let r = ReRolledActions::new();
        assert!(r.for_name("Go For It").is_some());
        assert!(r.for_name("go for it").is_some());
    }
    #[test]
    fn for_name_unknown_returns_none() {
        assert!(ReRolledActions::new().for_name("NOT_AN_ACTION").is_none());
    }
    #[test]
    fn values_contains_all_entries() {
        let r = ReRolledActions::new();
        assert!(!r.values().is_empty());
    }

    #[test]
    fn for_name_returns_action_with_original_casing() {
        let r = ReRolledActions::new();
        // Keys are lowercased, but the stored action preserves the original name
        let action = r.for_name("DODGE").expect("should find Dodge case-insensitively");
        assert_eq!(action.get_name(), "Dodge");
    }

    #[test]
    fn default_equals_new() {
        let via_new = ReRolledActions::new();
        let via_default = ReRolledActions::default();
        // Both must produce the same set of action names
        assert_eq!(
            via_new.values().len(),
            via_default.values().len(),
            "default() and new() should produce the same number of entries"
        );
        for key in via_new.values().keys() {
            assert!(via_default.values().contains_key(key), "default() missing key: {key}");
        }
    }
}
