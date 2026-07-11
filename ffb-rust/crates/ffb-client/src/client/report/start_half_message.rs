use crate::client::paragraph_style::ParagraphStyle;
use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_start_half::ReportStartHalf;

pub struct StartHalfMessage;

impl ReportMessage for StartHalfMessage {
    type Report = ReportStartHalf;

    fn report_id(&self) -> ReportId {
        ReportId::START_HALF
    }

    fn render(&self, status_report: &mut StatusReport, _game: &Game, report: &Self::Report) {
        let mut status = String::from("Starting ");
        if report.get_half() > 2 {
            status.push_str("Overtime");
        } else if report.get_half() > 1 {
            status.push_str("2nd half");
        } else {
            status.push_str("1st half");
        }
        status_report.println_style(Some(ParagraphStyle::SPACE_ABOVE_BELOW), Some(TextStyle::TURN), &status);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.to_string(), name: id.to_string(), race: "human".into(), roster_id: "human".into(),
            coach: "coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: Vec::new(), players: Vec::new(), vampire_lord: false, necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2025)
    }

    #[test]
    fn report_id_matches() {
        assert_eq!(StartHalfMessage.report_id(), ReportId::START_HALF);
    }

    #[test]
    fn first_half() {
        let game = make_game();
        let report = ReportStartHalf::new(1);
        let mut status_report = StatusReport::new();
        StartHalfMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Starting 1st half"));
        assert_eq!(status_report.rendered_runs[0].paragraph_style, Some(ParagraphStyle::SPACE_ABOVE_BELOW));
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::TURN));
    }

    #[test]
    fn second_half() {
        let game = make_game();
        let report = ReportStartHalf::new(2);
        let mut status_report = StatusReport::new();
        StartHalfMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Starting 2nd half"));
    }

    #[test]
    fn overtime() {
        let game = make_game();
        let report = ReportStartHalf::new(3);
        let mut status_report = StatusReport::new();
        StartHalfMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Starting Overtime"));
    }
}
