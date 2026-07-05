use crate::modifiers::player_stat_key::PlayerStatKey;
use crate::modifiers::player_stat_limit::PlayerStatLimit;
use crate::modifiers::temporary_stat_modifier::TemporaryStatModifier;

/// 1:1 translation of com.fumbbl.ffb.modifiers.TemporaryStatDecrementer. apply = value - 1.
pub fn new_decrementer(key: PlayerStatKey, limit: PlayerStatLimit) -> TemporaryStatModifier {
    TemporaryStatModifier::new(key, limit, |v| v - 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modifiers::player_stat_limit::PlayerStatLimit;

    #[test]
    fn decrementer_apply_subtracts_one() {
        let d = new_decrementer(PlayerStatKey::MA, PlayerStatLimit::new(1, 9));
        assert_eq!(d.apply(5), 4);
        assert_eq!(d.apply(1), 0);
    }

    #[test]
    fn decrementer_applies_to_correct_stat() {
        let d = new_decrementer(PlayerStatKey::AG, PlayerStatLimit::new(1, 6));
        assert!(d.applies_to(PlayerStatKey::AG));
        assert!(!d.applies_to(PlayerStatKey::MA));
    }
}
