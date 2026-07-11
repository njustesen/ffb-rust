use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_bite_spectator::ReportBiteSpectator;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `BiteSpectatorMessage.java`.
pub struct BiteSpectatorMessage;

impl ReportMessage for BiteSpectatorMessage {
    type Report = ReportBiteSpectator;

    fn report_id(&self) -> ReportId {
        ReportId::BITE_SPECTATOR
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let player = game.player(report.get_player_id());
        if player.is_some() {
            print_player(status_report, game, status_report.get_indent(), true, player);
            status_report.println_indent_style(
                status_report.get_indent(),
                TextStyle::BOLD,
                " heads off to the spectator ranks to bite some beautiful maiden.",
            );
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
        assert_eq!(BiteSpectatorMessage.report_id(), ReportId::BITE_SPECTATOR);
    }

    #[test]
    fn renders_for_known_player() {
        let mut game = make_game();
        let mut player = Player::default();
        player.id = "p1".into();
        player.name = "Biter".into();
        game.team_home.players.push(player);

        let report = ReportBiteSpectator::new("p1".into());
        let mut status_report = StatusReport::new();
        BiteSpectatorMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Biter"));
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::HOME_BOLD));
        assert_eq!(status_report.rendered_runs[1].text.as_deref(), Some(" heads off to the spectator ranks to bite some beautiful maiden."));
    }

    #[test]
    fn renders_away_player_bold() {
        let mut game = make_game();
        let mut player = Player::default();
        player.id = "p2".into();
        player.name = "AwayBiter".into();
        game.team_away.players.push(player);

        let report = ReportBiteSpectator::new("p2".into());
        let mut status_report = StatusReport::new();
        BiteSpectatorMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::AWAY_BOLD));
    }

    #[test]
    fn skips_render_for_unknown_player() {
        let game = make_game();
        let report = ReportBiteSpectator::new("missing".into());
        let mut status_report = StatusReport::new();
        BiteSpectatorMessage.render(&mut status_report, &game, &report);

        assert!(status_report.rendered_runs.is_empty());
    }
}
