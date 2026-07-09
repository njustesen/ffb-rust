use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportPassRoll.java` — extends ReportSkillRoll.
#[derive(Debug, Clone)]
pub struct ReportPassRoll {
    pub base: ReportSkillRoll,
    /// `fPassingDistance` — distance category name (e.g. "SHORT_PASS").
    pub passing_distance: Option<String>,
    /// `fHailMaryPass`
    pub hail_mary_pass: bool,
    /// `fBomb`
    pub bomb: bool,
    /// `result` — PassResult name string.
    pub result: Option<String>,
    /// `statBasedRollModifier` — modifier name, if any.
    pub stat_based_roll_modifier: Option<String>,
}

impl ReportPassRoll {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        roll: i32,
        minimum_roll: i32,
        re_rolled: bool,
        roll_modifiers: Vec<String>,
        passing_distance: Option<String>,
        bomb: bool,
        result: Option<String>,
        hail_mary_pass: bool,
        stat_based_roll_modifier: Option<String>,
    ) -> Self {
        Self {
            base: ReportSkillRoll::new(player_id, successful, roll, minimum_roll, re_rolled, roll_modifiers),
            passing_distance,
            hail_mary_pass,
            bomb,
            result,
            stat_based_roll_modifier,
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.base.get_player_id() }
    pub fn is_successful(&self) -> bool { self.base.is_successful() }
    pub fn get_roll(&self) -> i32 { self.base.get_roll() }
    pub fn get_minimum_roll(&self) -> i32 { self.base.get_minimum_roll() }
    pub fn is_re_rolled(&self) -> bool { self.base.is_re_rolled() }
    pub fn get_roll_modifiers(&self) -> &[String] { self.base.get_roll_modifiers() }
    pub fn get_passing_distance(&self) -> Option<&str> { self.passing_distance.as_deref() }
    pub fn is_hail_mary_pass(&self) -> bool { self.hail_mary_pass }
    pub fn is_bomb(&self) -> bool { self.bomb }
    pub fn get_result(&self) -> Option<&str> { self.result.as_deref() }
    pub fn get_stat_based_roll_modifier(&self) -> Option<&str> { self.stat_based_roll_modifier.as_deref() }
}

impl IReport for ReportPassRoll {
    fn get_id(&self) -> ReportId { ReportId::PASS_ROLL }
}

impl ReportPassRoll {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.base.player_id,
            "successful": self.base.successful,
            "roll": self.base.roll,
            "minimumRoll": self.base.minimum_roll,
            "reRolled": self.base.re_rolled,
            "rollModifiers": self.base.roll_modifier_names,
            "passingDistance": self.passing_distance,
            "passResult": self.result,
            "hailMaryPass": self.hail_mary_pass,
            "bomb": self.bomb,
            "statBasedRollModifier": self.stat_based_roll_modifier,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            base: ReportSkillRoll::new(
                json["playerId"].as_str().map(str::to_string),
                json["successful"].as_bool().unwrap_or(false),
                json["roll"].as_i64().unwrap_or(0) as i32,
                json["minimumRoll"].as_i64().unwrap_or(0) as i32,
                json["reRolled"].as_bool().unwrap_or(false),
                json["rollModifiers"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()).unwrap_or_default(),
            ),
            passing_distance: json["passingDistance"].as_str().map(str::to_string),
            hail_mary_pass: json["hailMaryPass"].as_bool().unwrap_or(false),
            bomb: json["bomb"].as_bool().unwrap_or(false),
            result: json["passResult"].as_str().map(str::to_string),
            stat_based_roll_modifier: json["statBasedRollModifier"].as_str().map(str::to_string),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPassRoll {
        ReportPassRoll::new(
            Some("p1".into()), true, 4, 3, false, vec![],
            Some("SHORT_PASS".into()), false, Some("ACCURATE".into()), false, None,
        )
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::PASS_ROLL); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "passRoll"); }

    #[test]
    fn get_result() { assert_eq!(make().get_result(), Some("ACCURATE")); }

    #[test]
    fn get_passing_distance() { assert_eq!(make().get_passing_distance(), Some("SHORT_PASS")); }
    #[test]
    fn get_name_is_nonempty() {
        assert!(!make().get_name().is_empty());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportPassRoll::from_json(&json);
        assert_eq!(restored.base.player_id, original.base.player_id);
        assert_eq!(restored.base.successful, original.base.successful);
        assert_eq!(restored.base.roll, original.base.roll);
        assert_eq!(restored.base.minimum_roll, original.base.minimum_roll);
        assert_eq!(restored.base.re_rolled, original.base.re_rolled);
        assert_eq!(restored.passing_distance, original.passing_distance);
        assert_eq!(restored.result, original.result);
        assert_eq!(restored.hail_mary_pass, original.hail_mary_pass);
        assert_eq!(restored.bomb, original.bomb);
        assert_eq!(restored.stat_based_roll_modifier, original.stat_based_roll_modifier);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("passRoll"));
    }
}
