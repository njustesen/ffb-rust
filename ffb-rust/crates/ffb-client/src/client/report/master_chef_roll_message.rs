use crate::client::report::report_message_base::{print_team_name, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_master_chef_roll::ReportMasterChefRoll;

/// 1:1 translation of `MasterChefRollMessage.java`.
pub struct MasterChefRollMessage;

impl ReportMessage for MasterChefRollMessage {
    type Report = ReportMasterChefRoll;

    fn report_id(&self) -> ReportId {
        ReportId::MASTER_CHEF_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let roll = report.get_master_chef_roll();
        let status = format!("Master Chef Roll [ {} ][ {} ][ {} ]", roll[0], roll[1], roll[2]);
        status_report.println_indent_style(status_report.get_indent(), TextStyle::ROLL, &status);
        print_team_name(status_report, game, false, report.get_team_id());
        let stolen_status = match report.get_re_rolls_stolen() {
            0 => " no re-rolls from ".to_string(),
            1 => format!("{} re-roll from ", report.get_re_rolls_stolen()),
            n => format!("{n} re-rolls from "),
        };
        status_report.print_indent(status_report.get_indent() + 1, &format!(" steal {stolen_status}"));
        if game.team_home.id == report.get_team_id() {
            print_team_name(status_report, game, false, &game.team_away.id);
        } else {
            print_team_name(status_report, game, false, &game.team_home.id);
        }
        status_report.println_indent(status_report.get_indent() + 1, ".");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team { id: id.to_string(), name: id.to_string(), race: "human".into(), roster_id: "human".into(),
            coach: "coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: Vec::new(), players: Vec::new(), vampire_lord: false, necromancer: false }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2025)
    }

    #[test]
    fn zero_stolen_says_no_rerolls() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportMasterChefRoll::new("home".into(), vec![1, 2, 3], 0);
        MasterChefRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == " steal  no re-rolls from "));
        assert!(texts.iter().any(|t| t == "away"));
    }

    #[test]
    fn one_stolen_uses_singular() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportMasterChefRoll::new("away".into(), vec![4, 5, 3], 1);
        MasterChefRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == " steal 1 re-roll from "));
        assert!(texts.iter().any(|t| t == "home"));
    }

    #[test]
    fn multiple_stolen_uses_plural() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportMasterChefRoll::new("home".into(), vec![6, 6, 6], 3);
        MasterChefRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == " steal 3 re-rolls from "));
    }

    #[test]
    fn roll_header_shows_all_three_dice() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportMasterChefRoll::new("home".into(), vec![1, 2, 3], 0);
        MasterChefRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "Master Chef Roll [ 1 ][ 2 ][ 3 ]"));
    }

    #[test]
    fn get_key_matches_report_id() {
        assert_eq!(MasterChefRollMessage.get_key(), "masterChefRoll");
    }
}
