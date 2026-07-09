/// 1:1 translation of com.fumbbl.ffb.injury.TtmHitPlayer.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct TtmHitPlayer {
    base: InjuryType,
}

impl TtmHitPlayer {
    pub fn new() -> Self {
        Self { base: InjuryType::new("ttmHitPlayer", false, SendToBoxReason::HIT_BY_THROWN_PLAYER) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }
}

impl Default for TtmHitPlayer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(TtmHitPlayer::new().base().name(), "ttmHitPlayer");
    }

    #[test]
    fn worth_spps_is_false() {
        assert!(!TtmHitPlayer::new().base().is_worth_spps());
    }

    #[test]
    fn send_to_box_reason_is_hit_by_thrown_player() {
        use crate::model::send_to_box_reason::SendToBoxReason;
        assert_eq!(TtmHitPlayer::new().base().send_to_box_reason(), SendToBoxReason::HIT_BY_THROWN_PLAYER);
    }
}
