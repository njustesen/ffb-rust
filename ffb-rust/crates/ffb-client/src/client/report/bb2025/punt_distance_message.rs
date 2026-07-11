use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::bb2025::report_punt_distance::ReportPuntDistance;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `PuntDistanceMessage.java`.
pub struct PuntDistanceMessage;

impl ReportMessage for PuntDistanceMessage {
    type Report = ReportPuntDistance;

    fn report_id(&self) -> ReportId {
        ReportId::PUNT_DISTANCE_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, _game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let distance_roll = report.get_roll();
        if distance_roll > 0 {
            status_report.println_indent_style(indent, TextStyle::ROLL, &format!("Punt Distance Roll [ {distance_roll} ]"));
            let mut status = format!("The ball is punted {distance_roll} squares");
            if report.is_out_of_bounds() {
                status.push_str(" putting it out of bounds");
            }
            status_report.println_indent(indent + 1, &status);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

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
    fn positive_roll_reports_distance() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPuntDistance::new(4, false);
        PuntDistanceMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.contains(&"Punt Distance Roll [ 4 ]".to_string()));
        assert!(texts.contains(&"The ball is punted 4 squares".to_string()));
    }

    #[test]
    fn out_of_bounds_appends_text() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPuntDistance::new(3, true);
        PuntDistanceMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.contains(&"The ball is punted 3 squares putting it out of bounds".to_string()));
    }

    #[test]
    fn zero_roll_prints_nothing() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPuntDistance::new(0, false);
        PuntDistanceMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }

    #[test]
    fn negative_roll_prints_nothing() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPuntDistance::new(-1, false);
        PuntDistanceMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }

    #[test]
    fn report_id_is_punt_distance_roll() {
        assert_eq!(PuntDistanceMessage.report_id(), ReportId::PUNT_DISTANCE_ROLL);
    }
}
