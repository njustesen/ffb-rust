use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_pickup_roll::ReportPickupRoll as BaseReportPickupRoll;

/// 1:1 translation of `ReportPickupRoll.java` (bb2025) — extends base ReportPickupRoll.
#[derive(Debug, Clone)]
pub struct ReportPickupRoll {
    pub base: BaseReportPickupRoll,
    pub secure_the_ball: bool,
}

impl ReportPickupRoll {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        roll: i32,
        minimum_roll: i32,
        re_rolled: bool,
        roll_modifiers: Vec<String>,
        secure_the_ball: bool,
    ) -> Self {
        Self {
            base: BaseReportPickupRoll::new(player_id, successful, roll, minimum_roll, re_rolled, roll_modifiers),
            secure_the_ball,
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.base.get_player_id() }
    pub fn is_successful(&self) -> bool { self.base.is_successful() }
    pub fn get_roll(&self) -> i32 { self.base.get_roll() }
    pub fn get_minimum_roll(&self) -> i32 { self.base.get_minimum_roll() }
    pub fn is_re_rolled(&self) -> bool { self.base.is_re_rolled() }
    pub fn is_secure_the_ball(&self) -> bool { self.secure_the_ball }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.base.base.player_id,
            "successful": self.base.base.successful,
            "roll": self.base.base.roll,
            "minimumRoll": self.base.base.minimum_roll,
            "reRolled": self.base.base.re_rolled,
            "rollModifiers": self.base.base.roll_modifier_names,
            "secureTheBallUsed": self.secure_the_ball,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        use crate::report::report_skill_roll::ReportSkillRoll;
        Self {
            base: BaseReportPickupRoll {
                base: ReportSkillRoll {
                    player_id: json["playerId"].as_str().map(String::from),
                    successful: json["successful"].as_bool().unwrap_or(false),
                    roll: json["roll"].as_i64().unwrap_or(0) as i32,
                    minimum_roll: json["minimumRoll"].as_i64().unwrap_or(0) as i32,
                    re_rolled: json["reRolled"].as_bool().unwrap_or(false),
                    roll_modifier_names: json["rollModifiers"].as_array()
                        .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                        .unwrap_or_default(),
                },
            },
            secure_the_ball: json["secureTheBallUsed"].as_bool().unwrap_or(false),
        }
    }
}

impl IReport for ReportPickupRoll {
    fn get_id(&self) -> ReportId { ReportId::PICK_UP_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPickupRoll {
        ReportPickupRoll::new(Some("p1".into()), true, 4, 3, false, vec![], true)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::PICK_UP_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "pickUpRoll");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_player_id(), Some("p1"));
        assert!(r.is_secure_the_ball());
    }

    #[test]
    fn minimum_roll_and_rerolled() {
        let r = make();
        assert_eq!(r.get_minimum_roll(), 3);
        assert!(!r.is_re_rolled());
    }

    #[test]
    fn unsuccessful_without_secure_the_ball() {
        let r = ReportPickupRoll::new(None, false, 2, 4, true, vec!["TackleZone".into()], false);
        assert!(!r.is_successful());
        assert!(!r.is_secure_the_ball());
        assert_eq!(r.get_player_id(), None);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportPickupRoll::from_json(&json);
        assert_eq!(restored.base.base.player_id, original.base.base.player_id);
        assert_eq!(restored.base.base.successful, original.base.base.successful);
        assert_eq!(restored.base.base.roll, original.base.base.roll);
        assert_eq!(restored.base.base.minimum_roll, original.base.base.minimum_roll);
        assert_eq!(restored.base.base.re_rolled, original.base.base.re_rolled);
        assert_eq!(restored.secure_the_ball, original.secure_the_ball);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("pickUpRoll"));
    }
}
