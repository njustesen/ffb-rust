use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::enums::PlayerAction;
use ffb_model::model::game::Game;
use ffb_model::report::report_block::ReportBlock;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `BlockMessage.java`.
pub struct BlockMessage;

impl ReportMessage for BlockMessage {
    type Report = ReportBlock;

    fn report_id(&self) -> ReportId {
        ReportId::BLOCK
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        status_report.set_indent(1);
        let acting_player = &game.acting_player;
        let attacker = acting_player.player_id.as_deref().and_then(|id| game.player(id));
        let defender = game.player(report.get_defender_id());

        print_player(status_report, game, status_report.get_indent(), true, attacker);
        if acting_player.player_action == Some(PlayerAction::Blitz) {
            status_report.print_indent_style(status_report.get_indent(), TextStyle::BOLD, " blitzes ");
        } else {
            status_report.print_indent_style(status_report.get_indent(), TextStyle::BOLD, " blocks ");
        }
        print_player(status_report, game, status_report.get_indent(), true, defender);
        status_report.println_indent_style(status_report.get_indent(), TextStyle::BOLD, ":");
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

    fn setup(game: &mut Game) {
        let mut attacker = Player::default();
        attacker.id = "att".into();
        attacker.name = "Attacker".into();
        game.team_home.players.push(attacker);
        let mut defender = Player::default();
        defender.id = "def".into();
        defender.name = "Defender".into();
        game.team_away.players.push(defender);
    }

    #[test]
    fn report_id_matches() {
        assert_eq!(BlockMessage.report_id(), ReportId::BLOCK);
    }

    #[test]
    fn renders_block_action() {
        let mut game = make_game();
        setup(&mut game);
        game.acting_player.player_id = Some("att".into());
        game.acting_player.player_action = Some(PlayerAction::Block);

        let report = ReportBlock::new("def".into());
        let mut status_report = StatusReport::new();
        BlockMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.get_indent(), 1);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Attacker"));
        assert_eq!(status_report.rendered_runs[1].text.as_deref(), Some(" blocks "));
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("Defender"));
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some(":"));
    }

    #[test]
    fn renders_blitz_action() {
        let mut game = make_game();
        setup(&mut game);
        game.acting_player.player_id = Some("att".into());
        game.acting_player.player_action = Some(PlayerAction::Blitz);

        let report = ReportBlock::new("def".into());
        let mut status_report = StatusReport::new();
        BlockMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs[1].text.as_deref(), Some(" blitzes "));
    }

    #[test]
    fn uses_bold_home_away_styles() {
        let mut game = make_game();
        setup(&mut game);
        game.acting_player.player_id = Some("att".into());
        game.acting_player.player_action = Some(PlayerAction::Block);

        let report = ReportBlock::new("def".into());
        let mut status_report = StatusReport::new();
        BlockMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::HOME_BOLD));
        assert_eq!(status_report.rendered_runs[2].text_style, Some(TextStyle::AWAY_BOLD));
    }
}
