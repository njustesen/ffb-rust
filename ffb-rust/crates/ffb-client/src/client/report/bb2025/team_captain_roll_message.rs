use crate::client::report::report_message_base::{print_team_name, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::bb2025::report_team_captain_roll::ReportTeamCaptainRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `TeamCaptainRollMessage.java`.
pub struct TeamCaptainRollMessage;

impl ReportMessage for TeamCaptainRollMessage {
    type Report = ReportTeamCaptainRoll;

    fn report_id(&self) -> ReportId {
        ReportId::TEAM_CAPTAIN_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        status_report.println_indent_style(indent, TextStyle::ROLL, &format!("Team Captain Roll [ {} ]", report.get_roll()));
        print_team_name(status_report, game, false, report.get_team_id());
        let mut builder = String::from(" look to their Team Captain for guidance");
        if report.is_successful() {
            builder.push_str(" and save the re-roll.");
        } else {
            builder.push_str(" but nothing happens.");
        }
        status_report.println_indent(indent, &builder);

        if !report.is_successful() {
            status_report.println_indent_style(indent + 1, TextStyle::NONE, &format!("(Roll >= {} to succeed)", report.get_minimum_roll()));
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
    fn report_id_is_team_captain_roll() {
        assert_eq!(TeamCaptainRollMessage.report_id(), ReportId::TEAM_CAPTAIN_ROLL);
    }

    #[test]
    fn successful_roll_saves_reroll_without_minimum_note() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportTeamCaptainRoll::new("home".into(), 4, 5, true);
        TeamCaptainRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "Team Captain Roll [ 5 ]"));
        assert!(texts.iter().any(|t| t == " look to their Team Captain for guidance and save the re-roll."));
        assert!(!texts.iter().any(|t| t.contains("Roll >=")));
    }

    #[test]
    fn unsuccessful_roll_shows_minimum_note() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportTeamCaptainRoll::new("away".into(), 4, 2, false);
        TeamCaptainRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == " look to their Team Captain for guidance but nothing happens."));
        assert!(texts.iter().any(|t| t == "(Roll >= 4 to succeed)"));
    }

    #[test]
    fn team_name_printed_for_home_team() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportTeamCaptainRoll::new("home".into(), 3, 6, true);
        TeamCaptainRollMessage.render(&mut status_report, &game, &report);
        let team_run = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("Team home")).unwrap();
        assert_eq!(team_run.text_style, Some(TextStyle::HOME));
    }
}
