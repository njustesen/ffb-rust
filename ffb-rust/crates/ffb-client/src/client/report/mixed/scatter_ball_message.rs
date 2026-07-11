use crate::client::report::report_message_base::{map_to_local, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_scatter_ball::ReportScatterBall;

/// 1:1 translation of `ScatterBallMessage.java`.
pub struct ScatterBallMessage;

impl ReportMessage for ScatterBallMessage {
    type Report = ReportScatterBall;

    fn report_id(&self) -> ReportId {
        ReportId::SCATTER_BALL
    }

    fn render(&self, status_report: &mut StatusReport, _game: &Game, report: &Self::Report) {
        let mut status = String::new();
        if report.is_gust_of_wind() {
            status_report.set_indent(status_report.get_indent() + 1);
            status.push_str("A gust of wind scatters the ball.");
            status_report.println_indent(status_report.get_indent(), &status);
            status = String::new();
        }
        let rolls = report.get_rolls();
        if !rolls.is_empty() {
            if rolls.len() > 1 {
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
            status_report.println_indent_style(status_report.get_indent(), TextStyle::ROLL, &status);
        }
        if report.is_gust_of_wind() {
            status_report.set_indent(status_report.get_indent() - 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Direction;

    #[test]
    fn single_roll_uses_bounce_wording() {
        let mut status_report = StatusReport::new();
        let game = Game::new(empty_team(), empty_team(), ffb_model::enums::Rules::Bb2020);
        let report = ReportScatterBall::new(vec![Direction::North], vec![3], false);
        ScatterBallMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Bounce Roll [ 3 ] North"));
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::ROLL));
    }

    #[test]
    fn multi_roll_uses_scatter_wording_and_joins_directions() {
        let mut status_report = StatusReport::new();
        let game = Game::new(empty_team(), empty_team(), ffb_model::enums::Rules::Bb2020);
        let report = ReportScatterBall::new(vec![Direction::North, Direction::East], vec![3, 5], false);
        ScatterBallMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Scatter Rolls [ 3, 5 ] North, East"));
    }

    #[test]
    fn gust_of_wind_prints_extra_line_at_incremented_indent_and_restores() {
        let mut status_report = StatusReport::new();
        let game = Game::new(empty_team(), empty_team(), ffb_model::enums::Rules::Bb2020);
        let report = ReportScatterBall::new(vec![Direction::North], vec![3], true);
        ScatterBallMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("A gust of wind scatters the ball."));
        // indent restored to 0 after rendering.
        assert_eq!(status_report.get_indent(), 0);
    }

    #[test]
    fn empty_rolls_prints_nothing_but_gust_line() {
        let mut status_report = StatusReport::new();
        let game = Game::new(empty_team(), empty_team(), ffb_model::enums::Rules::Bb2020);
        let report = ReportScatterBall::new(vec![], vec![], true);
        ScatterBallMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert_eq!(texts, vec!["A gust of wind scatters the ball."]);
    }

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
}
