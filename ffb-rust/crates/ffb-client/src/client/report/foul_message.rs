use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_foul::ReportFoul;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `FoulMessage.java`.
pub struct FoulMessage;

impl ReportMessage for FoulMessage {
    type Report = ReportFoul;

    fn report_id(&self) -> ReportId {
        ReportId::FOUL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let attacker = game.acting_player.player_id.as_deref().and_then(|id| game.player(id));
        let defender = game.player(report.get_defender_id());
        print_player(status_report, game, status_report.get_indent(), true, attacker);
        status_report.print_indent_style(status_report.get_indent(), TextStyle::BOLD, " fouls ");
        print_player(status_report, game, status_report.get_indent(), true, defender);
        status_report.println_indent(status_report.get_indent(), ":");
        status_report.set_indent(status_report.get_indent() + 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.to_string(),
            name: id.to_string(),
            race: "human".into(),
            roster_id: "human".into(),
            coach: "coach".into(),
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
            special_rules: Vec::new(),
            players: Vec::new(),
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, home: bool, id: &str) {
        let mut player = Player::default();
        player.id = id.to_string();
        player.name = id.to_string();
        if home {
            game.team_home.players.push(player);
        } else {
            game.team_away.players.push(player);
        }
    }

    #[test]
    fn report_id_matches() {
        assert_eq!(FoulMessage.report_id(), ReportId::FOUL);
    }

    #[test]
    fn renders_attacker_fouls_defender() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_player(&mut game, true, "attacker");
        add_player(&mut game, false, "defender");
        game.acting_player.player_id = Some("attacker".into());
        let report = ReportFoul::new("defender".into());
        FoulMessage.render(&mut status_report, &game, &report);

        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert_eq!(texts, vec!["attacker", " fouls ", "defender", ":"]);
    }

    #[test]
    fn increments_indent_after_render() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_player(&mut game, true, "attacker");
        add_player(&mut game, false, "defender");
        game.acting_player.player_id = Some("attacker".into());
        let report = ReportFoul::new("defender".into());
        assert_eq!(status_report.get_indent(), 0);
        FoulMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.get_indent(), 1);
    }

    #[test]
    fn attacker_and_defender_use_home_away_styles() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_player(&mut game, true, "attacker");
        add_player(&mut game, false, "defender");
        game.acting_player.player_id = Some("attacker".into());
        let report = ReportFoul::new("defender".into());
        FoulMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::HOME_BOLD));
        assert_eq!(status_report.rendered_runs[2].text_style, Some(TextStyle::AWAY_BOLD));
    }
}
