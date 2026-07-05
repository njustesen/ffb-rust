use crate::modifiers::player_stat_key::PlayerStatKey;
use crate::modifiers::player_stat_limit::PlayerStatLimit;
use crate::modifiers::temporary_stat_modifier::TemporaryStatModifier;

/// 1:1 translation of com.fumbbl.ffb.modifiers.TemporaryStatIncrementer. apply = value + 1.
pub fn new_incrementer(key: PlayerStatKey, limit: PlayerStatLimit) -> TemporaryStatModifier {
    TemporaryStatModifier::new(key, limit, |v| v + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modifiers::player_stat_limit::PlayerStatLimit;

    #[test]
    fn incrementer_apply_adds_one() {
        let inc = new_incrementer(PlayerStatKey::ST, PlayerStatLimit::new(1, 9));
        assert_eq!(inc.apply(3), 4);
        assert_eq!(inc.apply(0), 1);
    }

    #[test]
    fn incrementer_applies_to_correct_stat() {
        let inc = new_incrementer(PlayerStatKey::AV, PlayerStatLimit::new(3, 11));
        assert!(inc.applies_to(PlayerStatKey::AV));
        assert!(!inc.applies_to(PlayerStatKey::ST));
    }
}
