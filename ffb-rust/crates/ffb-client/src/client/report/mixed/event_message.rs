use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_event::ReportEvent;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `EventMessage.java`.
pub struct EventMessage;

impl ReportMessage for EventMessage {
    type Report = ReportEvent;

    fn report_id(&self) -> ReportId {
        ReportId::EVENT
    }

    fn render(&self, status_report: &mut StatusReport, _game: &Game, report: &Self::Report) {
        status_report.println_indent_style(
            status_report.get_indent() + 1,
            TextStyle::NONE,
            report.get_event_message().unwrap_or(""),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn empty_team(id: &str) -> Team {
        Team {
            id: id.into(),
            name: id.into(),
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
        Game::new(empty_team("home"), empty_team("away"), Rules::Bb2020)
    }

    #[test]
    fn renders_event_message_at_indent_plus_one() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportEvent::new(Some("Something happened.".into()));
        EventMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some("Something happened."));
    }

    #[test]
    fn renders_at_base_indent_plus_one_regardless_of_current_indent() {
        let mut sr = StatusReport::new();
        sr.set_indent(3);
        let game = make_game();
        let report = ReportEvent::new(Some("Kickoff!".into()));
        EventMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some("Kickoff!"));
    }

    #[test]
    fn renders_empty_string_when_no_event_message() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportEvent::new(None);
        EventMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some(""));
    }
}
