use crate::client::report::report_message_base::{print_team_name, ReportMessage};
use crate::client::status_report::StatusReport;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_receive_choice::ReportReceiveChoice;

/// 1:1 translation of `ReceiveChoiceMessage.java`.
pub struct ReceiveChoiceMessage;

impl ReportMessage for ReceiveChoiceMessage {
    type Report = ReportReceiveChoice;

    fn report_id(&self) -> ReportId {
        ReportId::RECEIVE_CHOICE
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        status_report.print_indent(status_report.get_indent() + 1, "Team ");
        print_team_name(status_report, game, false, report.get_team_id());
        let text = format!(" is {}", if report.is_receive_choice() { "receiving." } else { "kicking." });
        status_report.println_indent(status_report.get_indent() + 1, &text);
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
        assert_eq!(ReceiveChoiceMessage.report_id(), ReportId::RECEIVE_CHOICE);
    }

    #[test]
    fn render_receiving_home_team() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportReceiveChoice::new("home".into(), true);
        ReceiveChoiceMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Team "));
        assert_eq!(status_report.rendered_runs[1].text.as_deref(), Some("home"));
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some(" is receiving."));
    }

    #[test]
    fn render_kicking_away_team() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportReceiveChoice::new("away".into(), false);
        ReceiveChoiceMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some(" is kicking."));
    }

    #[test]
    fn render_uses_indent_plus_one() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        status_report.set_indent(2);
        let report = ReportReceiveChoice::new("home".into(), true);
        ReceiveChoiceMessage.render(&mut status_report, &game, &report);
        use crate::client::paragraph_style::ParagraphStyle;
        assert_eq!(status_report.rendered_runs[0].paragraph_style, Some(ParagraphStyle::INDENT_3));
    }
}
