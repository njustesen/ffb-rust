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
}
