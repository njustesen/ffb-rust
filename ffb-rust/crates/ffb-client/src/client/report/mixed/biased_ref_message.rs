use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_biased_ref::ReportBiasedRef;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `BiasedRefMessage.java`.
pub struct BiasedRefMessage;

impl ReportMessage for BiasedRefMessage {
    type Report = ReportBiasedRef;

    fn report_id(&self) -> ReportId {
        ReportId::BIASED_REF
    }

    fn render(&self, status_report: &mut StatusReport, _game: &Game, report: &Self::Report) {
        status_report.println_indent_style(status_report.get_indent(), TextStyle::ROLL, &format!("Biased Roll [ {} ]", report.get_roll()));
        if report.is_foul_spotted() {
            status_report.println_indent_style(status_report.get_indent(), TextStyle::NONE, "The biased referee spots the foul.");
        } else {
            status_report.println_indent_style(status_report.get_indent(), TextStyle::NONE, "The biased referee does not spot the foul.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::game::Game;

    fn game() -> Game {
        use ffb_model::enums::Rules;
        use ffb_model::model::team::Team;
        let team = |id: &str| Team {
            id: id.into(),
            name: format!("Team {id}"),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: "Coach".into(),
            rerolls: 0,
            apothecaries: 0,
            bribes: 0,
            master_chefs: 0,
            prayers_to_nuffle: 0,
            bloodweiser_kegs: 0,
            riotous_rookies: 0,
            cheerleaders: 0,
            assistant_coaches: 0,
            fan_factor: 0,
            dedicated_fans: 0,
            team_value: 0,
            treasury: 0,
            special_rules: vec![],
            players: vec![],
            vampire_lord: false,
            necromancer: false,
        };
        Game::new(team("home"), team("away"), Rules::Bb2020)
    }

    #[test]
    fn foul_spotted_reports_spotted() {
        let mut sr = StatusReport::new();
        let report = ReportBiasedRef::new(true, 4);
        BiasedRefMessage.render(&mut sr, &game(), &report);
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some("Biased Roll [ 4 ]"));
        assert_eq!(sr.rendered_runs[2].text.as_deref(), Some("The biased referee spots the foul."));
    }

    #[test]
    fn foul_not_spotted_reports_not_spotted() {
        let mut sr = StatusReport::new();
        let report = ReportBiasedRef::new(false, 2);
        BiasedRefMessage.render(&mut sr, &game(), &report);
        assert_eq!(sr.rendered_runs[2].text.as_deref(), Some("The biased referee does not spot the foul."));
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(BiasedRefMessage.report_id(), ReportId::BIASED_REF);
        assert_eq!(BiasedRefMessage.get_key(), "biasedRef");
    }
}
