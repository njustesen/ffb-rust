use crate::model::player::Player;

/// 1:1 translation of com.fumbbl.ffb.model.PlayerModifier.
pub trait PlayerModifier {
    fn apply(&self, player: &mut Player);
}
