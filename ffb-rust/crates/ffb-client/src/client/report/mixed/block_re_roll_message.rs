use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_mechanics::mechanics::block_result_for_roll;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_block_re_roll::ReportBlockReRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `BlockReRollMessage.java`. Java hardcodes indent `2`/`3` (not
/// `getIndent()`) — preserved literally here.
pub struct BlockReRollMessage;

impl ReportMessage for BlockReRollMessage {
    type Report = ReportBlockReRoll;

    fn report_id(&self) -> ReportId {
        ReportId::BLOCK_RE_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let player = report.get_player_id().and_then(|id| game.player(id));

        let mut status = String::from("Re-Rolled Block Dice");
        for &roll in report.get_block_roll() {
            let block_result = block_result_for_roll(roll);
            status.push_str(&format!(" [ {} ]", block_result.name()));
        }
        status_report.println_indent_style(2, TextStyle::ROLL, &status);

        let dice_count = report.get_block_roll().len();
        let mut status = format!(" re-rolled {dice_count} block ");
        status.push_str(if dice_count == 1 { "die" } else { "dice" });
        let source_name = report.get_re_roll_source().map(|s| s.name.as_str()).unwrap_or("");
        status.push_str(&format!(" using {}.", source_name));

        print_player(status_report, game, 3, true, player);
        status_report.println_indent_style(3, TextStyle::EXPLANATION, &status);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{ReRollSource, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_team(id: &str, players: Vec<Player>) -> Team {
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
            players,
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        let player = Player { id: "p1".into(), name: "Reroller".into(), ..Player::default() };
        Game::new(make_team("home", vec![player]), make_team("away", vec![]), Rules::Bb2020)
    }

    #[test]
    fn single_die_uses_singular_wording() {
        let game = make_game();
        let report = ReportBlockReRoll::new(vec![2], Some("p1".into()), Some(ReRollSource::new("Team Re-Roll")));
        let mut sr = StatusReport::new();
        BlockReRollMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some("Re-Rolled Block Dice [ BOTH DOWN ]"));
        assert_eq!(sr.rendered_runs[2].text.as_deref(), Some("Reroller"));
        assert_eq!(sr.rendered_runs[3].text.as_deref(), Some(" re-rolled 1 block die using Team Re-Roll."));
    }

    #[test]
    fn multiple_dice_use_plural_wording_and_join_results() {
        let game = make_game();
        let report = ReportBlockReRoll::new(vec![1, 6], Some("p1".into()), Some(ReRollSource::new("Pro")));
        let mut sr = StatusReport::new();
        BlockReRollMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some("Re-Rolled Block Dice [ SKULL ] [ POW ]"));
        assert_eq!(sr.rendered_runs[3].text.as_deref(), Some(" re-rolled 2 block dice using Pro."));
    }

    #[test]
    fn indent_is_hardcoded_regardless_of_status_report_indent() {
        let game = make_game();
        let mut sr = StatusReport::new();
        sr.set_indent(5);
        let report = ReportBlockReRoll::new(vec![3], Some("p1".into()), None);
        BlockReRollMessage.render(&mut sr, &game, &report);
        use crate::client::paragraph_style::ParagraphStyle;
        assert_eq!(sr.rendered_runs[0].paragraph_style, Some(ParagraphStyle::INDENT_2));
        assert_eq!(sr.rendered_runs[2].paragraph_style, Some(ParagraphStyle::INDENT_3));
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(BlockReRollMessage.report_id(), ReportId::BLOCK_RE_ROLL);
        assert_eq!(BlockReRollMessage.get_key(), "blockReRoll");
    }
}
