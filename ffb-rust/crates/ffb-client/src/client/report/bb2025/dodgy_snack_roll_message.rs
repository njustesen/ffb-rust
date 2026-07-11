use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::bb2025::report_dodgy_snack_roll::ReportDodgySnackRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `DodgySnackRollMessage.java`.
pub struct DodgySnackRollMessage;

impl ReportMessage for DodgySnackRollMessage {
    type Report = ReportDodgySnackRoll;

    fn report_id(&self) -> ReportId {
        ReportId::DODGY_SNACK_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        status_report.println_indent_style(
            status_report.get_indent(),
            TextStyle::ROLL,
            &format!("Dodgy Snack Effect Roll [ {} ]", report.get_roll()),
        );
        let player = game.player(report.get_player_id());
        let indent = status_report.get_indent() + 1;
        print_player(status_report, game, indent, false, player);
        let message = if report.get_roll() == 1 {
            " is sent to reserves."
        } else {
            " suffers -MA and -AV for this drive."
        };
        status_report.println_indent_style(indent, TextStyle::NONE, message);
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
        let home = make_team("home", vec![make_player("p1", "Player One")]);
        let away = make_team("away", vec![]);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn roll_of_one_sends_to_reserves() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportDodgySnackRoll::new(1, "p1".into());
        DodgySnackRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == " is sent to reserves."));
    }

    #[test]
    fn other_roll_suffers_ma_av_penalty() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportDodgySnackRoll::new(4, "p1".into());
        DodgySnackRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == " suffers -MA and -AV for this drive."));
    }

    #[test]
    fn prints_roll_header() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportDodgySnackRoll::new(6, "p1".into());
        DodgySnackRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "Dodgy Snack Effect Roll [ 6 ]"));
        assert!(texts.iter().any(|t| t == "Player One"));
    }

    #[test]
    fn report_id_is_dodgy_snack_roll() {
        assert_eq!(DodgySnackRollMessage.report_id(), ReportId::DODGY_SNACK_ROLL);
    }
}
