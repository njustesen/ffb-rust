use ffb_model::model::Player;
use crate::modifiers::player_stat_key::PlayerStatKey;
use crate::modifiers::stat_based_roll_modifier::StatBasedRollModifier;

/// 1:1 translation of com.fumbbl.ffb.modifiers.StatBasedRollModifierFactory.
pub struct StatBasedRollModifierFactory {
    pub name: String,
    pub stat_key: PlayerStatKey,
}

impl StatBasedRollModifierFactory {
    pub fn new(name: impl Into<String>, stat_key: PlayerStatKey) -> Self {
        Self { name: name.into(), stat_key }
    }

    pub fn create(&self, player: &Player) -> StatBasedRollModifier {
        let value = match self.stat_key {
            PlayerStatKey::AG => player.agility_with_modifiers(),
            PlayerStatKey::AV => player.armour_with_modifiers(),
            PlayerStatKey::MA => player.movement_with_modifiers(),
            PlayerStatKey::PA => player.passing_with_modifiers(),
            PlayerStatKey::ST => player.strength_with_modifiers(),
        };
        StatBasedRollModifier::new(&self.name, value)
    }
}
