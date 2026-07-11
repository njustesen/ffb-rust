use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::paragraph_style::ParagraphStyle;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_most_valuable_players::ReportMostValuablePlayers;

fn report_game_end(status_report: &mut StatusReport, game: &Game) {
    status_report.set_indent(0);
    let game_result = &game.game_result;
    let score_diff_home = game_result.home.score - game_result.away.score;
    if game_result.home.conceded {
        let status = format!("Coach {} concedes the game.", game.team_home.coach);
        status_report.println_style(Some(ParagraphStyle::SPACE_ABOVE_BELOW), Some(TextStyle::TURN_HOME), &status);
    } else if game_result.away.conceded {
        let status = format!("Coach {} concedes the game.", game.team_away.coach);
        status_report.println_style(Some(ParagraphStyle::SPACE_ABOVE_BELOW), Some(TextStyle::TURN_AWAY), &status);
    } else if score_diff_home > 0 {
        let status = format!("{} win the game.", game.team_home.name);
        status_report.println_style(Some(ParagraphStyle::SPACE_ABOVE_BELOW), Some(TextStyle::TURN_HOME), &status);
    } else if score_diff_home < 0 {
        let status = format!("{} win the game.", game.team_away.name);
        status_report.println_style(Some(ParagraphStyle::SPACE_ABOVE_BELOW), Some(TextStyle::TURN_AWAY), &status);
    } else {
        status_report.println_style(Some(ParagraphStyle::SPACE_ABOVE_BELOW), Some(TextStyle::TURN), "The game ends in a tie.");
    }
}

pub struct MostValuablePlayersMessage;

impl ReportMessage for MostValuablePlayersMessage {
    type Report = ReportMostValuablePlayers;

    fn report_id(&self) -> ReportId {
        ReportId::MOST_VALUABLE_PLAYERS
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        report_game_end(status_report, game);
        let indent = status_report.get_indent();
        status_report.println_indent_style(indent, TextStyle::BOLD, "Most Valuable Players");

        for player_id in report.get_player_ids_home() {
            if let Some(player) = game.player(player_id) {
                status_report.print_indent_style(indent + 1, TextStyle::NONE, "The jury voted ");
                status_report.print_indent_style(indent + 1, TextStyle::HOME, &player.name.clone());
                status_report.print_indent_style(indent + 1, TextStyle::NONE, " the most valuable player of ");
                status_report.print_indent_style(indent + 1, TextStyle::NONE, player.gender.genitive());
                status_report.println_indent_style(indent + 1, TextStyle::NONE, " team.");
            }
        }

        for player_id in report.get_player_ids_away() {
            if let Some(player) = game.player(player_id) {
                status_report.print_indent_style(indent + 1, TextStyle::NONE, "The jury voted ");
                status_report.print_indent_style(indent + 1, TextStyle::AWAY, &player.name.clone());
                status_report.print_indent_style(indent + 1, TextStyle::NONE, " the most valuable player of ");
                status_report.print_indent_style(indent + 1, TextStyle::NONE, player.gender.genitive());
                status_report.println_indent_style(indent + 1, TextStyle::NONE, " team.");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.into(), name: format!("Team {id}"), race: "Human".into(), roster_id: "human".into(), coach: format!("Coach{id}"),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false,
        }
    }

    fn make_game() -> Game {
        let mut home = make_team("home");
        home.players.push(Player {
            id: "p1".into(), name: "Grubb".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        });
        Game::new(home, make_team("away"), Rules::Bb2016)
    }

    #[test]
    fn get_key_is_most_valuable_players() {
        assert_eq!(MostValuablePlayersMessage.get_key(), "mostValuablePlayers");
    }

    #[test]
    fn home_win_reports_win_and_mvp() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        game.game_result.home.score = 2;
        let report = ReportMostValuablePlayers::new(vec!["p1".into()], vec![]);
        MostValuablePlayersMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Team home win the game.")));
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Grubb")));
    }

    #[test]
    fn tie_reports_tie_message() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportMostValuablePlayers::new(vec![], vec![]);
        MostValuablePlayersMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("The game ends in a tie.")));
    }

    #[test]
    fn conceded_reports_concession_message() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        game.game_result.away.conceded = true;
        let report = ReportMostValuablePlayers::new(vec![], vec![]);
        MostValuablePlayersMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Coach Coachaway concedes the game.")));
    }
}
