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
}
