use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportPrayersAndInducementsBought.java`.
#[derive(Debug, Clone)]
pub struct ReportPrayersAndInducementsBought {
    pub team_id: String,
    pub inducements: i32,
    pub stars: i32,
    pub mercenaries: i32,
    pub gold: i32,
    pub new_tv: i32,
}

impl ReportPrayersAndInducementsBought {
    pub fn new(team_id: String, inducements: i32, stars: i32, mercenaries: i32, gold: i32, new_tv: i32) -> Self {
        Self { team_id, inducements, stars, mercenaries, gold, new_tv }
    }

    pub fn get_team_id(&self) -> &str { &self.team_id }
    pub fn get_inducements(&self) -> i32 { self.inducements }
    pub fn get_stars(&self) -> i32 { self.stars }
    pub fn get_mercenaries(&self) -> i32 { self.mercenaries }
    pub fn get_gold(&self) -> i32 { self.gold }
    pub fn get_new_tv(&self) -> i32 { self.new_tv }
}

impl IReport for ReportPrayersAndInducementsBought {
    fn get_id(&self) -> ReportId { ReportId::PRAYERS_AND_INDUCEMENTS_BOUGHT }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPrayersAndInducementsBought {
        ReportPrayersAndInducementsBought::new("team1".into(), 2, 1, 0, 150000, 1100000)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::PRAYERS_AND_INDUCEMENTS_BOUGHT);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "prayersAndInducementsBought");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_team_id(), "team1");
        assert_eq!(r.get_inducements(), 2);
        assert_eq!(r.get_gold(), 150000);
    }

    #[test]
    fn stars_mercenaries_and_new_tv() {
        let r = make();
        assert_eq!(r.get_stars(), 1);
        assert_eq!(r.get_mercenaries(), 0);
        assert_eq!(r.get_new_tv(), 1100000);
    }

    #[test]
    fn different_team() {
        let r = ReportPrayersAndInducementsBought::new("team2".into(), 0, 2, 1, 200000, 950000);
        assert_eq!(r.get_team_id(), "team2");
        assert_eq!(r.get_stars(), 2);
        assert_eq!(r.get_mercenaries(), 1);
    }
}
