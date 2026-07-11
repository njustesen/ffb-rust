use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_stand_up_roll::ReportStandUpRoll;

pub struct StandUpRollMessage;

impl ReportMessage for StandUpRollMessage {
    type Report = ReportStandUpRoll;

    fn report_id(&self) -> ReportId {
        ReportId::STAND_UP_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let player = game.acting_player.player_id.as_deref().and_then(|id| game.player(id));
        let status = format!("Stand Up Roll [ {} ]", report.get_roll());
        status_report.println_indent_style(indent, TextStyle::ROLL, &status);
        print_player(status_report, game, indent + 1, false, player);
        let mut needed_roll: Option<String> = None;
        if report.is_successful() {
            status_report.println_indent(indent + 1, " stands up.");
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Succeeded on a roll of {}+.", report.get_minimum_roll()));
            }
        } else {
            let status = format!(
                " doesn't get to {} feet.",
                player.map(|p| p.gender.genitive()).unwrap_or("")
            );
            status_report.println_indent(indent + 1, &status);
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Roll a {}+ to succeed.", report.get_minimum_roll()));
            }
        }
        if let Some(needed_roll) = needed_roll {
            status_report.println_indent_style(indent + 1, TextStyle::NEEDED_ROLL, &needed_roll);
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

    fn make_game_with_acting_player(id: &str) -> Game {
        let mut game = Game::new(make_team("home"), make_team("away"), Rules::Bb2025);
        let mut player = Player::default();
        player.id = id.to_string();
        player.name = "Thorsson".to_string();
        game.team_home.players.push(player);
        game.acting_player.player_id = Some(id.to_string());
        game
    }

    #[test]
    fn report_id_matches() {
        assert_eq!(StandUpRollMessage.report_id(), ReportId::STAND_UP_ROLL);
    }

    #[test]
    fn successful_stand_up_reports_success_and_needed_roll_when_not_rerolled() {
        let game = make_game_with_acting_player("p1");
        let report = ReportStandUpRoll::new(Some("p1".into()), true, 4, 1, false);
        let mut status_report = StatusReport::new();
        StandUpRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<Option<String>> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.contains(&Some("Stand Up Roll [ 4 ]".to_string())));
        assert!(texts.contains(&Some(" stands up.".to_string())));
        assert!(texts.contains(&Some("Succeeded on a roll of 3+.".to_string())));
    }

    #[test]
    fn failed_stand_up_reports_failure_and_needed_roll() {
        let game = make_game_with_acting_player("p1");
        let report = ReportStandUpRoll::new(Some("p1".into()), false, 1, 1, false);
        let mut status_report = StatusReport::new();
        StandUpRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<Option<String>> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.contains(&Some(" doesn't get to his feet.".to_string())));
        assert!(texts.contains(&Some("Roll a 3+ to succeed.".to_string())));
    }

    #[test]
    fn rerolled_success_omits_needed_roll() {
        let game = make_game_with_acting_player("p1");
        let report = ReportStandUpRoll::new(Some("p1".into()), true, 4, 1, true);
        let mut status_report = StatusReport::new();
        StandUpRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<Option<String>> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(!texts.iter().any(|t| t.as_deref().is_some_and(|s| s.starts_with("Succeeded on a roll of"))));
    }

    #[test]
    fn rerolled_failure_omits_needed_roll() {
        let game = make_game_with_acting_player("p1");
        let report = ReportStandUpRoll::new(Some("p1".into()), false, 1, 1, true);
        let mut status_report = StatusReport::new();
        StandUpRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<Option<String>> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(!texts.iter().any(|t| t.as_deref().is_some_and(|s| s.starts_with("Roll a"))));
    }
}
