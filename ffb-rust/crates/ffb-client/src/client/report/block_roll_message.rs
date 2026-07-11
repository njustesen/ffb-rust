use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::factory::block_result_factory::BlockResultFactory;
use ffb_model::model::game::Game;
use ffb_model::report::report_block_roll::ReportBlockRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `BlockRollMessage.java`.
pub struct BlockRollMessage;

impl ReportMessage for BlockRollMessage {
    type Report = ReportBlockRoll;

    fn report_id(&self) -> ReportId {
        ReportId::BLOCK_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        // java: ArrayTool.isProvided(report.getBlockRoll())
        if !report.get_block_roll().is_empty() {
            status_report.set_indent(2);
            let mut status = String::from("Block Roll");
            // java: StringTool.isProvided(report.getDefenderId())
            if report.get_defender_id().is_some_and(|id| !id.is_empty()) {
                status.push_str(" against ");
                status_report.print_indent_style(status_report.get_indent(), TextStyle::ROLL, &status);
                let defender = report.get_defender_id().and_then(|id| game.player(id));
                print_player(status_report, game, status_report.get_indent(), true, defender);
                status = String::new();
            }
            let block_result_factory = BlockResultFactory;
            for roll in report.get_block_roll() {
                if let Some(block_result) = block_result_factory.for_roll(*roll) {
                    status.push_str(" [ ");
                    status.push_str(block_result.name());
                    status.push_str(" ]");
                }
            }
            status_report.println_indent_style(status_report.get_indent(), TextStyle::ROLL, &status);
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
        assert_eq!(BlockRollMessage.report_id(), ReportId::BLOCK_ROLL);
    }

    #[test]
    fn empty_roll_renders_nothing() {
        let game = make_game();
        let report = ReportBlockRoll::new("home".into(), vec![], None);
        let mut status_report = StatusReport::new();
        BlockRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }

    #[test]
    fn roll_without_defender() {
        let game = make_game();
        let report = ReportBlockRoll::new("home".into(), vec![1, 6], None);
        let mut status_report = StatusReport::new();
        BlockRollMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.get_indent(), 2);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Block Roll [ SKULL ] [ POW ]"));
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::ROLL));
    }

    #[test]
    fn roll_with_defender() {
        let mut game = make_game();
        let mut defender = Player::default();
        defender.id = "def1".into();
        defender.name = "Defender".into();
        game.team_away.players.push(defender);

        let report = ReportBlockRoll::new("home".into(), vec![2, 5], Some("def1".into()));
        let mut status_report = StatusReport::new();
        BlockRollMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Block Roll against "));
        assert_eq!(status_report.rendered_runs[1].text.as_deref(), Some("Defender"));
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some(" [ BOTH DOWN ] [ POW/PUSH ]"));
    }
}
