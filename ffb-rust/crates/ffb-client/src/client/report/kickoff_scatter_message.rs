use crate::client::report::report_message_base::{map_to_local, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_kickoff_scatter::ReportKickoffScatter;

/// 1:1 translation of `KickoffScatterMessage.java`.
pub struct KickoffScatterMessage;

impl ReportMessage for KickoffScatterMessage {
    type Report = ReportKickoffScatter;

    fn report_id(&self) -> ReportId {
        ReportId::KICKOFF_SCATTER
    }

    fn render(&self, status_report: &mut StatusReport, _game: &Game, report: &Self::Report) {
        status_report.set_indent(0);
        let status = format!(
            "Kick-off Scatter Roll [ {} ][ {} ]",
            report.get_roll_scatter_direction(),
            report.get_roll_scatter_distance()
        );
        status_report.println_indent_style(status_report.get_indent(), TextStyle::ROLL, &status);
        let squares_word = if report.get_roll_scatter_distance() == 1 { " square " } else { " squares " };
        let status = format!(
            "The kick will land {}{}{} of where it was aimed.",
            report.get_roll_scatter_distance(),
            squares_word,
            map_to_local(report.get_scatter_direction()).name().to_lowercase()
        );
        status_report.println_indent(status_report.get_indent() + 1, &status);
        status_report.set_indent(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Direction;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;
    use ffb_model::types::FieldCoordinate;

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
    fn single_square_uses_singular_wording() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportKickoffScatter::new(FieldCoordinate::new(0, 0), Direction::North, 3, 1);
        KickoffScatterMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.as_deref() == Some("The kick will land 1 square north of where it was aimed.")));
    }

    #[test]
    fn multiple_squares_uses_plural_wording() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportKickoffScatter::new(FieldCoordinate::new(0, 0), Direction::South, 5, 3);
        KickoffScatterMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.as_deref() == Some("The kick will land 3 squares south of where it was aimed.")));
    }

    #[test]
    fn roll_header_shows_direction_and_distance_rolls() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportKickoffScatter::new(FieldCoordinate::new(0, 0), Direction::East, 2, 4);
        KickoffScatterMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.as_deref() == Some("Kick-off Scatter Roll [ 2 ][ 4 ]")));
    }

    #[test]
    fn indent_is_left_at_1_after_render() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportKickoffScatter::new(FieldCoordinate::new(0, 0), Direction::East, 2, 4);
        KickoffScatterMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.get_indent(), 1);
    }

    #[test]
    fn get_key_matches_report_id() {
        assert_eq!(KickoffScatterMessage.get_key(), "kickoffScatter");
    }
}
