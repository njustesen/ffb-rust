use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::bb2020::report_kickoff_officious_ref::ReportKickoffOfficiousRef;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `KickoffOfficiousRefMessage.java`.
pub struct KickoffOfficiousRefMessage;

impl ReportMessage for KickoffOfficiousRefMessage {
    type Report = ReportKickoffOfficiousRef;

    fn report_id(&self) -> ReportId {
        ReportId::KICKOFF_OFFICIOUS_REF
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let game_result = &game.game_result;

        let mut status = format!("Officious Ref Roll Home Team [ {} ]", report.get_roll_home());
        status_report.println_indent_style(indent, TextStyle::ROLL, &status);
        let total_home = report.get_roll_home() + game_result.team_result(true).fan_factor;
        status = format!("Rolled {}", report.get_roll_home());
        status.push_str(&format!(" + {} Fan Factor", game_result.team_result(true).fan_factor));
        status.push_str(&format!(" = {}.", total_home));
        status_report.println_indent(indent + 1, &status);

        status = format!("Officious Ref Roll Away Team [ {} ]", report.get_roll_away());
        status_report.println_indent_style(indent, TextStyle::ROLL, &status);
        let total_away = report.get_roll_away() + game_result.team_result(false).fan_factor;
        status = format!("Rolled {}", report.get_roll_away());
        status.push_str(&format!(" + {} Fan Factor", game_result.team_result(false).fan_factor));
        status.push_str(&format!(" = {}.", total_away));
        status_report.println_indent(indent + 1, &status);

        for player_id in report.get_player_ids() {
            let player = game.player(player_id);
            print_player(status_report, game, indent + 1, false, player);
            status_report.println_indent(indent + 1, " gets into an argument with the ref.");
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
        Game::new(make_team("home"), make_team("away"), Rules::Bb2020)
    }

    #[test]
    fn report_id_matches() {
        assert_eq!(KickoffOfficiousRefMessage.report_id(), ReportId::KICKOFF_OFFICIOUS_REF);
    }

    #[test]
    fn renders_rolls_for_both_teams() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportKickoffOfficiousRef::new(3, 5, vec![]);
        KickoffOfficiousRefMessage.render(&mut sr, &game, &report);
        assert!(sr.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Officious Ref Roll Home Team [ 3 ]")));
        assert!(sr.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Officious Ref Roll Away Team [ 5 ]")));
    }

    #[test]
    fn no_players_means_no_argument_lines() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportKickoffOfficiousRef::new(1, 1, vec![]);
        KickoffOfficiousRefMessage.render(&mut sr, &game, &report);
        assert!(!sr.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" gets into an argument with the ref.")));
    }

    #[test]
    fn player_argument_line_rendered() {
        let mut sr = StatusReport::new();
        let mut game = make_game();
        game.team_home.players.push(Player { id: "p1".into(), name: "Bob".into(), ..Player::default() });
        let report = ReportKickoffOfficiousRef::new(1, 1, vec!["p1".to_string()]);
        KickoffOfficiousRefMessage.render(&mut sr, &game, &report);
        assert!(sr.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" gets into an argument with the ref.")));
        assert!(sr.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Bob")));
    }
}
