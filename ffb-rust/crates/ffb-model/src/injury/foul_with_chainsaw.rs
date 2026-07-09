/// 1:1 translation of com.fumbbl.ffb.injury.FoulWithChainsaw.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct FoulWithChainsaw {
    base: InjuryType,
}

impl FoulWithChainsaw {
    pub fn new() -> Self {
        Self { base: InjuryType::new("foulWithChainsaw", false, SendToBoxReason::FOULED) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn should_play_fall_sound(&self) -> bool { false }

    pub fn is_foul(&self) -> bool { true }

    pub fn is_chainsaw(&self) -> bool { true }
}

impl Default for FoulWithChainsaw {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(FoulWithChainsaw::new().base().name(), "foulWithChainsaw");
    }

    #[test]
    fn worth_spps_is_false() {
        assert!(!FoulWithChainsaw::new().base().is_worth_spps());
    }

    #[test]
    fn is_foul_is_true() {
        assert!(FoulWithChainsaw::new().is_foul());
    }

    #[test]
    fn is_chainsaw_is_true() {
        assert!(FoulWithChainsaw::new().is_chainsaw());
    }
}
