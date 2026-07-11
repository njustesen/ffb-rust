use crate::client::report::report_message_base::{map_to_local, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_throw_in::ReportThrowIn;

pub struct ThrowInMessage;

impl ReportMessage for ThrowInMessage {
    type Report = ReportThrowIn;

    fn report_id(&self) -> ReportId {
        ReportId::THROW_IN
    }

    fn render(&self, status_report: &mut StatusReport, _game: &Game, report: &Self::Report) {
        let direction_roll = report.get_direction_roll();
        let distance_roll = report.get_distance_roll();
        let direction = report.get_direction();
        // java: `(distanceRoll != null) && (distanceRoll.length > 1) && (direction != null)` —
        // Rust's `direction`/`distance_roll` fields are non-optional, so only the length check
        // is meaningful here.
        if distance_roll.len() > 1 {
            let indent = status_report.get_indent();
            let status = format!(
                "Throw In Direction Roll [ {} ] {}",
                direction_roll,
                map_to_local(direction).name()
            );
            status_report.println_indent_style(indent, TextStyle::ROLL, &status);
            let status = format!(
                "Throw In Distance Roll [ {} ][ {} ]",
                distance_roll[0], distance_roll[1]
            );
            status_report.println_indent_style(indent, TextStyle::ROLL, &status);
            status_report.println_indent(indent + 1, "The fans throw the ball back onto the pitch.");
            let distance = distance_roll[0] + distance_roll[1];
            let status = format!("It lands {} squares {}", distance, map_to_local(direction).name());
            status_report.println_indent(indent + 1, &status);
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
        assert_eq!(ThrowInMessage.report_id(), ReportId::THROW_IN);
    }

    #[test]
    fn full_throw_in_renders_all_lines() {
        let game = make_game();
        let report = ReportThrowIn::new(Direction::North, 3, vec![2, 4]);
        let mut status_report = StatusReport::new();
        ThrowInMessage.render(&mut status_report, &game, &report);
        let texts: Vec<Option<String>> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.contains(&Some("Throw In Direction Roll [ 3 ] North".to_string())));
        assert!(texts.contains(&Some("Throw In Distance Roll [ 2 ][ 4 ]".to_string())));
        assert!(texts.contains(&Some("The fans throw the ball back onto the pitch.".to_string())));
        assert!(texts.contains(&Some("It lands 6 squares North".to_string())));
    }

    #[test]
    fn single_element_distance_roll_renders_nothing() {
        let game = make_game();
        let report = ReportThrowIn::new(Direction::South, 1, vec![2]);
        let mut status_report = StatusReport::new();
        ThrowInMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }

    #[test]
    fn empty_distance_roll_renders_nothing() {
        let game = make_game();
        let report = ReportThrowIn::new(Direction::East, 1, vec![]);
        let mut status_report = StatusReport::new();
        ThrowInMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }
}
