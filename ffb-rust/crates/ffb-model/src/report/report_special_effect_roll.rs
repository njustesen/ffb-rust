use crate::model::special_effect::SpecialEffect;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportSpecialEffectRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportSpecialEffectRoll {
    pub special_effect: SpecialEffect,
    pub player_id: Option<String>,
    pub roll: i32,
    pub successful: bool,
}

impl ReportSpecialEffectRoll {
    pub fn new(
        special_effect: SpecialEffect,
        player_id: Option<String>,
        roll: i32,
        successful: bool,
    ) -> Self {
        Self { special_effect, player_id, roll, successful }
    }

    pub fn get_special_effect(&self) -> SpecialEffect { self.special_effect }
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_roll(&self) -> i32 { self.roll }
    pub fn is_successful(&self) -> bool { self.successful }
}

impl IReport for ReportSpecialEffectRoll {
    fn get_id(&self) -> ReportId { ReportId::SPELL_EFFECT_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportSpecialEffectRoll {
        ReportSpecialEffectRoll::new(SpecialEffect::LIGHTNING, Some("p1".into()), 4, true)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::SPELL_EFFECT_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "spellEffectRoll");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_special_effect(), SpecialEffect::LIGHTNING);
        assert_eq!(r.get_player_id(), Some("p1"));
        assert_eq!(r.get_roll(), 4);
        assert!(r.is_successful());
    }

    #[test]
    fn unsuccessful_no_player() {
        let r = ReportSpecialEffectRoll::new(SpecialEffect::FIREBALL, None, 2, false);
        assert!(!r.is_successful());
        assert_eq!(r.get_player_id(), None);
    }

    #[test]
    fn fireball_effect() {
        let r = ReportSpecialEffectRoll::new(SpecialEffect::FIREBALL, Some("p2".into()), 5, true);
        assert_eq!(r.get_special_effect(), SpecialEffect::FIREBALL);
        assert_eq!(r.get_roll(), 5);
    }
}
