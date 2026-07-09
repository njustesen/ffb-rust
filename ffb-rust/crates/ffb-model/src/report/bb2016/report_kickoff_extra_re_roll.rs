use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::enums::KickoffResult;

/// 1:1 translation of `ReportKickoffExtraReRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportKickoffExtraReRoll {
    pub kickoff_result: KickoffResult,
    pub roll_home: i32,
    pub home_gains_re_roll: bool,
    pub roll_away: i32,
    pub away_gains_re_roll: bool,
}

impl ReportKickoffExtraReRoll {
    pub fn new(
        kickoff_result: KickoffResult,
        roll_home: i32,
        home_gains_re_roll: bool,
        roll_away: i32,
        away_gains_re_roll: bool,
    ) -> Self {
        Self { kickoff_result, roll_home, home_gains_re_roll, roll_away, away_gains_re_roll }
    }

    pub fn get_kickoff_result(&self) -> KickoffResult { self.kickoff_result }
    pub fn get_roll_home(&self) -> i32 { self.roll_home }
    pub fn is_home_gains_re_roll(&self) -> bool { self.home_gains_re_roll }
    pub fn get_roll_away(&self) -> i32 { self.roll_away }
    pub fn is_away_gains_re_roll(&self) -> bool { self.away_gains_re_roll }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "kickoffResult": self.kickoff_result.name(),
            "rollHome": self.roll_home,
            "homeGainsReRoll": self.home_gains_re_roll,
            "rollAway": self.roll_away,
            "awayGainsReRoll": self.away_gains_re_roll,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            kickoff_result: json["kickoffResult"].as_str().and_then(KickoffResult::from_name).unwrap_or(KickoffResult::BrilliantCoaching),
            roll_home: json["rollHome"].as_i64().unwrap_or(0) as i32,
            home_gains_re_roll: json["homeGainsReRoll"].as_bool().unwrap_or(false),
            roll_away: json["rollAway"].as_i64().unwrap_or(0) as i32,
            away_gains_re_roll: json["awayGainsReRoll"].as_bool().unwrap_or(false),
        }
    }
}

impl IReport for ReportKickoffExtraReRoll {
    fn get_id(&self) -> ReportId { ReportId::KICKOFF_EXTRA_RE_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportKickoffExtraReRoll {
        ReportKickoffExtraReRoll::new(KickoffResult::BrilliantCoaching, 3, true, 2, false)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::KICKOFF_EXTRA_RE_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "extraReRoll");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_roll_home(), 3);
        assert!(r.is_home_gains_re_roll());
        assert!(!r.is_away_gains_re_roll());
    }

    #[test]
    fn roll_away_and_kickoff_result() {
        let r = make();
        assert_eq!(r.get_roll_away(), 2);
        assert_eq!(r.get_kickoff_result(), KickoffResult::BrilliantCoaching);
    }

    #[test]
    fn away_gains_reroll() {
        let r = ReportKickoffExtraReRoll::new(KickoffResult::BrilliantCoaching, 1, false, 5, true);
        assert!(!r.is_home_gains_re_roll());
        assert!(r.is_away_gains_re_roll());
        assert_eq!(r.get_roll_away(), 5);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportKickoffExtraReRoll::from_json(&json);
        assert_eq!(restored.roll_home, original.roll_home);
        assert_eq!(restored.roll_away, original.roll_away);
        assert_eq!(restored.home_gains_re_roll, original.home_gains_re_roll);
        assert_eq!(restored.kickoff_result, original.kickoff_result);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("extraReRoll"));
    }
}
