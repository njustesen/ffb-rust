use crate::modifiers::player_stat_key::PlayerStatKey;
use crate::modifiers::player_stat_limit::PlayerStatLimit;

/// 1:1 translation of com.fumbbl.ffb.modifiers.TemporaryStatModifier (abstract).
pub struct TemporaryStatModifier {
    pub key: PlayerStatKey,
    pub limit: PlayerStatLimit,
    apply_fn: Box<dyn Fn(i32) -> i32 + Send + Sync>,
}

impl TemporaryStatModifier {
    pub fn new(key: PlayerStatKey, limit: PlayerStatLimit, apply_fn: impl Fn(i32) -> i32 + Send + Sync + 'static) -> Self {
        Self { key, limit, apply_fn: Box::new(apply_fn) }
    }

    pub fn get_name(&self) -> String {
        format!("{:?}-{}", self.key, std::any::type_name::<Self>())
    }

    pub fn applies_to(&self, stat: PlayerStatKey) -> bool {
        self.key == stat
    }

    pub fn apply(&self, value: i32) -> i32 {
        (self.apply_fn)(value)
    }

    pub fn get_limit(&self) -> &PlayerStatLimit { &self.limit }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_fn_is_called_with_value() {
        let m = TemporaryStatModifier::new(PlayerStatKey::MA, PlayerStatLimit::new(1, 9), |v| v + 2);
        assert_eq!(m.apply(4), 6);
    }

    #[test]
    fn applies_to_matching_stat() {
        let m = TemporaryStatModifier::new(PlayerStatKey::ST, PlayerStatLimit::new(1, 9), |v| v);
        assert!(m.applies_to(PlayerStatKey::ST));
        assert!(!m.applies_to(PlayerStatKey::MA));
    }

    #[test]
    fn get_limit_returns_correct_bounds() {
        let m = TemporaryStatModifier::new(PlayerStatKey::AV, PlayerStatLimit::new(2, 13), |v| v);
        assert_eq!(m.get_limit().get_min(), 2);
        assert_eq!(m.get_limit().get_max(), 13);
    }

    #[test]
    fn apply_fn_can_clamp_to_limit() {
        let limit = PlayerStatLimit::new(1, 6);
        let m = TemporaryStatModifier::new(PlayerStatKey::MA, limit, |v| v.max(1).min(6));
        assert_eq!(m.apply(0), 1);
        assert_eq!(m.apply(10), 6);
        assert_eq!(m.apply(3), 3);
    }

    #[test]
    fn applies_to_returns_false_for_other_stat() {
        let m = TemporaryStatModifier::new(PlayerStatKey::AG, PlayerStatLimit::new(1, 6), |v| v + 1);
        assert!(m.applies_to(PlayerStatKey::AG));
        assert!(!m.applies_to(PlayerStatKey::PA));
        assert!(!m.applies_to(PlayerStatKey::AV));
    }
}
