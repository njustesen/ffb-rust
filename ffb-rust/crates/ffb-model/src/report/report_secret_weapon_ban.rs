use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportSecretWeaponBan.java`.
/// Stores parallel lists of player IDs, rolls, and ban flags.
#[derive(Debug, Clone, Default)]
pub struct ReportSecretWeaponBan {
    pub player_ids: Vec<String>,
    pub rolls: Vec<i32>,
    pub bans: Vec<bool>,
}

impl ReportSecretWeaponBan {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, player_id: String, roll: i32, banned: bool) {
        self.player_ids.push(player_id);
        self.rolls.push(roll);
        self.bans.push(banned);
    }

    pub fn get_player_ids(&self) -> &[String] { &self.player_ids }
    pub fn get_rolls(&self) -> &[i32] { &self.rolls }
    pub fn get_bans(&self) -> &[bool] { &self.bans }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerIds": self.player_ids,
            "rolls": self.rolls,
            "banArray": self.bans,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        let player_ids: Vec<String> = json["playerIds"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()).unwrap_or_default();
        let rolls: Vec<i32> = json["rolls"].as_array().map(|a| a.iter().map(|v| v.as_i64().unwrap_or(0) as i32).collect()).unwrap_or_default();
        let bans: Vec<bool> = json["banArray"].as_array().map(|a| a.iter().map(|v| v.as_bool().unwrap_or(false)).collect()).unwrap_or_default();
        Self { player_ids, rolls, bans }
    }
}

impl IReport for ReportSecretWeaponBan {
    fn get_id(&self) -> ReportId { ReportId::SECRET_WEAPON_BAN }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportSecretWeaponBan {
        let mut r = ReportSecretWeaponBan::new();
        r.add("p1".into(), 3, true);
        r.add("p2".into(), 5, false);
        r
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::SECRET_WEAPON_BAN);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "secretWeaponBan");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_player_ids(), &["p1", "p2"]);
        assert_eq!(r.get_rolls(), &[3, 5]);
        assert_eq!(r.get_bans(), &[true, false]);
    }

    #[test]
    fn empty_on_new() {
        let r = ReportSecretWeaponBan::new();
        assert_eq!(r.get_player_ids().len(), 0);
        assert_eq!(r.get_rolls().len(), 0);
        assert_eq!(r.get_bans().len(), 0);
    }

    #[test]
    fn single_entry() {
        let mut r = ReportSecretWeaponBan::new();
        r.add("p3".into(), 6, false);
        assert_eq!(r.get_player_ids(), &["p3"]);
        assert_eq!(r.get_rolls(), &[6]);
        assert_eq!(r.get_bans(), &[false]);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportSecretWeaponBan::from_json(&json);
        assert_eq!(restored.player_ids, original.player_ids);
        assert_eq!(restored.rolls, original.rolls);
        assert_eq!(restored.bans, original.bans);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("secretWeaponBan"));
    }
}
