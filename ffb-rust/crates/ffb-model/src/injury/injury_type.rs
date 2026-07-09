/// 1:1 translation of com.fumbbl.ffb.injury.InjuryType.
use crate::model::send_to_box_reason::SendToBoxReason;

/// Abstract base for all injury types. Concrete subtypes live in sibling modules.
pub struct InjuryType {
    name: String,
    worth_spps: bool,
    send_to_box_reason: SendToBoxReason,
    failed_armour_places_prone: bool,
}

impl InjuryType {
    /// `InjuryType(String pName, boolean pWorthSpps, SendToBoxReason pSendToBoxReason)`.
    pub fn new(name: impl Into<String>, worth_spps: bool, send_to_box_reason: SendToBoxReason) -> Self {
        InjuryType {
            name: name.into(),
            worth_spps,
            send_to_box_reason,
            failed_armour_places_prone: true,
        }
    }

    /// Java `getName()`.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Java `isWorthSpps()`.
    pub fn is_worth_spps(&self) -> bool {
        self.worth_spps
    }

    /// Java `sendToBoxReason()`.
    pub fn send_to_box_reason(&self) -> SendToBoxReason {
        self.send_to_box_reason
    }

    /// Java `isCausedByOpponent()` — false in base class, overridden by subtypes.
    pub fn is_caused_by_opponent(&self) -> bool {
        false
    }

    /// Java `canUseApo()` — true in base class, overridden by EatPlayer etc.
    pub fn can_use_apo(&self) -> bool {
        true
    }

    /// Java `canApoKoIntoStun()` — true in base class.
    pub fn can_apo_ko_into_stun(&self) -> bool {
        true
    }

    /// Java `shouldPlayFallSound()` — true in base class.
    pub fn should_play_fall_sound(&self) -> bool {
        true
    }

    /// Java `fallingDownCausesTurnover()` — true in base class.
    pub fn falling_down_causes_turnover(&self) -> bool {
        true
    }

    /// Java `failedArmourPlacesProne()`.
    pub fn failed_armour_places_prone(&self) -> bool {
        self.failed_armour_places_prone
    }

    /// Java `setFailedArmourPlacesProne(boolean)`.
    pub fn set_failed_armour_places_prone(&mut self, flag: bool) {
        self.failed_armour_places_prone = flag;
    }

    /// Java `isStab()` — false in base class.
    pub fn is_stab(&self) -> bool {
        false
    }

    /// Java `isFoul()` — false in base class.
    pub fn is_foul(&self) -> bool {
        false
    }

    /// Java `isVomitLike()` — false in base class.
    pub fn is_vomit_like(&self) -> bool {
        false
    }

    /// Java `isChainsaw()` — false in base class.
    pub fn is_chainsaw(&self) -> bool {
        false
    }

    /// Java `isBlock()` — false in base class.
    pub fn is_block(&self) -> bool {
        false
    }
}

impl Default for InjuryType {
    fn default() -> Self {
        InjuryType::new("", false, SendToBoxReason::MNG)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::send_to_box_reason::SendToBoxReason;

    #[test]
    fn new_stores_name() {
        let t = InjuryType::new("block", true, SendToBoxReason::BLOCKED);
        assert_eq!(t.name(), "block");
    }

    #[test]
    fn new_stores_worth_spps() {
        let t = InjuryType::new("block", true, SendToBoxReason::BLOCKED);
        assert!(t.is_worth_spps());
    }

    #[test]
    fn new_stores_send_to_box_reason() {
        let t = InjuryType::new("block", true, SendToBoxReason::BLOCKED);
        assert_eq!(t.send_to_box_reason(), SendToBoxReason::BLOCKED);
    }

    #[test]
    fn defaults_to_not_caused_by_opponent() {
        let t = InjuryType::new("block", false, SendToBoxReason::FOULED);
        assert!(!t.is_caused_by_opponent());
    }

    #[test]
    fn can_use_apo_defaults_true() {
        let t = InjuryType::new("block", false, SendToBoxReason::BLOCKED);
        assert!(t.can_use_apo());
    }

    #[test]
    fn can_apo_ko_into_stun_defaults_true() {
        let t = InjuryType::new("block", false, SendToBoxReason::BLOCKED);
        assert!(t.can_apo_ko_into_stun());
    }

    #[test]
    fn should_play_fall_sound_defaults_true() {
        let t = InjuryType::new("block", false, SendToBoxReason::BLOCKED);
        assert!(t.should_play_fall_sound());
    }

    #[test]
    fn falling_down_causes_turnover_defaults_true() {
        let t = InjuryType::new("block", false, SendToBoxReason::BLOCKED);
        assert!(t.falling_down_causes_turnover());
    }

    #[test]
    fn failed_armour_places_prone_defaults_true() {
        let t = InjuryType::new("block", false, SendToBoxReason::BLOCKED);
        assert!(t.failed_armour_places_prone());
    }

    #[test]
    fn set_failed_armour_places_prone_to_false() {
        let mut t = InjuryType::new("block", false, SendToBoxReason::BLOCKED);
        t.set_failed_armour_places_prone(false);
        assert!(!t.failed_armour_places_prone());
    }

    #[test]
    fn is_block_defaults_false() {
        let t = InjuryType::new("block", false, SendToBoxReason::BLOCKED);
        assert!(!t.is_block());
    }

    #[test]
    fn is_foul_defaults_false() {
        let t = InjuryType::new("foul", false, SendToBoxReason::FOULED);
        assert!(!t.is_foul());
    }
}
