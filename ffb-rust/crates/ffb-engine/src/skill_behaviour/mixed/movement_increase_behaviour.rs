//! 1:1 translation of `com.fumbbl.ffb.server.skillbehaviour.mixed.MovementIncreaseBehaviour`
//! (@RulesCollection BB2020 + BB2025).
//!
//! A load-time `PlayerModifier`: `RosterPlayer.updatePosition(.., updateStats=true, ..)` resets
//! the stats from the position, then `applyPlayerModifiersFromBehaviours` runs this
//! (Rust: `registry::apply_player_modifiers_from_behaviours`, BACKLOG §M phase 2b(h)).
use ffb_model::enums::SkillId;
use ffb_model::model::{Player, PlayerModifier};

use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::skill_behaviour::registry::SkillRegistry;

pub struct MovementIncreaseBehaviour;

impl MovementIncreaseBehaviour {
    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_player_modifier(Box::new(MovementIncreaseModifier));
        registry.register(SkillId::MovementIncrease, sb);
    }
}

struct MovementIncreaseModifier;

impl PlayerModifier for MovementIncreaseModifier {
    /// Java: `player.setMovement(Math.min(Math.min(9, position.getMovement() + 2), player.getMovement() + 1))`
    fn apply(&self, player: &mut Player) {
        player.movement = 9.min(player.position_movement + 2).min(player.movement + 1);
    }
}
