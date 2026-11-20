//! 1:1 translations of the three BB2020 stat-increase behaviours, each a load-time
//! `PlayerModifier` (see `registry::apply_player_modifiers_from_behaviours`, BACKLOG §M 2b(h)):
//! - `skillbehaviour.bb2020.AgilityIncreaseBehaviour` (AG is a target number: -1 is better)
//! - `skillbehaviour.bb2020.PassingIncreaseBehaviour` (no PA (<= 0) becomes 6+)
//! - `skillbehaviour.bb2020.StrengthIncreaseBehaviour`
//! (+MA / +AV are the `mixed` behaviours.) The BB2025 classes are byte-identical in logic and
//! reuse these modifiers via `bb2025::stat_increase_behaviours`.
use ffb_model::enums::SkillId;
use ffb_model::model::{Player, PlayerModifier};

use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::skill_behaviour::registry::SkillRegistry;

fn register(registry: &mut SkillRegistry, id: SkillId, m: Box<dyn PlayerModifier>) {
    let mut sb = SbContainer::new();
    sb.register_player_modifier(m);
    registry.register(id, sb);
}

pub struct AgilityIncreaseBehaviour;
impl AgilityIncreaseBehaviour {
    pub fn register_into(registry: &mut SkillRegistry) {
        register(registry, SkillId::AgilityIncrease, Box::new(AgilityIncreaseModifier));
    }
}
pub(crate) struct AgilityIncreaseModifier;
impl PlayerModifier for AgilityIncreaseModifier {
    /// Java: `setAgility(Math.max(Math.max(1, position.getAgility() - 2), player.getAgility() - 1))`
    fn apply(&self, player: &mut Player) {
        player.agility = 1.max(player.position_agility - 2).max(player.agility - 1);
    }
}

pub struct PassingIncreaseBehaviour;
impl PassingIncreaseBehaviour {
    pub fn register_into(registry: &mut SkillRegistry) {
        register(registry, SkillId::PassingIncrease, Box::new(PassingIncreaseModifier));
    }
}
pub(crate) struct PassingIncreaseModifier;
impl PlayerModifier for PassingIncreaseModifier {
    /// Java: `if (getPassing() <= 0) setPassing(6); else
    /// setPassing(Math.max(Math.max(1, position.getPassing() - 2), player.getPassing() - 1))`
    fn apply(&self, player: &mut Player) {
        if player.passing <= 0 {
            player.passing = 6;
        } else {
            player.passing = 1.max(player.position_passing - 2).max(player.passing - 1);
        }
    }
}

pub struct StrengthIncreaseBehaviour;
impl StrengthIncreaseBehaviour {
    pub fn register_into(registry: &mut SkillRegistry) {
        register(registry, SkillId::StrengthIncrease, Box::new(StrengthIncreaseModifier));
    }
}
pub(crate) struct StrengthIncreaseModifier;
impl PlayerModifier for StrengthIncreaseModifier {
    /// Java: `setStrength(Math.min(Math.min(8, position.getStrength() + 2), player.getStrength() + 1))`
    fn apply(&self, player: &mut Player) {
        player.strength = 8.min(player.position_strength + 2).min(player.strength + 1);
    }
}
