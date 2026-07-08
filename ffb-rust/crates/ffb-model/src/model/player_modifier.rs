use crate::model::player::Player;

/// 1:1 translation of com.fumbbl.ffb.model.PlayerModifier.
pub trait PlayerModifier: Send + Sync {
    fn apply(&self, player: &mut Player);
}
