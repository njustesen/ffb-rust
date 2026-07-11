use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::bb2025::report_saboteur_roll::ReportSaboteurRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `SaboteurRollMessage.java`.
pub struct SaboteurRollMessage;

impl ReportMessage for SaboteurRollMessage {
    type Report = ReportSaboteurRoll;

    fn report_id(&self) -> ReportId {
        ReportId::SABOTEUR_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        status_report.println_indent_style(indent, TextStyle::ROLL, &format!("Saboteur Roll [ {} ]", report.get_roll()));
        let player = report.get_player_id().and_then(|id| game.player(id));
        print_player(status_report, game, indent + 1, false, player);
        if report.is_successful() {
            status_report.println_indent_style(indent + 1, TextStyle::NONE, " sabotages their weapon! They are KO'd and the blocker is knocked down.");
        } else {
            status_report.println_indent_style(indent + 1, TextStyle::NONE, " fails to detonate the weapon.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_player(id: &str, name: &str) -> Player {
        Player {
            id: id.into(),
            name: name.into(),
            gender: PlayerGender::Male,
            player_type: PlayerType::default(),
            ..Default::default()
        }
    }

    fn make_team(id: &str, players: Vec<Player>) -> Team {
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
            players,
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        let home = make_team("home", vec![make_player("p1", "Saboteur")]);
        let away = make_team("away", vec![]);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn report_id_is_saboteur_roll() {
        assert_eq!(SaboteurRollMessage.report_id(), ReportId::SABOTEUR_ROLL);
    }

    #[test]
    fn successful_roll_reports_sabotage() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportSaboteurRoll::new(Some("p1".into()), true, 4, 3, false);
        SaboteurRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "Saboteur Roll [ 4 ]"));
        assert!(texts.iter().any(|t| t == " sabotages their weapon! They are KO'd and the blocker is knocked down."));
    }

    #[test]
    fn unsuccessful_roll_reports_failure() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportSaboteurRoll::new(Some("p1".into()), false, 2, 3, false);
        SaboteurRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == " fails to detonate the weapon."));
    }

    #[test]
    fn roll_text_uses_roll_style() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportSaboteurRoll::new(Some("p1".into()), true, 6, 3, false);
        SaboteurRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::ROLL));
    }
}
