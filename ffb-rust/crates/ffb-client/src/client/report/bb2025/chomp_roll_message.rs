use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::bb2025::report_chomp_roll::ReportChompRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `ChompRollMessage.java`.
pub struct ChompRollMessage;

impl ReportMessage for ChompRollMessage {
    type Report = ReportChompRoll;

    fn report_id(&self) -> ReportId {
        ReportId::CHOMP_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let player = game.player(report.get_chomper());
        status_report.println_indent_style(
            status_report.get_indent(),
            TextStyle::ROLL,
            &format!("Chomp Roll [ {} ]", report.get_roll()),
        );
        let indent = status_report.get_indent() + 1;
        print_player(status_report, game, indent, false, player);
        let status = if report.is_successful() { " chomped " } else { " failed to chomp " };
        status_report.print_indent(indent, status);

        let defender = game.player(report.get_chompee());
        print_player(status_report, game, indent, false, defender);

        status_report.println_indent(indent, ".");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerType, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_player(id: &str, name: &str) -> Player {
        Player { id: id.into(), name: name.into(), player_type: PlayerType::default(), ..Default::default() }
    }

    fn make_team(id: &str, players: Vec<Player>) -> Team {
        Team {
            id: id.into(), name: format!("Team {id}"), race: "Human".into(), roster_id: "human".into(),
            coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players, vampire_lord: false, necromancer: false,
        }
    }

    fn make_game() -> Game {
        let home = make_team("home", vec![make_player("chomper1", "Chomper")]);
        let away = make_team("away", vec![make_player("chompee1", "Chompee")]);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn successful_chomp() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportChompRoll::new(Some("chomper1".into()), true, 5, 3, false, "chomper1".into(), "chompee1".into());
        ChompRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == " chomped "));
    }

    #[test]
    fn failed_chomp() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportChompRoll::new(Some("chomper1".into()), false, 2, 3, false, "chomper1".into(), "chompee1".into());
        ChompRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == " failed to chomp "));
    }

    #[test]
    fn prints_roll_and_both_players() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportChompRoll::new(Some("chomper1".into()), true, 6, 3, false, "chomper1".into(), "chompee1".into());
        ChompRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "Chomp Roll [ 6 ]"));
        assert!(texts.iter().any(|t| t == "Chomper"));
        assert!(texts.iter().any(|t| t == "Chompee"));
    }

    #[test]
    fn report_id_is_chomp_roll() {
        assert_eq!(ChompRollMessage.report_id(), ReportId::CHOMP_ROLL);
    }
}
