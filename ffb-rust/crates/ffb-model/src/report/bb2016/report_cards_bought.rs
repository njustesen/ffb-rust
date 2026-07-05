use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportCardsBought.java`.
#[derive(Debug, Clone)]
pub struct ReportCardsBought {
    pub team_id: String,
    pub nr_of_cards: i32,
    pub gold: i32,
}

impl ReportCardsBought {
    pub fn new(team_id: String, nr_of_cards: i32, gold: i32) -> Self {
        Self { team_id, nr_of_cards, gold }
    }

    pub fn get_team_id(&self) -> &str { &self.team_id }
    pub fn get_nr_of_cards(&self) -> i32 { self.nr_of_cards }
    pub fn get_gold(&self) -> i32 { self.gold }
}

impl IReport for ReportCardsBought {
    fn get_id(&self) -> ReportId { ReportId::CARDS_BOUGHT }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportCardsBought {
        ReportCardsBought::new("team1".into(), 2, 50000)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::CARDS_BOUGHT);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "cardsBought");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_team_id(), "team1");
        assert_eq!(r.get_nr_of_cards(), 2);
        assert_eq!(r.get_gold(), 50000);
    }
}
