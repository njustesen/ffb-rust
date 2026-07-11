use crate::client::report::report_message_base::{print_team_name, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::bb2025::report_team_event::ReportTeamEvent;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `TeamEventMessage.java`.
pub struct TeamEventMessage;

impl ReportMessage for TeamEventMessage {
    type Report = ReportTeamEvent;

    fn report_id(&self) -> ReportId {
        ReportId::TEAM_EVENT
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        print_team_name(status_report, game, false, report.get_team_id());
        let indent = status_report.get_indent();
        status_report.println_indent_style(indent + 1, TextStyle::NONE, &format!(" {}", report.get_event_message()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.into(), name: format!("Team {id}"), race: "Human".into(), roster_id: "human".into(),
            coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2025)
    }

    #[test]
    fn report_id_is_team_event() {
        assert_eq!(TeamEventMessage.report_id(), ReportId::TEAM_EVENT);
    }

    #[test]
    fn renders_home_team_name_and_message() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportTeamEvent::new("home".into(), "Player banned!".into());
        TeamEventMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "Team home"));
        assert!(texts.iter().any(|t| t == " Player banned!"));
    }

    #[test]
    fn renders_away_team_name_and_message() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportTeamEvent::new("away".into(), "Mascot injured!".into());
        TeamEventMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "Team away"));
        assert!(texts.iter().any(|t| t == " Mascot injured!"));
    }

    #[test]
    fn message_uses_none_text_style_at_indent_plus_one() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        status_report.set_indent(1);
        let report = ReportTeamEvent::new("home".into(), "Test".into());
        TeamEventMessage.render(&mut status_report, &game, &report);
        let run = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some(" Test")).unwrap();
        assert_eq!(run.text_style, Some(TextStyle::NONE));
    }
}
