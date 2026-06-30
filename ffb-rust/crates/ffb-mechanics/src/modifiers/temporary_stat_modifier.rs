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
