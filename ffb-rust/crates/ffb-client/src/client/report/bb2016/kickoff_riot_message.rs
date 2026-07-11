use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::bb2016::report_kickoff_riot::ReportKickoffRiot;
use ffb_model::report::report_id::ReportId;

pub struct KickoffRiotMessage;

impl ReportMessage for KickoffRiotMessage {
    type Report = ReportKickoffRiot;

    fn report_id(&self) -> ReportId {
        ReportId::KICKOFF_RIOT
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let status = if report.get_roll() > 0 {
            format!("Riot Roll [ {} ]", report.get_roll())
        } else {
            let turn_nr = if game.home_playing { game.turn_data_away.turn_nr } else { game.turn_data_home.turn_nr };
            format!("Riot in Turn {}", turn_nr)
        };
        status_report.println_indent_style(indent, TextStyle::ROLL, &status);
        if report.get_turn_modifier() < 0 {
            status_report.println_indent(indent + 1, "The referee adjusts the clock back after the riot is over.");
            let steps = if report.get_turn_modifier() == -1 { "step" } else { "steps" };
            let status = format!("Turn Counter is moved {} {} backward.", report.get_turn_modifier().abs(), steps);
            status_report.println_indent(indent + 1, &status);
        } else {
            status_report.println_indent(indent + 1, "The referee does not stop the clock during the riot.");
            let steps = if report.get_turn_modifier() == -1 { "step" } else { "steps" };
            let status = format!("Turn Counter is moved {} {} forward.", report.get_turn_modifier().abs(), steps);
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
        Team { id: id.into(), name: format!("Team {id}"), race: "Human".into(), roster_id: "human".into(), coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0, special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2016)
    }

    #[test]
    fn get_key_is_kickoff_riot() {
        assert_eq!(KickoffRiotMessage.get_key(), "kickoffRiot");
    }

    #[test]
    fn positive_roll_reports_roll_value() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportKickoffRiot::new(4, -2);
        KickoffRiotMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Riot Roll [ 4 ]"));
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Turn Counter is moved 2 steps backward.")));
    }

    #[test]
    fn zero_roll_reports_turn_number() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportKickoffRiot::new(0, 1);
        KickoffRiotMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs[0].text.as_deref().unwrap().starts_with("Riot in Turn"));
    }

    #[test]
    fn positive_modifier_moves_forward() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportKickoffRiot::new(3, 1);
        KickoffRiotMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Turn Counter is moved 1 steps forward.")));
    }
}
