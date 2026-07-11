use crate::client::report::report_message_base::{map_to_local, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_scatter_player::ReportScatterPlayer;

/// 1:1 translation of `ScatterPlayerMessage.java`.
pub struct ScatterPlayerMessage;

impl ReportMessage for ScatterPlayerMessage {
    type Report = ReportScatterPlayer;

    fn report_id(&self) -> ReportId {
        ReportId::SCATTER_PLAYER
    }

    fn render(&self, status_report: &mut StatusReport, _game: &Game, report: &Self::Report) {
        let rolls = report.get_rolls();
        if rolls.is_empty() {
            return;
        }
        let indent = status_report.get_indent();
        let scatters = report.get_scatter().unwrap_or(rolls.len() > 1);

        let mut status = String::new();
        if scatters {
            status.push_str("Scatter Rolls [ ");
        } else {
            status.push_str("Bounce Roll [ ");
        }
        for (i, roll) in rolls.iter().enumerate() {
            if i > 0 {
                status.push_str(", ");
            }
            status.push_str(&roll.to_string());
        }
        status.push_str(" ] ");
        let directions = report.get_directions();
        for (i, direction) in directions.iter().enumerate() {
            if i > 0 {
                status.push_str(", ");
            }
            status.push_str(map_to_local(*direction).name());
        }
        status_report.println_indent_style(indent, TextStyle::ROLL, &status);

        let mut status = String::from("Player ");
        if scatters {
            status.push_str("scatters");
        } else {
            status.push_str("bounces");
        }
        status.push_str(" from square (");
        status.push_str(&report.get_start_coordinate().x.to_string());
        status.push(',');
        status.push_str(&report.get_start_coordinate().y.to_string());
        status.push_str(") to square (");
        status.push_str(&report.get_end_coordinate().x.to_string());
        status.push(',');
        status.push_str(&report.get_end_coordinate().y.to_string());
        status.push_str(").");
        status_report.println_indent(indent + 1, &status);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Direction, Rules};
    use ffb_model::types::FieldCoordinate;

    fn empty_team() -> ffb_model::model::team::Team {
        ffb_model::model::team::Team {
            id: "t".into(),
            name: "Team".into(),
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
        Game::new(empty_team(), empty_team(), Rules::Bb2020)
    }

    #[test]
    fn single_roll_without_explicit_scatter_flag_bounces() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportScatterPlayer::new(
            FieldCoordinate::new(3, 5),
            FieldCoordinate::new(4, 5),
            vec![Direction::East],
            vec![3],
            None,
        );
        ScatterPlayerMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Bounce Roll [ 3 ] East"));
        let second = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).nth(1).unwrap();
        assert_eq!(second, "Player bounces from square (3,5) to square (4,5).");
    }

    #[test]
    fn multi_roll_without_explicit_scatter_flag_scatters() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportScatterPlayer::new(
            FieldCoordinate::new(0, 0),
            FieldCoordinate::new(2, 1),
            vec![Direction::North, Direction::East],
            vec![3, 4],
            None,
        );
        ScatterPlayerMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Scatter Rolls [ 3, 4 ] North, East"));
        let second = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).nth(1).unwrap();
        assert_eq!(second, "Player scatters from square (0,0) to square (2,1).");
    }

    #[test]
    fn explicit_scatter_flag_overrides_roll_count() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportScatterPlayer::new(
            FieldCoordinate::new(0, 0),
            FieldCoordinate::new(1, 0),
            vec![Direction::West],
            vec![4],
            Some(true),
        );
        ScatterPlayerMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Scatter Rolls [ 4 ] West"));
    }

    #[test]
    fn empty_rolls_renders_nothing() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportScatterPlayer::new(
            FieldCoordinate::new(0, 0),
            FieldCoordinate::new(1, 0),
            vec![],
            vec![],
            None,
        );
        ScatterPlayerMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }
}
