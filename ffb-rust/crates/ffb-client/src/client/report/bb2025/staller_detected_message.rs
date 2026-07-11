use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_staller_detected::ReportStallerDetected;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `StallerDetectedMessage.java`.
pub struct StallerDetectedMessage;

impl ReportMessage for StallerDetectedMessage {
    type Report = ReportStallerDetected;

    fn report_id(&self) -> ReportId {
        ReportId::STALLER_DETECTED
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        status_report.println_indent_style(indent, TextStyle::BOLD, "Stalling Detection");
        let player = report.get_player_id().and_then(|id| game.player(id));
        print_player(status_report, game, indent + 1, true, player);
        status_report.println_indent_style(indent + 1, TextStyle::NONE, " could stall");
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
        let home = make_team("home", vec![make_player("p1", "Staller")]);
        let away = make_team("away", vec![]);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn report_id_is_staller_detected() {
        assert_eq!(StallerDetectedMessage.report_id(), ReportId::STALLER_DETECTED);
    }

    #[test]
    fn reports_stalling_detection_header() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportStallerDetected::new(Some("p1".into()));
        StallerDetectedMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "Stalling Detection"));
        assert!(texts.iter().any(|t| t == " could stall"));
    }

    #[test]
    fn player_printed_bold() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportStallerDetected::new(Some("p1".into()));
        StallerDetectedMessage.render(&mut status_report, &game, &report);
        let player_run = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("Staller")).unwrap();
        assert_eq!(player_run.text_style, Some(TextStyle::HOME_BOLD));
    }

    #[test]
    fn missing_player_id_prints_no_player_run() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportStallerDetected::new(None);
        StallerDetectedMessage.render(&mut status_report, &game, &report);
        assert!(!status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Staller")));
    }
}
