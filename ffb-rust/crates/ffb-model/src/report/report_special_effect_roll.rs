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

impl ReportSpecialEffectRoll {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "specialEffect": self.special_effect.get_name(),
            "playerId": self.player_id,
            "roll": self.roll,
            "successful": self.successful,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            special_effect: json["specialEffect"].as_str()
                .and_then(SpecialEffect::for_name)
                .unwrap_or(SpecialEffect::LIGHTNING),
            player_id: json["playerId"].as_str().map(str::to_string),
            roll: json["roll"].as_i64().unwrap_or(0) as i32,
            successful: json["successful"].as_bool().unwrap_or(false),
        }
    }
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

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportSpecialEffectRoll::from_json(&json);
        assert_eq!(restored.special_effect, original.special_effect);
        assert_eq!(restored.player_id, original.player_id);
        assert_eq!(restored.roll, original.roll);
        assert_eq!(restored.successful, original.successful);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("spellEffectRoll"));
    }
}
