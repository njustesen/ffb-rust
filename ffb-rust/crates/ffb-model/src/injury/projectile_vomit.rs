/// 1:1 translation of com.fumbbl.ffb.injury.ProjectileVomit.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct ProjectileVomit {
    base: InjuryType,
}

impl ProjectileVomit {
    pub fn new() -> Self {
        Self { base: InjuryType::new("projectileVomit", false, SendToBoxReason::PROJECTILE_VOMIT) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_vomit_like(&self) -> bool { true }

    pub fn is_caused_by_opponent(&self) -> bool { true }
}

impl Default for ProjectileVomit {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(ProjectileVomit::new().base().name(), "projectileVomit");
    }

    #[test]
    fn worth_spps_is_false() {
        assert!(!ProjectileVomit::new().base().is_worth_spps());
    }

    #[test]
    fn is_vomit_like_is_true() {
        assert!(ProjectileVomit::new().is_vomit_like());
    }

    #[test]
    fn is_caused_by_opponent_is_true() {
        assert!(ProjectileVomit::new().is_caused_by_opponent());
    }
}
