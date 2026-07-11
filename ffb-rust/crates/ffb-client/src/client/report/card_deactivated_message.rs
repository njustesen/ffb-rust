use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_card_deactivated::ReportCardDeactivated;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `CardDeactivatedMessage.java`.
pub struct CardDeactivatedMessage;

impl ReportMessage for CardDeactivatedMessage {
    type Report = ReportCardDeactivated;

    fn report_id(&self) -> ReportId {
        ReportId::CARD_DEACTIVATED
    }

    fn render(&self, status_report: &mut StatusReport, _game: &Game, report: &Self::Report) {
        let status = format!("Card {} effect ended.", report.get_card());
        status_report.println_indent_style(status_report.get_indent(), TextStyle::BOLD, &status);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::paragraph_style::ParagraphStyle;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team { id: id.to_string(), name: id.to_string(), race: "human".into(), roster_id: "human".into(),
            coach: "coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: Vec::new(), players: Vec::new(), vampire_lord: false, necromancer: false }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2025)
    }

    #[test]
    fn renders_card_name_and_ended_text() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportCardDeactivated::new("CUSTARD_PIE".into());
        CardDeactivatedMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Card CUSTARD_PIE effect ended."));
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::BOLD));
    }

    #[test]
    fn renders_different_card_name() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportCardDeactivated::new("ILLEGAL_PROCEDURE".into());
        CardDeactivatedMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Card ILLEGAL_PROCEDURE effect ended."));
    }

    #[test]
    fn respects_current_indent() {
        let mut status_report = StatusReport::new();
        status_report.set_indent(2);
        let game = make_game();
        let report = ReportCardDeactivated::new("BRIBE".into());
        CardDeactivatedMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].paragraph_style, Some(ParagraphStyle::INDENT_2));
    }

    #[test]
    fn report_id_is_card_deactivated() {
        assert_eq!(CardDeactivatedMessage.report_id(), ReportId::CARD_DEACTIVATED);
    }
}
