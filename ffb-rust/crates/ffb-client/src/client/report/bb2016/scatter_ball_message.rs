use crate::client::report::report_message_base::{map_to_local, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_scatter_ball::ReportScatterBall;

pub struct ScatterBallMessage;

impl ReportMessage for ScatterBallMessage {
    type Report = ReportScatterBall;

    fn report_id(&self) -> ReportId {
        ReportId::SCATTER_BALL
    }

    fn render(&self, status_report: &mut StatusReport, _game: &Game, report: &Self::Report) {
        if report.is_gust_of_wind() {
            let indent = status_report.get_indent() + 1;
            status_report.set_indent(indent);
            status_report.println_indent(indent, "A gust of wind scatters the ball.");
        }
        let rolls = report.get_rolls();
        if !rolls.is_empty() {
            let indent = status_report.get_indent();
            let mut status = if rolls.len() > 1 { String::from("Scatter Rolls [ ") } else { String::from("Scatter Roll [ ") };
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
        }
        if report.is_gust_of_wind() {
            let indent = status_report.get_indent() - 1;
            status_report.set_indent(indent);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Direction, Rules};
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.into(), name: format!("Team {id}"), race: "Human".into(), roster_id: "human".into(), coach: "Coach".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2016)
    }

    #[test]
    fn get_key_is_scatter_ball() {
        assert_eq!(ScatterBallMessage.get_key(), "scatterBall");
    }

    #[test]
    fn reports_single_scatter_roll() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportScatterBall::new(vec![Direction::North], vec![3], false);
        ScatterBallMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Scatter Roll [ 3 ] North"));
    }

    #[test]
    fn gust_of_wind_bumps_indent_and_reports_message() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportScatterBall::new(vec![Direction::East, Direction::South], vec![2, 4], true);
        ScatterBallMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("A gust of wind scatters the ball."));
        assert_eq!(status_report.get_indent(), 0);
    }

    #[test]
    fn empty_rolls_produce_no_roll_line() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportScatterBall::new(vec![], vec![], false);
        ScatterBallMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }
}
