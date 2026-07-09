use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportThrowTeamMateRoll.java` — extends ReportSkillRoll.
#[derive(Debug, Clone)]
pub struct ReportThrowTeamMateRoll {
    pub base: ReportSkillRoll,
    /// `fThrownPlayerId`
    pub thrown_player_id: Option<String>,
    /// `fPassingDistance` — distance category name.
    pub passing_distance: Option<String>,
    /// `passResult` — PassResult name string.
    pub pass_result: Option<String>,
    pub is_kick: bool,
}

impl ReportThrowTeamMateRoll {
    pub fn new(
        thrower_id: Option<String>,
        successful: bool,
        roll: i32,
        minimum_roll: i32,
        re_rolled: bool,
        roll_modifiers: Vec<String>,
        passing_distance: Option<String>,
        thrown_player_id: Option<String>,
        pass_result: Option<String>,
        is_kick: bool,
    ) -> Self {
        Self {
            base: ReportSkillRoll::new(thrower_id, successful, roll, minimum_roll, re_rolled, roll_modifiers),
            thrown_player_id,
            passing_distance,
            pass_result,
            is_kick,
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.base.get_player_id() }
    pub fn is_successful(&self) -> bool { self.base.is_successful() }
    pub fn get_roll(&self) -> i32 { self.base.get_roll() }
    pub fn get_minimum_roll(&self) -> i32 { self.base.get_minimum_roll() }
    pub fn is_re_rolled(&self) -> bool { self.base.is_re_rolled() }
    pub fn get_roll_modifiers(&self) -> &[String] { self.base.get_roll_modifiers() }
    pub fn get_thrown_player_id(&self) -> Option<&str> { self.thrown_player_id.as_deref() }
    pub fn get_passing_distance(&self) -> Option<&str> { self.passing_distance.as_deref() }
    pub fn get_pass_result(&self) -> Option<&str> { self.pass_result.as_deref() }
    pub fn is_kick(&self) -> bool { self.is_kick }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.base.player_id,
            "successful": self.base.successful,
            "roll": self.base.roll,
            "minimumRoll": self.base.minimum_roll,
            "reRolled": self.base.re_rolled,
            "rollModifiers": self.base.roll_modifier_names,
            "thrownPlayerId": self.thrown_player_id,
            "passingDistance": self.passing_distance,
            "passResult": self.pass_result,
            "kicked": self.is_kick,
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
            thrown_player_id: json["thrownPlayerId"].as_str().map(str::to_string),
            passing_distance: json["passingDistance"].as_str().map(str::to_string),
            pass_result: json["passResult"].as_str().map(str::to_string),
            is_kick: json["kicked"].as_bool().unwrap_or(false),
        }
    }
}

impl IReport for ReportThrowTeamMateRoll {
    fn get_id(&self) -> ReportId { ReportId::THROW_TEAM_MATE_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportThrowTeamMateRoll {
        ReportThrowTeamMateRoll::new(
            Some("thrower".into()), true, 4, 3, false, vec![],
            Some("SHORT_PASS".into()), Some("thrown".into()), Some("ACCURATE".into()), false,
        )
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::THROW_TEAM_MATE_ROLL); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "throwTeamMateRoll"); }

    #[test]
    fn get_thrown_player_id() { assert_eq!(make().get_thrown_player_id(), Some("thrown")); }

    #[test]
    fn get_pass_result() { assert_eq!(make().get_pass_result(), Some("ACCURATE")); }

    #[test]
    fn is_kick() { assert!(!make().is_kick()); }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportThrowTeamMateRoll::from_json(&json);
        assert_eq!(restored.base.player_id, original.base.player_id);
        assert_eq!(restored.base.successful, original.base.successful);
        assert_eq!(restored.base.roll, original.base.roll);
        assert_eq!(restored.base.minimum_roll, original.base.minimum_roll);
        assert_eq!(restored.base.re_rolled, original.base.re_rolled);
        assert_eq!(restored.thrown_player_id, original.thrown_player_id);
        assert_eq!(restored.passing_distance, original.passing_distance);
        assert_eq!(restored.pass_result, original.pass_result);
        assert_eq!(restored.is_kick, original.is_kick);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("throwTeamMateRoll"));
    }
}
