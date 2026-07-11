use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_player_action::ReportPlayerAction;
use ffb_model::util::string_tool::is_provided;

/// 1:1 translation of `PlayerActionMessage.java`.
pub struct PlayerActionMessage;

impl ReportMessage for PlayerActionMessage {
    type Report = ReportPlayerAction;

    fn report_id(&self) -> ReportId {
        ReportId::PLAYER_ACTION
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        status_report.set_indent(0);
        let player = game.player(report.get_acting_player_id());
        let player_action = report.get_player_action();
        let action_description = player_action.description();
        if player.is_some() && is_provided(action_description) {
            print_player(status_report, game, status_report.get_indent(), true, player);
            let text = format!(" {}.", action_description.unwrap());
            status_report.println_indent_style(status_report.get_indent(), TextStyle::BOLD, &text);
        }
        status_report.set_indent(status_report.get_indent() + 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerAction, Rules};
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
        assert_eq!(PlayerActionMessage.report_id(), ReportId::PLAYER_ACTION);
    }

    #[test]
    fn render_prints_player_and_description_when_present() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        let mut player = Player::default();
        player.id = "p1".into();
        player.name = "Grombrindal".into();
        game.team_home.players.push(player);
        let report = ReportPlayerAction::new("p1".into(), PlayerAction::Move);
        PlayerActionMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Grombrindal"));
        assert_eq!(status_report.rendered_runs[1].text.as_deref(), Some(" starts a Move Action."));
        assert_eq!(status_report.rendered_runs[1].text_style, Some(TextStyle::BOLD));
    }

    #[test]
    fn render_skips_output_when_description_is_none() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        let mut player = Player::default();
        player.id = "p1".into();
        game.team_home.players.push(player);
        let report = ReportPlayerAction::new("p1".into(), PlayerAction::Blitz);
        PlayerActionMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }

    #[test]
    fn render_skips_output_when_player_missing() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPlayerAction::new("nobody".into(), PlayerAction::Move);
        PlayerActionMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }

    #[test]
    fn render_sets_indent_to_one_afterward() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        status_report.set_indent(5);
        let report = ReportPlayerAction::new("nobody".into(), PlayerAction::Move);
        PlayerActionMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.get_indent(), 1);
    }
}
