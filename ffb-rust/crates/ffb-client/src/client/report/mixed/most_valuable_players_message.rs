use crate::client::paragraph_style::ParagraphStyle;
use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_most_valuable_players::ReportMostValuablePlayers;

/// 1:1 translation of `MostValuablePlayersMessage.java`.
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
            let player = game.player(player_id);
            status_report.print_indent_style(indent + 1, TextStyle::NONE, "The jury voted ");
            if let Some(player) = player {
                status_report.print_indent_style(indent + 1, TextStyle::HOME, &player.name.clone());
            }
            status_report.print_indent_style(indent + 1, TextStyle::NONE, " the most valuable player of ");
            let genitive = player.map(|p| p.gender.genitive()).unwrap_or_default();
            status_report.print_indent_style(indent + 1, TextStyle::NONE, genitive);
            status_report.println_indent_style(indent + 1, TextStyle::NONE, " team.");
        }

        for player_id in report.get_player_ids_away() {
            let player = game.player(player_id);
            status_report.print_indent_style(indent + 1, TextStyle::NONE, "The jury voted ");
            if let Some(player) = player {
                status_report.print_indent_style(indent + 1, TextStyle::AWAY, &player.name.clone());
            }
            status_report.print_indent_style(indent + 1, TextStyle::NONE, " the most valuable player of ");
            let genitive = player.map(|p| p.gender.genitive()).unwrap_or_default();
            status_report.print_indent_style(indent + 1, TextStyle::NONE, genitive);
            status_report.println_indent_style(indent + 1, TextStyle::NONE, " team.");
        }
    }
}

/// Java: private `reportGameEnd()`.
fn report_game_end(status_report: &mut StatusReport, game: &Game) {
    status_report.set_indent(0);

    let game_result = &game.game_result;
    let score_diff_home = (game_result.home.score + game_result.home.penalty_score)
        - (game_result.away.score + game_result.away.penalty_score);

    if game_result.home.conceded {
        let mut status = format!("Coach {} concedes the game", game.team_home.coach);
        if game.conceded_legally {
            status.push_str(" without penalties due to excessive player loss");
        }
        status.push('.');
        status_report.println_style(Some(ParagraphStyle::SPACE_ABOVE_BELOW), Some(TextStyle::TURN_HOME), &status);
    } else if game_result.away.conceded {
        let mut status = format!("Coach {} concedes the game", game.team_away.coach);
        if game.conceded_legally {
            status.push_str(" without penalties due to excessive player loss");
        }
        status.push('.');
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

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_team(id: &str, coach: &str, players: Vec<Player>) -> Team {
        Team {
            id: id.into(),
            name: format!("Team {id}"),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: coach.into(),
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
        let home_player = Player { id: "h1".into(), name: "Homer".into(), ..Player::default() };
        let away_player = Player { id: "a1".into(), name: "Awey".into(), ..Player::default() };
        Game::new(
            make_team("home", "CoachHome", vec![home_player]),
            make_team("away", "CoachAway", vec![away_player]),
            Rules::Bb2020,
        )
    }

    #[test]
    fn home_wins_reports_mvp_for_both_sides() {
        let mut game = make_game();
        game.game_result.home.score = 2;
        let report = ReportMostValuablePlayers::new(vec!["h1".into()], vec!["a1".into()]);
        let mut status_report = StatusReport::new();
        MostValuablePlayersMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Team home win the game."));
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::TURN_HOME));
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.contains(&"Homer"));
        assert!(texts.contains(&"Awey"));
    }

    #[test]
    fn tie_reports_tie_message() {
        let game = make_game();
        let report = ReportMostValuablePlayers::new(vec![], vec![]);
        let mut status_report = StatusReport::new();
        MostValuablePlayersMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("The game ends in a tie."));
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::TURN));
    }

    #[test]
    fn home_conceded_legally() {
        let mut game = make_game();
        game.game_result.home.conceded = true;
        game.conceded_legally = true;
        let report = ReportMostValuablePlayers::new(vec![], vec![]);
        let mut status_report = StatusReport::new();
        MostValuablePlayersMessage.render(&mut status_report, &game, &report);
        assert_eq!(
            status_report.rendered_runs[0].text.as_deref(),
            Some("Coach CoachHome concedes the game without penalties due to excessive player loss.")
        );
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::TURN_HOME));
    }

    #[test]
    fn away_conceded_without_legal_flag() {
        let mut game = make_game();
        game.game_result.away.conceded = true;
        let report = ReportMostValuablePlayers::new(vec![], vec![]);
        let mut status_report = StatusReport::new();
        MostValuablePlayersMessage.render(&mut status_report, &game, &report);
        assert_eq!(
            status_report.rendered_runs[0].text.as_deref(),
            Some("Coach CoachAway concedes the game.")
        );
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::TURN_AWAY));
    }

    #[test]
    fn away_wins_on_score_diff() {
        let mut game = make_game();
        game.game_result.away.score = 3;
        let report = ReportMostValuablePlayers::new(vec![], vec![]);
        let mut status_report = StatusReport::new();
        MostValuablePlayersMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Team away win the game."));
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::TURN_AWAY));
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(MostValuablePlayersMessage.report_id(), ReportId::MOST_VALUABLE_PLAYERS);
        assert_eq!(MostValuablePlayersMessage.get_key(), "mostValuablePlayers");
    }
}
