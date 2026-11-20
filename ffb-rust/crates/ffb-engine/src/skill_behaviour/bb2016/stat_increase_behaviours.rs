//! 1:1 translations of the four BB2016 stat-increase behaviours, each a load-time
//! `PlayerModifier` (see `registry::apply_player_modifiers_from_behaviours`, BACKLOG §M 2b(h)):
//! - `skillbehaviour.bb2016.AgilityIncreaseBehaviour` (BB2016 AG is a characteristic, +1 is better)
//! - `skillbehaviour.bb2016.ArmourIncreaseBehaviour`
//! - `skillbehaviour.bb2016.MovementIncreaseBehaviour`
//! - `skillbehaviour.bb2016.StrengthIncreaseBehaviour`
//! Kept in one file because each Java class is a constructor with one lambda and nothing else.
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
struct AgilityIncreaseModifier;
impl PlayerModifier for AgilityIncreaseModifier {
    /// Java: `setAgility(Math.min(Math.min(10, position.getAgility() + 2), player.getAgility() + 1))`
    fn apply(&self, player: &mut Player) {
        player.agility = 10.min(player.position_agility + 2).min(player.agility + 1);
    }
}

pub struct ArmourIncreaseBehaviour;
impl ArmourIncreaseBehaviour {
    pub fn register_into(registry: &mut SkillRegistry) {
        register(registry, SkillId::ArmourIncrease, Box::new(ArmourIncreaseModifier));
    }
}
struct ArmourIncreaseModifier;
impl PlayerModifier for ArmourIncreaseModifier {
    /// Java: `setArmour(Math.min(Math.min(10, position.getArmour() + 2), player.getArmour() + 1))`
    fn apply(&self, player: &mut Player) {
        player.armour = 10.min(player.position_armour + 2).min(player.armour + 1);
    }
}

pub struct MovementIncreaseBehaviour;
impl MovementIncreaseBehaviour {
    pub fn register_into(registry: &mut SkillRegistry) {
        register(registry, SkillId::MovementIncrease, Box::new(MovementIncreaseModifier));
    }
}
struct MovementIncreaseModifier;
impl PlayerModifier for MovementIncreaseModifier {
    /// Java: `setMovement(Math.min(Math.min(10, position.getMovement() + 2), player.getMovement() + 1))`
    fn apply(&self, player: &mut Player) {
        player.movement = 10.min(player.position_movement + 2).min(player.movement + 1);
    }
}

pub struct StrengthIncreaseBehaviour;
impl StrengthIncreaseBehaviour {
    pub fn register_into(registry: &mut SkillRegistry) {
        register(registry, SkillId::StrengthIncrease, Box::new(StrengthIncreaseModifier));
    }
}
struct StrengthIncreaseModifier;
impl PlayerModifier for StrengthIncreaseModifier {
    /// Java: `setStrength(Math.min(Math.min(10, position.getStrength() + 2), player.getStrength() + 1))`
    fn apply(&self, player: &mut Player) {
        player.strength = 10.min(player.position_strength + 2).min(player.strength + 1);
    }
}
