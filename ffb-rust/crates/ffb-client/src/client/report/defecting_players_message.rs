use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_defecting_players::ReportDefectingPlayers;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `DefectingPlayersMessage.java`.
pub struct DefectingPlayersMessage;

impl ReportMessage for DefectingPlayersMessage {
    type Report = ReportDefectingPlayers;

    fn report_id(&self) -> ReportId {
        ReportId::DEFECTING_PLAYERS
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let player_ids = report.get_player_ids();
        if player_ids.is_empty() {
            return;
        }
        let rolls = report.get_rolls();
        let defecting = report.get_defectings();
        for (i, player_id) in player_ids.iter().enumerate() {
            let status = format!("Defecting Players Roll [ {} ]", rolls[i]);
            status_report.println_indent_style(status_report.get_indent(), TextStyle::ROLL, &status);
            let player = game.player(player_id);
            print_player(status_report, game, status_report.get_indent() + 1, false, player);
            if defecting[i] {
                status_report.println_indent_style(status_report.get_indent() + 1, TextStyle::NONE, " leaves the team in disgust.");
            } else {
                status_report.println_indent_style(status_report.get_indent() + 1, TextStyle::NONE, " stays with the team.");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team { id: id.to_string(), name: id.to_string(), race: "human".into(), roster_id: "human".into(),
            coach: "coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: Vec::new(), players: Vec::new(), vampire_lord: false, necromancer: false }
    }

    fn make_game() -> Game {
        let mut game = Game::new(make_team("home"), make_team("away"), Rules::Bb2025);
        let mut p1 = Player::default();
        p1.id = "p1".into();
        p1.name = "Loyal".into();
        let mut p2 = Player::default();
        p2.id = "p2".into();
        p2.name = "Deserter".into();
        game.team_home.players.push(p1);
        game.team_home.players.push(p2);
        game
    }

    #[test]
    fn empty_player_ids_renders_nothing() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportDefectingPlayers::new(vec![], vec![], vec![]);
        DefectingPlayersMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }

    #[test]
    fn defecting_player_prints_disgust_message() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportDefectingPlayers::new(vec!["p2".into()], vec![1], vec![true]);
        DefectingPlayersMessage.render(&mut status_report, &game, &report);
        let msg = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some(" leaves the team in disgust."));
        assert!(msg.is_some());
    }

    #[test]
    fn staying_player_prints_stays_message() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportDefectingPlayers::new(vec!["p1".into()], vec![5], vec![false]);
        DefectingPlayersMessage.render(&mut status_report, &game, &report);
        let msg = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some(" stays with the team."));
        assert!(msg.is_some());
    }

    #[test]
    fn multiple_players_render_one_block_each() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportDefectingPlayers::new(vec!["p1".into(), "p2".into()], vec![2, 6], vec![false, true]);
        DefectingPlayersMessage.render(&mut status_report, &game, &report);
        let roll_lines = status_report.rendered_runs.iter().filter(|r| r.text_style == Some(TextStyle::ROLL)).count();
        assert_eq!(roll_lines, 2);
        let names: Vec<&str> = status_report.rendered_runs.iter().filter_map(|r| {
            if r.text.as_deref() == Some("Loyal") || r.text.as_deref() == Some("Deserter") { r.text.as_deref() } else { None }
        }).collect();
        assert_eq!(names, vec!["Loyal", "Deserter"]);
    }

    #[test]
    fn report_id_is_defecting_players() {
        assert_eq!(DefectingPlayersMessage.report_id(), ReportId::DEFECTING_PLAYERS);
    }
}
