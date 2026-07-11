use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::bb2025::report_kickoff_dodgy_snack::ReportKickoffDodgySnack;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `KickoffDodgySnackMessage.java`.
pub struct KickoffDodgySnackMessage;

impl ReportMessage for KickoffDodgySnackMessage {
    type Report = ReportKickoffDodgySnack;

    fn report_id(&self) -> ReportId {
        ReportId::KICKOFF_DODGY_SNACK
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        status_report.println_indent_style(
            indent,
            TextStyle::ROLL,
            &format!("Dodgy Snack Roll Home Team [ {} ]", report.get_roll_home()),
        );
        status_report.println_indent_style(
            indent,
            TextStyle::ROLL,
            &format!("Dodgy Snack Roll Away Team [ {} ]", report.get_roll_away()),
        );

        for player_id in report.get_player_ids() {
            let player = game.player(player_id);
            print_player(status_report, game, indent + 1, false, player);
            status_report.println_indent(indent + 1, " had a dodgy snack.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_player(id: &str, name: &str) -> Player {
        Player {
            id: id.into(),
            name: name.into(),
            gender: PlayerGender::Male,
            player_type: PlayerType::default(),
            ..Default::default()
        }
    }

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
        let home = make_team("home", vec![make_player("p1", "Hungry Guy")]);
        let away = make_team("away", vec![make_player("p2", "Snacker")]);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn reports_both_team_rolls() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportKickoffDodgySnack::new(3, 5, vec![]);
        KickoffDodgySnackMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.contains(&"Dodgy Snack Roll Home Team [ 3 ]".to_string()));
        assert!(texts.contains(&"Dodgy Snack Roll Away Team [ 5 ]".to_string()));
    }

    #[test]
    fn no_players_means_no_snack_lines() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportKickoffDodgySnack::new(1, 1, vec![]);
        KickoffDodgySnackMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(!texts.iter().any(|t| t.contains("dodgy snack")));
    }

    #[test]
    fn each_player_gets_a_dodgy_snack_line() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportKickoffDodgySnack::new(2, 2, vec!["p1".into(), "p2".into()]);
        KickoffDodgySnackMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "Hungry Guy"));
        assert!(texts.iter().any(|t| t == "Snacker"));
        assert_eq!(texts.iter().filter(|t| *t == " had a dodgy snack.").count(), 2);
    }

    #[test]
    fn report_id_is_kickoff_dodgy_snack() {
        assert_eq!(KickoffDodgySnackMessage.report_id(), ReportId::KICKOFF_DODGY_SNACK);
    }
}
