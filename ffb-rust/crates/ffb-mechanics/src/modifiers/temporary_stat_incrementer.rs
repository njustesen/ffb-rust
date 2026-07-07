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

    #[test]
    fn incrementer_does_not_apply_to_other_stats() {
        let inc = new_incrementer(PlayerStatKey::MA, PlayerStatLimit::new(1, 9));
        assert!(!inc.applies_to(PlayerStatKey::ST));
        assert!(!inc.applies_to(PlayerStatKey::AG));
        assert!(!inc.applies_to(PlayerStatKey::PA));
        assert!(!inc.applies_to(PlayerStatKey::AV));
    }

    #[test]
    fn incrementer_limit_matches() {
        let inc = new_incrementer(PlayerStatKey::PA, PlayerStatLimit::new(2, 6));
        assert_eq!(inc.get_limit().get_min(), 2);
        assert_eq!(inc.get_limit().get_max(), 6);
    }

    #[test]
    fn incrementer_apply_from_max_still_adds_one() {
        // The incrementer applies unconditionally; clamping is caller's responsibility
        let inc = new_incrementer(PlayerStatKey::ST, PlayerStatLimit::new(1, 9));
        assert_eq!(inc.apply(9), 10);
    }
}
