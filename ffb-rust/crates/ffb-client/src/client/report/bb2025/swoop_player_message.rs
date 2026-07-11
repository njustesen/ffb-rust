use crate::client::report::report_message_base::{map_to_local, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::bb2025::report_swoop_player::ReportSwoopPlayer;
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
        let status = format!("Swoop Roll [ {} ] in direction {direction_name}", report.get_distance());
        status_report.println_indent_style(indent, TextStyle::ROLL, &status);

        let start = report.get_start_coordinate();
        let mut status = format!("Player swoops from square ({},{}", start.x, start.y);
        if report.is_out_of_bounds() {
            status.push_str(") into the fans");
        } else {
            let end = report.get_end_coordinate();
            status.push_str(&format!(") to square ({},{}", end.x, end.y));
        }
        status.push(')');
        status.push('.');
        status_report.println_indent(indent + 1, &status);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Direction, Rules};
    use ffb_model::model::team::Team;
    use ffb_model::types::FieldCoordinate;

    fn make_team(id: &str) -> Team {
        Team {
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
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2025)
    }

    #[test]
    fn report_id_is_swoop_player() {
        assert_eq!(SwoopPlayerMessage.report_id(), ReportId::SWOOP_PLAYER);
    }

    #[test]
    fn in_bounds_reports_end_square() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportSwoopPlayer::new(FieldCoordinate::new(3, 5), FieldCoordinate::new(6, 5), Direction::East, 3, false);
        SwoopPlayerMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "Swoop Roll [ 3 ] in direction East"));
        assert!(texts.iter().any(|t| t == "Player swoops from square (3,5) to square (6,5)."));
    }

    #[test]
    fn out_of_bounds_reports_fans() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportSwoopPlayer::new(FieldCoordinate::new(0, 0), FieldCoordinate::new(0, 3), Direction::North, 3, true);
        SwoopPlayerMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        // java: the Java source always appends `").")` after either branch, so the
        // out-of-bounds branch (which already closed with `") into the fans"`) ends up with
        // a doubled closing paren — faithfully reproduced here rather than "fixed".
        assert!(texts.iter().any(|t| t == "Player swoops from square (0,0) into the fans)."));
    }

    #[test]
    fn roll_header_uses_roll_style() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportSwoopPlayer::new(FieldCoordinate::new(1, 1), FieldCoordinate::new(1, 4), Direction::South, 3, false);
        SwoopPlayerMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::ROLL));
    }
}
