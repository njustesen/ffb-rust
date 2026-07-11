use crate::client::report::report_message_base::{map_to_local, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::bb2020::report_swoop_player::ReportSwoopPlayer;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `SwoopPlayerMessage.java`.
pub struct SwoopPlayerMessage;

impl ReportMessage for SwoopPlayerMessage {
    type Report = ReportSwoopPlayer;

    fn report_id(&self) -> ReportId {
        ReportId::SWOOP_PLAYER
    }

    fn render(&self, status_report: &mut StatusReport, _game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let direction_name = map_to_local(report.get_direction()).name();
        let status = format!("Swoop Roll [ {} ] in direction {}", report.get_distance(), direction_name);
        status_report.println_indent_style(indent, TextStyle::ROLL, &status);

        let start = report.get_start_coordinate();
        let end = report.get_end_coordinate();
        let status = format!(
            "Player swoops from square ({},{}) to square ({},{}).",
            start.x, start.y, end.x, end.y
        );
        status_report.println_indent(indent + 1, &status);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Direction, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;
    use ffb_model::types::FieldCoordinate;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.into(),
            name: "Team".into(),
            race: String::new(),
            roster_id: String::new(),
            coach: String::new(),
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
            players: Vec::<Player>::new(),
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2020)
    }

    fn texts(status_report: &StatusReport) -> Vec<&str> {
        status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect()
    }

    #[test]
    fn renders_roll_and_squares() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportSwoopPlayer::new(FieldCoordinate::new(5, 7), FieldCoordinate::new(8, 7), Direction::East, 3);
        SwoopPlayerMessage.render(&mut status_report, &game, &report);
        let t = texts(&status_report);
        assert!(t.contains(&"Swoop Roll [ 3 ] in direction East"));
        assert!(t.contains(&"Player swoops from square (5,7) to square (8,7)."));
    }

    #[test]
    fn different_direction_and_distance() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportSwoopPlayer::new(FieldCoordinate::new(0, 0), FieldCoordinate::new(0, 2), Direction::North, 2);
        SwoopPlayerMessage.render(&mut status_report, &game, &report);
        let t = texts(&status_report);
        assert!(t.contains(&"Swoop Roll [ 2 ] in direction North"));
        assert!(t.contains(&"Player swoops from square (0,0) to square (0,2)."));
    }

    #[test]
    fn roll_line_uses_roll_style() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportSwoopPlayer::new(FieldCoordinate::new(1, 1), FieldCoordinate::new(2, 2), Direction::Southeast, 1);
        SwoopPlayerMessage.render(&mut status_report, &game, &report);
        let roll_run = status_report.rendered_runs.iter().find(|r| r.text.is_some()).unwrap();
        assert_eq!(roll_run.text_style, Some(TextStyle::ROLL));
    }
}
