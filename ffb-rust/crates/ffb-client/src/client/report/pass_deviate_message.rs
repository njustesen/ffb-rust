use crate::client::report::report_message_base::{map_to_local, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_pass_deviate::ReportPassDeviate;

/// 1:1 translation of `PassDeviateMessage.java`.
pub struct PassDeviateMessage;

impl ReportMessage for PassDeviateMessage {
    type Report = ReportPassDeviate;

    fn report_id(&self) -> ReportId {
        ReportId::PASS_DEVIATE
    }

    fn render(&self, status_report: &mut StatusReport, _game: &Game, report: &Self::Report) {
        status_report.set_indent(0);
        let action = if report.is_ttm() { "Throw Team Mate" } else { "Pass" };
        let thrown_entity = if report.is_ttm() { "player" } else { "ball" };
        let status = format!(
            "{} Deviates [ {} ][ {} ]",
            action,
            report.get_roll_scatter_direction(),
            report.get_roll_scatter_distance()
        );
        status_report.println_indent_style(status_report.get_indent(), TextStyle::ROLL, &status);
        let squares_word = if report.get_roll_scatter_distance() == 1 { " square " } else { " squares " };
        let status = format!(
            "The {} will land {}{}{} from the thrower.",
            thrown_entity,
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
    fn pass_deviate_uses_ball_wording() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportPassDeviate::new(FieldCoordinate::new(0, 0), Direction::North, 3, 2, false);
        PassDeviateMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.as_deref() == Some("Pass Deviates [ 3 ][ 2 ]")));
        assert!(texts.iter().any(|t| t.as_deref() == Some("The ball will land 2 squares north from the thrower.")));
    }

    #[test]
    fn ttm_deviate_uses_player_wording() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportPassDeviate::new(FieldCoordinate::new(0, 0), Direction::South, 1, 1, true);
        PassDeviateMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.as_deref() == Some("Throw Team Mate Deviates [ 1 ][ 1 ]")));
        assert!(texts.iter().any(|t| t.as_deref() == Some("The player will land 1 square south from the thrower.")));
    }

    #[test]
    fn indent_is_left_at_1_after_render() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportPassDeviate::new(FieldCoordinate::new(0, 0), Direction::East, 2, 4, false);
        PassDeviateMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.get_indent(), 1);
    }

    #[test]
    fn get_key_matches_report_id() {
        assert_eq!(PassDeviateMessage.get_key(), "passDeviate");
    }
}
