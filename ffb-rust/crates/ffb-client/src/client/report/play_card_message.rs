use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_play_card::ReportPlayCard;
use ffb_model::util::string_tool::is_provided;

/// 1:1 translation of `PlayCardMessage.java`.
pub struct PlayCardMessage;

impl ReportMessage for PlayCardMessage {
    type Report = ReportPlayCard;

    fn report_id(&self) -> ReportId {
        ReportId::PLAY_CARD
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let mut status = format!("Card {}", report.get_card());
        if is_provided(report.get_player_id()) {
            status.push_str(" is played on ");
        } else {
            status.push_str(" is played.");
        }
        status_report.print_indent_style(status_report.get_indent(), TextStyle::BOLD, &status);
        if is_provided(report.get_player_id()) {
            let player = report.get_player_id().and_then(|id| game.player(id));
            print_player(status_report, game, status_report.get_indent(), true, player);
            status_report.println_indent_style(status_report.get_indent(), TextStyle::BOLD, ".");
        } else {
            status_report.println();
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
            id: id.to_string(), name: id.to_string(), race: "human".into(), roster_id: "human".into(),
            coach: "coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: Vec::new(), players: Vec::new(), vampire_lord: false, necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2025)
    }

    #[test]
    fn report_id_matches() {
        assert_eq!(PlayCardMessage.report_id(), ReportId::PLAY_CARD);
    }

    #[test]
    fn render_without_player_prints_played_and_blank_line() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPlayCard::new("home".into(), "Bribery".into());
        PlayCardMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Card Bribery is played."));
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::BOLD));
        // println() pushes two runs with no text.
        assert_eq!(status_report.rendered_runs.len(), 3);
        assert_eq!(status_report.rendered_runs[1].text, None);
    }

    #[test]
    fn render_with_player_prints_player_name_and_period() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        let mut player = Player::default();
        player.id = "p1".into();
        player.name = "Grombrindal".into();
        game.team_home.players.push(player);
        let report = ReportPlayCard::new_with_player("home".into(), "Bribery".into(), Some("p1".into()));
        PlayCardMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Card Bribery is played on "));
        assert_eq!(status_report.rendered_runs[1].text.as_deref(), Some("Grombrindal"));
        assert_eq!(status_report.rendered_runs[1].text_style, Some(TextStyle::HOME_BOLD));
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("."));
    }

    #[test]
    fn render_with_player_on_away_team_uses_away_style() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        let mut player = Player::default();
        player.id = "p2".into();
        player.name = "Skitter".into();
        game.team_away.players.push(player);
        let report = ReportPlayCard::new_with_player("away".into(), "Poison".into(), Some("p2".into()));
        PlayCardMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[1].text_style, Some(TextStyle::AWAY_BOLD));
    }
}
