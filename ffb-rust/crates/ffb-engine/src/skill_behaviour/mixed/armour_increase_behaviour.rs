//! 1:1 translation of `com.fumbbl.ffb.server.skillbehaviour.mixed.ArmourIncreaseBehaviour`
//! (@RulesCollection BB2020 + BB2025). Load-time `PlayerModifier` (see
//! `registry::apply_player_modifiers_from_behaviours`, BACKLOG §M phase 2b(h)).
use ffb_model::enums::SkillId;
use ffb_model::model::{Player, PlayerModifier};

use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::skill_behaviour::registry::SkillRegistry;

pub struct ArmourIncreaseBehaviour;

impl ArmourIncreaseBehaviour {
    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_player_modifier(Box::new(ArmourIncreaseModifier));
        registry.register(SkillId::ArmourIncrease, sb);
    }
}

struct ArmourIncreaseModifier;

impl PlayerModifier for ArmourIncreaseModifier {
    /// Java: `player.setArmour(Math.min(Math.min(11, position.getArmour() + 2), player.getArmour() + 1))`
    fn apply(&self, player: &mut Player) {
        player.armour = 11.min(player.position_armour + 2).min(player.armour + 1);
    }
}
