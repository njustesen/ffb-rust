use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportCardsAndInducementsBought.java`.
#[derive(Debug, Clone)]
pub struct ReportCardsAndInducementsBought {
    pub team_id: String,
    pub cards: i32,
    pub inducements: i32,
    pub stars: i32,
    pub mercenaries: i32,
    pub gold: i32,
    pub new_tv: i32,
}

impl ReportCardsAndInducementsBought {
    pub fn new(
        team_id: String,
        cards: i32,
        inducements: i32,
        stars: i32,
        mercenaries: i32,
        gold: i32,
        new_tv: i32,
    ) -> Self {
        Self { team_id, cards, inducements, stars, mercenaries, gold, new_tv }
    }

    pub fn get_team_id(&self) -> &str { &self.team_id }
    pub fn get_cards(&self) -> i32 { self.cards }
    pub fn get_inducements(&self) -> i32 { self.inducements }
    pub fn get_stars(&self) -> i32 { self.stars }
    pub fn get_mercenaries(&self) -> i32 { self.mercenaries }
    pub fn get_gold(&self) -> i32 { self.gold }
    pub fn get_new_tv(&self) -> i32 { self.new_tv }
}

impl IReport for ReportCardsAndInducementsBought {
    fn get_id(&self) -> ReportId { ReportId::CARDS_AND_INDUCEMENTS_BOUGHT }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportCardsAndInducementsBought {
        ReportCardsAndInducementsBought::new("team1".into(), 2, 1, 0, 0, 100000, 1200000)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::CARDS_AND_INDUCEMENTS_BOUGHT);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "cardsAndInducementsBought");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_team_id(), "team1");
        assert_eq!(r.get_cards(), 2);
        assert_eq!(r.get_gold(), 100000);
    }

    #[test]
    fn stars_mercenaries_and_inducements() {
        let r = make();
        assert_eq!(r.get_inducements(), 1);
        assert_eq!(r.get_stars(), 0);
        assert_eq!(r.get_mercenaries(), 0);
    }

    #[test]
    fn new_tv() {
        let r = ReportCardsAndInducementsBought::new("team2".into(), 0, 0, 1, 2, 50000, 900000);
        assert_eq!(r.get_new_tv(), 900000);
        assert_eq!(r.get_team_id(), "team2");
    }
}
