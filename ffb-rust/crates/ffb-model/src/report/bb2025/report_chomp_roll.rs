use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportChompRoll.java` — extends ReportSkillRoll.
#[derive(Debug, Clone)]
pub struct ReportChompRoll {
    pub base: ReportSkillRoll,
    pub chomper: String,
    pub chompee: String,
}

impl ReportChompRoll {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        roll: i32,
        minimum_roll: i32,
        re_rolled: bool,
        chomper: String,
        chompee: String,
    ) -> Self {
        Self {
            base: ReportSkillRoll::new(player_id, successful, roll, minimum_roll, re_rolled, vec![]),
            chomper,
            chompee,
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.base.get_player_id() }
    pub fn is_successful(&self) -> bool { self.base.is_successful() }
    pub fn get_roll(&self) -> i32 { self.base.get_roll() }
    pub fn get_minimum_roll(&self) -> i32 { self.base.get_minimum_roll() }
    pub fn is_re_rolled(&self) -> bool { self.base.is_re_rolled() }
    pub fn get_chomper(&self) -> &str { &self.chomper }
    pub fn get_chompee(&self) -> &str { &self.chompee }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.base.player_id,
            "successful": self.base.successful,
            "roll": self.base.roll,
            "minimumRoll": self.base.minimum_roll,
            "reRolled": self.base.re_rolled,
            "rollModifiers": self.base.roll_modifier_names,
            "chomper": self.chomper,
            "chompee": self.chompee,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        use crate::report::report_skill_roll::ReportSkillRoll;
        Self {
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
            chomper: json["chomper"].as_str().unwrap_or("").to_string(),
            chompee: json["chompee"].as_str().unwrap_or("").to_string(),
        }
    }
}

impl IReport for ReportChompRoll {
    fn get_id(&self) -> ReportId { ReportId::CHOMP_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportChompRoll {
        ReportChompRoll::new(Some("p1".into()), true, 5, 3, false, "chomper1".into(), "chompee1".into())
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::CHOMP_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "chompRoll");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_chomper(), "chomper1");
        assert_eq!(r.get_chompee(), "chompee1");
        assert!(r.is_successful());
    }

    #[test]
    fn minimum_roll_and_rerolled() {
        let r = make();
        assert_eq!(r.get_minimum_roll(), 3);
        assert!(!r.is_re_rolled());
    }

    #[test]
    fn unsuccessful_with_player_id() {
        let r = ReportChompRoll::new(Some("p2".into()), false, 2, 4, true, "c1".into(), "c2".into());
        assert!(!r.is_successful());
        assert_eq!(r.get_player_id(), Some("p2"));
        assert!(r.is_re_rolled());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportChompRoll::from_json(&json);
        assert_eq!(restored.base.player_id, original.base.player_id);
        assert_eq!(restored.base.successful, original.base.successful);
        assert_eq!(restored.base.roll, original.base.roll);
        assert_eq!(restored.base.minimum_roll, original.base.minimum_roll);
        assert_eq!(restored.base.re_rolled, original.base.re_rolled);
        assert_eq!(restored.chomper, original.chomper);
        assert_eq!(restored.chompee, original.chompee);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("chompRoll"));
    }
}
