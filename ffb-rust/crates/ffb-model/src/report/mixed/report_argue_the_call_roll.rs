use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportArgueTheCallRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportArgueTheCallRoll {
    pub player_id: Option<String>,
    pub successful: bool,
    pub coach_banned: bool,
    pub roll: i32,
    pub stays_on_pitch: bool,
    pub friends_with_ref: bool,
    pub biased_refs: i32,
}

impl ReportArgueTheCallRoll {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        coach_banned: bool,
        roll: i32,
        stays_on_pitch: bool,
        friends_with_ref: bool,
        biased_refs: i32,
    ) -> Self {
        Self { player_id, successful, coach_banned, roll, stays_on_pitch, friends_with_ref, biased_refs }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn is_successful(&self) -> bool { self.successful }
    pub fn is_coach_banned(&self) -> bool { self.coach_banned }
    pub fn get_roll(&self) -> i32 { self.roll }
    pub fn is_stays_on_pitch(&self) -> bool { self.stays_on_pitch }
    pub fn is_friends_with_ref(&self) -> bool { self.friends_with_ref }
    pub fn get_biased_refs(&self) -> i32 { self.biased_refs }
}

impl IReport for ReportArgueTheCallRoll {
    fn get_id(&self) -> ReportId { ReportId::ARGUE_THE_CALL }
}

impl ReportArgueTheCallRoll {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.player_id,
            "successful": self.successful,
            "coachBanned": self.coach_banned,
            "roll": self.roll,
            "staysOnPitch": self.stays_on_pitch,
            "friendsWithRef": self.friends_with_ref,
            "biasedRefs": self.biased_refs,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json["playerId"].as_str().map(str::to_string),
            successful: json["successful"].as_bool().unwrap_or(false),
            coach_banned: json["coachBanned"].as_bool().unwrap_or(false),
            roll: json["roll"].as_i64().unwrap_or(0) as i32,
            stays_on_pitch: json["staysOnPitch"].as_bool().unwrap_or(false),
            friends_with_ref: json["friendsWithRef"].as_bool().unwrap_or(false),
            biased_refs: json["biasedRefs"].as_i64().unwrap_or(0) as i32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportArgueTheCallRoll {
        ReportArgueTheCallRoll::new(Some("p1".into()), true, false, 5, true, false, 1)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::ARGUE_THE_CALL); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "argueTheCall"); }

    #[test]
    fn get_roll() { assert_eq!(make().get_roll(), 5); }

    #[test]
    fn is_successful_and_stays_on_pitch() {
        assert!(make().is_successful());
        assert!(make().is_stays_on_pitch());
    }

    #[test]
    fn coach_banned_and_biased_refs() {
        assert!(!make().is_coach_banned());
        assert_eq!(make().get_biased_refs(), 1);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportArgueTheCallRoll::from_json(&json);
        assert_eq!(restored.player_id, original.player_id);
        assert_eq!(restored.successful, original.successful);
        assert_eq!(restored.coach_banned, original.coach_banned);
        assert_eq!(restored.roll, original.roll);
        assert_eq!(restored.stays_on_pitch, original.stays_on_pitch);
        assert_eq!(restored.friends_with_ref, original.friends_with_ref);
        assert_eq!(restored.biased_refs, original.biased_refs);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("argueTheCall"));
    }
}
