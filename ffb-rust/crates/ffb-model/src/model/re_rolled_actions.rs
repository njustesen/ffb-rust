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
