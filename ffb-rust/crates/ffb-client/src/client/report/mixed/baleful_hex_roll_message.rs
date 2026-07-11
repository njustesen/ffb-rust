use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_baleful_hex_roll::ReportBalefulHexRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `BalefulHexRollMessage.java`.
pub struct BalefulHexRollMessage;

impl ReportMessage for BalefulHexRollMessage {
    type Report = ReportBalefulHexRoll;

    fn report_id(&self) -> ReportId {
        ReportId::BALEFUL_HEX
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let acting_player = game.acting_player.player_id.as_deref().and_then(|id| game.player(id));

        status_report.println_indent_style(indent, TextStyle::ROLL, &format!("Baleful Hex Roll [ {} ]", report.get_roll()));

        print_player(status_report, game, indent + 1, false, acting_player);
        let status = if report.is_successful() { " makes " } else { " fails to make " };
        status_report.print_indent(indent + 1, status);

        let target = report.get_target().and_then(|id| game.player(id));
        print_player(status_report, game, indent + 1, false, target);
        status_report.println_indent(indent + 1, " miss a turn.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::acting_player::ActingPlayer;
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
        let attacker = Player { id: "p1".into(), name: "Hexer".into(), ..Player::default() };
        let target = Player { id: "p2".into(), name: "Victim".into(), ..Player::default() };
        let mut game = Game::new(make_team("home", vec![attacker]), make_team("away", vec![target]), Rules::Bb2020);
        game.acting_player = ActingPlayer { player_id: Some("p1".into()), ..Default::default() };
        game
    }

    #[test]
    fn successful_makes_target_miss_turn() {
        let game = make_game();
        let report = ReportBalefulHexRoll::new(None, true, 5, 2, false, Some("p2".into()));
        let mut sr = StatusReport::new();
        BalefulHexRollMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some("Baleful Hex Roll [ 5 ]"));
        assert_eq!(sr.rendered_runs[2].text.as_deref(), Some("Hexer"));
        assert_eq!(sr.rendered_runs[3].text.as_deref(), Some(" makes "));
        assert_eq!(sr.rendered_runs[4].text.as_deref(), Some("Victim"));
        assert_eq!(sr.rendered_runs[5].text.as_deref(), Some(" miss a turn."));
    }

    #[test]
    fn unsuccessful_fails_to_make_target_miss_turn() {
        let game = make_game();
        let report = ReportBalefulHexRoll::new(None, false, 1, 2, false, Some("p2".into()));
        let mut sr = StatusReport::new();
        BalefulHexRollMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[3].text.as_deref(), Some(" fails to make "));
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(BalefulHexRollMessage.report_id(), ReportId::BALEFUL_HEX);
        assert_eq!(BalefulHexRollMessage.get_key(), "balefulHex");
    }
}
