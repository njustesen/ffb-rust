use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportSkillUseOtherPlayer.java`.
#[derive(Debug, Clone)]
pub struct ReportSkillUseOtherPlayer {
    pub player_id: String,
    pub other_player_id: String,
    pub skill: String,
    pub skill_use: String,
}

impl ReportSkillUseOtherPlayer {
    pub fn new(player_id: String, skill: String, skill_use: String, other_player_id: String) -> Self {
        Self { player_id, other_player_id, skill, skill_use }
    }

    pub fn get_player_id(&self) -> &str { &self.player_id }
    pub fn get_other_player_id(&self) -> &str { &self.other_player_id }
    pub fn get_skill(&self) -> &str { &self.skill }
    pub fn get_skill_use(&self) -> &str { &self.skill_use }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.player_id,
            "playerIdOtherPlayer": self.other_player_id,
            "skill": self.skill,
            "skillUse": self.skill_use,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json["playerId"].as_str().unwrap_or("").to_string(),
            other_player_id: json["playerIdOtherPlayer"].as_str().unwrap_or("").to_string(),
            skill: json["skill"].as_str().unwrap_or("").to_string(),
            skill_use: json["skillUse"].as_str().unwrap_or("").to_string(),
        }
    }
}

impl IReport for ReportSkillUseOtherPlayer {
    fn get_id(&self) -> ReportId { ReportId::SKILL_USE_OTHER_PLAYER }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportSkillUseOtherPlayer {
        ReportSkillUseOtherPlayer::new("p1".into(), "Block".into(), "USE".into(), "p2".into())
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::SKILL_USE_OTHER_PLAYER);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "skillUseOtherPlayer");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_player_id(), "p1");
        assert_eq!(r.get_other_player_id(), "p2");
        assert_eq!(r.get_skill(), "Block");
    }

    #[test]
    fn skill_use_field() {
        let r = make();
        assert_eq!(r.get_skill_use(), "USE");
    }

    #[test]
    fn different_skill_and_use() {
        let r = ReportSkillUseOtherPlayer::new("p3".into(), "Dodge".into(), "CANCEL".into(), "p4".into());
        assert_eq!(r.get_skill(), "Dodge");
        assert_eq!(r.get_skill_use(), "CANCEL");
        assert_eq!(r.get_other_player_id(), "p4");
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportSkillUseOtherPlayer::from_json(&json);
        assert_eq!(restored.player_id, original.player_id);
        assert_eq!(restored.other_player_id, original.other_player_id);
        assert_eq!(restored.skill, original.skill);
        assert_eq!(restored.skill_use, original.skill_use);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("skillUseOtherPlayer"));
    }
}
