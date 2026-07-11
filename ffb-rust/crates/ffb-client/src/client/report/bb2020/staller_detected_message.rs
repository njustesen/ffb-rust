use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_staller_detected::ReportStallerDetected;
use ffb_model::report::report_id::ReportId;
use ffb_model::util::string_tool;

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
        if string_tool::is_provided(report.get_player_id()) {
            let player = report.get_player_id().and_then(|id| game.player(id));
            print_player(status_report, game, indent + 1, true, player);
        } else {
            status_report.print_indent_style(indent + 1, TextStyle::NONE, "Nobody");
        }
        status_report.println_indent_style(indent + 1, TextStyle::NONE, " is stalling");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_player(id: &str) -> Player {
        Player {
            id: id.into(),
            name: format!("Player {id}"),
            nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6,
            strength: 3,
            agility: 3,
            passing: 4,
            armour: 8,
            ..Default::default()
        }
    }

    fn make_team(id: &str, players: Vec<Player>) -> Team {
        Team {
            id: id.into(),
            name: "Team".into(),
            race: String::new(),
            roster_id: String::new(),
            coach: String::new(),
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
        Game::new(
            make_team("home", vec![make_player("p1")]),
            make_team("away", vec![]),
            Rules::Bb2020,
        )
    }

    #[test]
    fn header_is_always_printed() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportStallerDetected::new(None);
        StallerDetectedMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.contains(&"Stalling Detection"));
    }

    #[test]
    fn player_present_prints_player_name() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportStallerDetected::new(Some("p1".into()));
        StallerDetectedMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.contains(&"Player p1"));
        assert!(texts.iter().any(|t| t.contains(" is stalling")));
    }

    #[test]
    fn no_player_prints_nobody() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportStallerDetected::new(None);
        StallerDetectedMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.contains(&"Nobody"));
    }

    #[test]
    fn empty_player_id_treated_as_absent() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportStallerDetected::new(Some("".into()));
        StallerDetectedMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.contains(&"Nobody"));
    }
}
