use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportInducementsBought.java`.
#[derive(Debug, Clone)]
pub struct ReportInducementsBought {
    pub team_id: String,
    pub nr_of_inducements: i32,
    pub nr_of_stars: i32,
    pub nr_of_mercenaries: i32,
    pub gold: i32,
}

impl ReportInducementsBought {
    pub fn new(
        team_id: String,
        nr_of_inducements: i32,
        nr_of_stars: i32,
        nr_of_mercenaries: i32,
        gold: i32,
    ) -> Self {
        Self { team_id, nr_of_inducements, nr_of_stars, nr_of_mercenaries, gold }
    }

    pub fn get_team_id(&self) -> &str { &self.team_id }
    pub fn get_nr_of_inducements(&self) -> i32 { self.nr_of_inducements }
    pub fn get_nr_of_stars(&self) -> i32 { self.nr_of_stars }
    pub fn get_nr_of_mercenaries(&self) -> i32 { self.nr_of_mercenaries }
    pub fn get_gold(&self) -> i32 { self.gold }
}

impl IReport for ReportInducementsBought {
    fn get_id(&self) -> ReportId { ReportId::INDUCEMENTS_BOUGHT }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportInducementsBought {
        ReportInducementsBought::new("team1".into(), 3, 1, 0, 150000)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::INDUCEMENTS_BOUGHT);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "inducementsBought");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_team_id(), "team1");
        assert_eq!(r.get_nr_of_inducements(), 3);
        assert_eq!(r.get_nr_of_stars(), 1);
        assert_eq!(r.get_gold(), 150000);
    }

    #[test]
    fn mercenaries_count() {
        let r = ReportInducementsBought::new("team2".into(), 2, 0, 1, 80000);
        assert_eq!(r.get_nr_of_mercenaries(), 1);
        assert_eq!(r.get_nr_of_stars(), 0);
    }

    #[test]
    fn zero_inducements() {
        let r = ReportInducementsBought::new("team3".into(), 0, 0, 0, 0);
        assert_eq!(r.get_nr_of_inducements(), 0);
        assert_eq!(r.get_gold(), 0);
    }
}
