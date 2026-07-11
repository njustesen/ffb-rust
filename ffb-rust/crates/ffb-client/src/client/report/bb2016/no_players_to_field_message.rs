use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::paragraph_style::ParagraphStyle;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::bb2016::report_no_players_to_field::ReportNoPlayersToField;
use ffb_model::report::report_id::ReportId;

pub struct NoPlayersToFieldMessage;

impl ReportMessage for NoPlayersToFieldMessage {
    type Report = ReportNoPlayersToField;

    fn report_id(&self) -> ReportId {
        ReportId::NO_PLAYERS_TO_FIELD
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        status_report.set_indent(0);
        let team_id = report.get_team_id();
        if !team_id.is_empty() {
            if game.team_home.id == team_id {
                let status = format!("{} can field no players.", game.team_home.name);
                status_report.println_style(Some(ParagraphStyle::SPACE_ABOVE_BELOW), Some(TextStyle::TURN_HOME), &status);
            } else {
                let status = format!("{} can field no players.", game.team_away.name);
                status_report.println_style(Some(ParagraphStyle::SPACE_ABOVE_BELOW), Some(TextStyle::TURN_AWAY), &status);
            }
        } else {
            status_report.println_style(Some(ParagraphStyle::SPACE_ABOVE_BELOW), Some(TextStyle::TURN), "Both teams can field no players.");
        }
        if !team_id.is_empty() {
            let indent = status_report.get_indent();
            status_report.println_indent_style(indent, TextStyle::BOLD, "The opposing team is awarded a touchdown.");
        }
        status_report.println_style(Some(ParagraphStyle::SPACE_BELOW), Some(TextStyle::BOLD), "The turn counter is advanced 2 steps.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
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
    fn get_key_is_no_players_to_field() {
        assert_eq!(NoPlayersToFieldMessage.get_key(), "noPlayersToField");
    }

    #[test]
    fn team_id_provided_reports_touchdown_award() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportNoPlayersToField::new("home".into());
        NoPlayersToFieldMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Team home can field no players.")));
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("The opposing team is awarded a touchdown.")));
    }

    #[test]
    fn no_team_id_reports_both_teams() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportNoPlayersToField::new("".into());
        NoPlayersToFieldMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Both teams can field no players.")));
        assert!(!status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("The opposing team is awarded a touchdown.")));
    }

    #[test]
    fn always_reports_turn_advance() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportNoPlayersToField::new("away".into());
        NoPlayersToFieldMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("The turn counter is advanced 2 steps.")));
    }
}
