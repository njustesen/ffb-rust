use crate::modifiers::player_stat_key::PlayerStatKey;
use crate::modifiers::player_stat_limit::PlayerStatLimit;
use crate::modifiers::temporary_stat_modifier::TemporaryStatModifier;

/// 1:1 translation of com.fumbbl.ffb.modifiers.TemporaryStatDecrementer. apply = value - 1.
pub fn new_decrementer(key: PlayerStatKey, limit: PlayerStatLimit) -> TemporaryStatModifier {
    TemporaryStatModifier::new(key, limit, |v| v - 1)
}
